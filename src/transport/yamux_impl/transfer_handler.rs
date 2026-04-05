// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

use std::sync::{Arc, OnceLock};

use crate::error::{Result, VirgeError};
use futures::future::poll_fn;
use futures::AsyncReadExt;
use futures::AsyncWriteExt;
use log::*;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tokio_vsock::{VsockAddr, VsockStream};

use yamux::Stream;
use yamux::{Config, Connection, Mode};

/// 消息长度前缀的字节数（使用 usize, 8字节）
const LENGTH_PREFIX_SIZE: usize = 8;

/// 全局 tokio 运行时（多线程）
static TOKIO_RT: OnceLock<Runtime> = OnceLock::new();

pub fn get_runtime() -> &'static Runtime {
    TOKIO_RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for yamux")
    })
}

/// Yamux 传输协议处理器
///
/// 对外提供同步接口，内部通过 tokio runtime 驱动 yamux 异步操作。
/// Connection 所有权在获取 stream 后移交给 driver task，避免死锁。
///
/// 使用 Arc<Mutex<Stream>> 在 block_on 和 driver task 之间共享 stream，
/// 避免 block_on 阻塞整个 runtime 导致死锁。
pub struct YamuxTransportHandler {
    yamux_stream: Option<Arc<tokio::sync::Mutex<Stream>>>,
    driver_handle: Option<JoinHandle<()>>,
    mode: Mode,
}

impl YamuxTransportHandler {
    pub fn new(mode: Mode) -> Self {
        Self {
            yamux_stream: None,
            driver_handle: None,
            mode,
        }
    }
}

impl YamuxTransportHandler {
    /// 客户端连接到 vsock 地址
    pub fn connect(&mut self, cid: u32, port: u32, _chunk_size: u32, _is_ack: bool) -> Result<()> {
        info!("Yamux transport connecting to cid={}, port={}", cid, port);

        let vsock_stream = get_runtime()
            .block_on(async { VsockStream::connect(VsockAddr::new(cid, port)).await })
            .map_err(|e| VirgeError::ConnectionError(format!("Failed to connect vsock: {}", e)))?;

        let config = Config::default();
        let mut connection = Connection::new(vsock_stream.compat(), config, Mode::Client);
        self.mode = Mode::Client;

        // 获取 outbound stream
        let stream = get_runtime()
            .block_on(async { poll_fn(|cx| connection.poll_new_outbound(cx)).await })
            .map_err(|e| {
                VirgeError::TransportError(format!("Failed to open yamux outbound stream: {}", e))
            })?;
        self.yamux_stream = Some(Arc::new(tokio::sync::Mutex::new(stream)));

        // 将 connection 移交给 driver task
        let handle = get_runtime().spawn(async move {
            debug!("Yamux connection driver started");
            loop {
                match poll_fn(|cx| connection.poll_next_inbound(cx)).await {
                    Some(Ok(_stream)) => {}
                    Some(Err(e)) => {
                        warn!("Yamux connection error in driver: {}", e);
                        break;
                    }
                    None => {
                        debug!("Yamux connection closed (driver)");
                        break;
                    }
                }
            }
            debug!("Yamux connection driver stopped");
        });
        self.driver_handle = Some(handle);

        info!("Yamux transport connected successfully");
        Ok(())
    }

