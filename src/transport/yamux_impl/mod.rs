// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

//! Yamux 传输协议实现

mod transfer_handler;
pub use transfer_handler::get_runtime;
pub use transfer_handler::YamuxTransportHandler;
