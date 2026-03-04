// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

//! XTransport 传输协议实现
//!
//! 基于 xtransport 库的传输实现。
//!
//! # 特点
//! - 针对 vsock 优化的传输协议
//! - 轻量级设计

mod transfer_handler;

pub use transfer_handler::XTransportHandler;
