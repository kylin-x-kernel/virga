// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

// Protocol constants
pub const MAGIC: u32 = 0x58545250; // "XTRP"
pub const VERSION: u8 = 0x01;
pub const HEADER_SIZE: usize = 16;
pub const MESSAGE_HEAD_SIZE: usize = 32;
const DEFAULT_MAX_FRAME_SIZE: usize = 4096; // 4KB

pub struct TransportConfig {
    pub max_payload_size: usize,
    pub wait_for_ack: bool,
}

impl TransportConfig {
    pub fn new() -> Self {
        Self {
            max_payload_size: DEFAULT_MAX_FRAME_SIZE - HEADER_SIZE,
            wait_for_ack: false,
        }
    }

    pub fn with_max_frame_size(mut self, frame_size: usize) -> Self {
        self.max_payload_size = frame_size.saturating_sub(HEADER_SIZE);
        self
    }

    pub fn with_ack(mut self, wait_for_ack: bool) -> Self {
        self.wait_for_ack = wait_for_ack;
        self
    }
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::xtransport::config::{HEADER_SIZE, MAGIC, MESSAGE_HEAD_SIZE, VERSION};

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(MAGIC, 0x58545250);
        assert_eq!(VERSION, 0x01);
        assert_eq!(HEADER_SIZE, 16);
        assert_eq!(MESSAGE_HEAD_SIZE, 32);
    }

    #[test]
    fn transport_config_new_defaults() {
        let config = TransportConfig::new();
        assert_eq!(config.max_payload_size, 4096 - HEADER_SIZE);
        assert!(!config.wait_for_ack);
    }

    #[test]
    fn transport_config_default_trait() {
        let config = TransportConfig::default();
        assert_eq!(config.max_payload_size, 4096 - HEADER_SIZE);
        assert!(!config.wait_for_ack);
    }

    #[test]
    fn transport_config_with_max_frame_size() {
        let config = TransportConfig::new().with_max_frame_size(1024);
        assert_eq!(config.max_payload_size, 1024 - HEADER_SIZE);
    }

    #[test]
    fn transport_config_with_max_frame_size_small() {
        let config = TransportConfig::new().with_max_frame_size(10);
        assert_eq!(config.max_payload_size, 0);
    }

    #[test]
    fn transport_config_with_max_frame_size_zero() {
        let config = TransportConfig::new().with_max_frame_size(0);
        assert_eq!(config.max_payload_size, 0);
    }

    #[test]
    fn transport_config_with_max_frame_size_exact_header() {
        let config = TransportConfig::new().with_max_frame_size(HEADER_SIZE);
        assert_eq!(config.max_payload_size, 0);
    }

    #[test]
    fn transport_config_with_ack_true() {
        let config = TransportConfig::new().with_ack(true);
        assert!(config.wait_for_ack);
    }

    #[test]
    fn transport_config_with_ack_false() {
        let config = TransportConfig::new().with_ack(false);
        assert!(!config.wait_for_ack);
    }

    #[test]
    fn transport_config_builder_chain() {
        let config = TransportConfig::new()
            .with_max_frame_size(2048)
            .with_ack(true);
        assert_eq!(config.max_payload_size, 2048 - HEADER_SIZE);
        assert!(config.wait_for_ack);
    }

    #[test]
    fn transport_config_with_large_frame_size() {
        let config = TransportConfig::new().with_max_frame_size(1024 * 1024);
        assert_eq!(config.max_payload_size, 1024 * 1024 - HEADER_SIZE);
    }
}
