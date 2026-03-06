// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod config;
pub mod error;
pub mod io;
pub mod protocol;
pub mod transport;

pub use config::{HEADER_SIZE, MAGIC, MESSAGE_HEAD_SIZE, TransportConfig, VERSION};
pub use error::{Error, Result};
pub use io::{Read, Write};
pub use transport::XTransport;
