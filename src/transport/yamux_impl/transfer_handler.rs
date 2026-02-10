use std::sync::OnceLock;

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

/// 全局 tokio 运行时，供所有 yamux 操作使用。
/// 由于对外接口是同步的，内部通过此运行时调度异步操作。
static TOKIO_RT: OnceLock<Runtime> = OnceLock::new();

pub fn get_runtime() -> &'static Runtime {
    TOKIO_RT.get_or_init(|| {
        Runtime::new().expect("Failed to create tokio runtime for yamux")
    })
}

/// Yamux 传输协议实现
///
/// 直接管理 tokio-vsock 连接并使用 yamux 进行多路复用。
///
/// ## 设计说明
///
/// Yamux 的 `Connection` 需要持续驱动（通过 `poll_next_inbound`）来处理协议帧。
/// **Connection 的所有权在获取 stream 后移交给 driver task**，避免锁竞争和死锁。
///
/// - **客户端模式**：通过 `poll_new_outbound` 获取 yamux stream 进行 I/O，
///   然后将 connection 所有权移交给后台 tokio task，由 `poll_next_inbound` 驱动协议。
///   Stream I/O 和 connection driver 在不同 task 中并发运行，无需锁。
/// - **服务端模式**：通过 `poll_next_inbound` 获取客户端打开的 yamux stream 进行 I/O，
///   然后将 connection 所有权移交给后台 driver task 继续驱动。
///
/// 对外提供同步接口（`connect`/`send`/`recv`/`disconnect`），
/// 内部通过全局 tokio runtime 的 `block_on` 桥接异步操作。
pub struct YamuxTransportHandler {
    yamux_stream: Option<Stream>,
    /// driver task 的 JoinHandle，用于断开时等待 driver 退出
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

// ── 公开同步接口 ────────────────────────────────────────────

impl YamuxTransportHandler {
    /// 客户端连接到远端 vsock 地址，建立 yamux session 并打开 stream
    pub fn connect(&mut self, cid: u32, port: u32, _chunk_size: u32, _is_ack: bool) -> Result<()> {
        info!("Yamux transport connecting to cid={}, port={}", cid, port);

        let vsock_stream = get_runtime()
            .block_on(async { VsockStream::connect(VsockAddr::new(cid, port)).await })
            .map_err(|e| {
                VirgeError::ConnectionError(format!("Failed to connect vsock: {}", e))
            })?;

        let config = Config::default();
        let mut connection = Connection::new(vsock_stream.compat(), config, Mode::Client);
        self.mode = Mode::Client;

        // 1. 先从 connection 获取 outbound stream
        let stream = get_runtime()
            .block_on(async {
                poll_fn(|cx| connection.poll_new_outbound(cx)).await
            })
            .map_err(|e| {
                VirgeError::TransportError(format!(
                    "Failed to open yamux outbound stream: {}",
                    e
                ))
            })?;
        self.yamux_stream = Some(stream);

        // 2. 将 connection 所有权移交给 driver task（无需 Arc<Mutex>）
        let handle = get_runtime().spawn(async move {
            debug!("Yamux connection driver started");
            loop {
                match poll_fn(|cx| connection.poll_next_inbound(cx)).await {
                    Some(Ok(_stream)) => {
                        // 客户端一般不会收到 inbound stream，忽略
                    }
                    Some(Err(e)) => {
                        warn!("Yamux connection error in driver: {}", e);
                        break;
                    }
                    None => {
                        info!("Yamux connection closed (driver)");
                        break;
                    }
                }
            }
            info!("Yamux connection driver stopped");
        });
        self.driver_handle = Some(handle);

        info!("Yamux transport connected successfully");
        Ok(())
    }

    /// 从已有的 tokio VsockStream 初始化（服务端模式）
    pub fn from_tokio_stream(&mut self, vsock_stream: VsockStream) -> Result<()> {
        let config = Config::default();
        let mut connection = Connection::new(vsock_stream.compat(), config, Mode::Server);
        self.mode = Mode::Server;

        // 1. 服务端通过 poll_next_inbound 等待客户端打开的 inbound stream
        let stream_result = get_runtime().block_on(async {
            poll_fn(|cx| connection.poll_next_inbound(cx)).await
        });

        match stream_result {
            Some(Ok(s)) => {
                self.yamux_stream = Some(s);
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

        // 2. 将 connection 所有权移交给 driver task 继续驱动
        let handle = get_runtime().spawn(async move {
            debug!("Yamux server connection driver started");
            loop {
                match poll_fn(|cx| connection.poll_next_inbound(cx)).await {
                    Some(Ok(_stream)) => {
                        // 当前只使用一个 stream，忽略后续 inbound
                    }
                    Some(Err(e)) => {
                        warn!("Yamux server connection error in driver: {}", e);
                        break;
                    }
                    None => {
                        info!("Yamux server connection closed (driver)");
                        break;
                    }
                }
            }
            info!("Yamux server connection driver stopped");
        });
        self.driver_handle = Some(handle);

        info!("Yamux transport initialized from stream (server mode)");
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        info!("Yamux transport disconnecting");

        // 关闭 yamux stream，这会通知对端 stream EOF
        if let Some(mut stream) = self.yamux_stream.take() {
            let _ = get_runtime().block_on(async {
                let _ = stream.close().await;
            });
        }

        // 等待 driver task 退出（stream 关闭后 connection 会自然结束）
        if let Some(handle) = self.driver_handle.take() {
            let _ = get_runtime().block_on(async {
                let _ = tokio::time::timeout(std::time::Duration::from_secs(2), handle).await;
            });
        }

        info!("Yamux transport disconnected");
        Ok(())
    }

    /// 发送数据（不会关闭 stream，可多次调用）
    pub fn send(&mut self, data: &[u8]) -> Result<usize> {
        let stream = self
            .yamux_stream
            .as_mut()
            .ok_or_else(|| VirgeError::TransportError("Yamux stream not available".into()))?;

        get_runtime().block_on(async {
            stream
                .write_all(data)
                .await
                .map_err(|e| VirgeError::Other(format!("yamux send error: {}", e)))?;
            stream
                .flush()
                .await
                .map_err(|e| VirgeError::Other(format!("yamux flush error: {}", e)))?;
            Ok::<_, VirgeError>(())
        })?;

        debug!("Yamux sent {} bytes", data.len());
        Ok(data.len())
    }

    /// 接收数据：读取一次可用的数据块（非阻塞到 EOF，而是读取当前可用数据）
    pub fn recv(&mut self) -> Result<Vec<u8>> {
        let stream = self
            .yamux_stream
            .as_mut()
            .ok_or_else(|| VirgeError::TransportError("Yamux stream not available".into()))?;

        let mut buf = vec![0u8; 64 * 1024]; // 64KB 接收缓冲区
        let n = get_runtime().block_on(async {
            stream
                .read(&mut buf)
                .await
                .map_err(|e| VirgeError::Other(format!("yamux recv error: {}", e)))
        })?;

        buf.truncate(n);
        debug!("Yamux received {} bytes", n);
        Ok(buf)
    }

    pub fn is_connected(&self) -> bool {
        self.yamux_stream.is_some()
    }
}