    /// 从已有的 VsockStream 初始化（服务端模式）
    pub fn from_tokio_stream(&mut self, vsock_stream: VsockStream) -> Result<()> {
        let config = Config::default();
        let mut connection = Connection::new(vsock_stream.compat(), config, Mode::Server);
        self.mode = Mode::Server;

        // 等待客户端打开的 inbound stream
        let stream_result =
            get_runtime().block_on(async { poll_fn(|cx| connection.poll_next_inbound(cx)).await });

        match stream_result {
            Some(Ok(s)) => {
                self.yamux_stream = Some(Arc::new(tokio::sync::Mutex::new(s)));
            }
            Some(Err(e)) => {
                return Err(VirgeError::TransportError(format!(
                    "Failed to accept yamux inbound stream: {}",
                    e
                )));
            }
            None => {
                return Err(VirgeError::TransportError(
                    "Yamux connection closed, no inbound stream".into(),
                ));
            }
        }

        // 将 connection 移交给 driver task
        let handle = get_runtime().spawn(async move {
            debug!("Yamux server connection driver started");
            loop {
                match poll_fn(|cx| connection.poll_next_inbound(cx)).await {
                    Some(Ok(_stream)) => {}
                    Some(Err(e)) => {
                        warn!("Yamux server connection error in driver: {}", e);
                        break;
                    }
                    None => {
                        debug!("Yamux server connection poll returned None (connection closed)");
                        break;
                    }
                }
            }
            debug!("Yamux server connection driver stopped");
        });
        self.driver_handle = Some(handle);

        info!("Yamux transport initialized from stream (server mode)");
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        info!("Yamux transport disconnecting");

        // 关闭 stream（会发送 FIN 帧）
        if let Some(stream) = self.yamux_stream.take() {
            let _ = get_runtime().block_on(async {
                let mut s = stream.lock().await;
                // 先 flush 确保所有数据发送完成
                let _ = s.flush().await;
                // 然后关闭
                let _ = s.close().await;
            });
        }

        // 给 driver 一点时间处理关闭帧
        std::thread::sleep(std::time::Duration::from_secs(1));

        // 等待 driver 退出
        if let Some(handle) = self.driver_handle.take() {
            let _ = get_runtime().block_on(async {
                let _ = tokio::time::timeout(std::time::Duration::from_secs(2), handle).await;
            });
        }

        info!("Yamux transport disconnected");
        Ok(())
    }

    /// 发送数据（使用长度前缀协议）
    pub fn send(&mut self, data: &[u8]) -> Result<usize> {
        let stream = self
            .yamux_stream
            .as_ref()
            .ok_or_else(|| VirgeError::TransportError("Yamux stream not available".into()))?
            .clone();

        let data_len = data.len();
        let data = data.to_vec();

        // 使用 spawn 在独立任务中执行，避免阻塞 driver
        get_runtime().block_on(async {
            let send_task = tokio::spawn(async move {
                let mut s = stream.lock().await;

                // 先发送8字节的长度前缀
                let len = data.len() as usize;
                let len_bytes = len.to_be_bytes();
                s.write_all(&len_bytes)
                    .await
                    .map_err(|e| VirgeError::Other(format!("yamux send length error: {}", e)))?;

                // 再发送实际数据
                s.write_all(&data)
                    .await
                    .map_err(|e| VirgeError::Other(format!("yamux send error: {}", e)))?;

                // flush 确保数据发送出去
                s.flush()
                    .await
                    .map_err(|e| VirgeError::Other(format!("yamux flush error: {}", e)))?;

                Ok::<_, VirgeError>(())
            });

            send_task
                .await
                .map_err(|e| VirgeError::Other(format!("send task join error: {}", e)))?
        })?;

        debug!("Yamux sent {} bytes (with length prefix)", data_len);
        Ok(data_len)
    }

    /// 接收数据（使用长度前缀协议）
    pub fn recv(&mut self) -> Result<Vec<u8>> {
        let stream = self
            .yamux_stream
            .as_ref()
            .ok_or_else(|| VirgeError::TransportError("Yamux stream not available".into()))?
            .clone();

        let data = get_runtime().block_on(async {
            let recv_task = tokio::spawn(async move {
                let mut s = stream.lock().await;

                // 先读取8字节的长度前缀
                let mut len_buf = [0u8; LENGTH_PREFIX_SIZE];
                s.read_exact(&mut len_buf)
                    .await
                    .map_err(|e| VirgeError::Other(format!("yamux recv length error: {}", e)))?;

                let len = u64::from_be_bytes(len_buf) as usize;
                debug!("Yamux expecting to receive {} bytes", len);

                // 读取实际数据
                let mut buf = vec![0u8; len];
                s.read_exact(&mut buf)
                    .await
                    .map_err(|e| VirgeError::Other(format!("yamux recv error: {}", e)))?;

                Ok::<Vec<u8>, VirgeError>(buf)
            });

            recv_task
                .await
                .map_err(|e| VirgeError::Other(format!("recv task join error: {}", e)))?
        })?;

        debug!("Yamux received {} bytes", data.len());
        Ok(data)
    }

    pub fn is_connected(&self) -> bool {
        self.yamux_stream.is_some()
    }
}
