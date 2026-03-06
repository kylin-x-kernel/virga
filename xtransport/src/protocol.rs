// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

use crate::{Error, error::ErrorKind, Result};
use crate::config::{MAGIC, VERSION, HEADER_SIZE, MESSAGE_HEAD_SIZE};
use alloc::vec::Vec;
use crc32fast::Hasher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    Data = 0,          // Single packet message
    MessageHead = 1,   // Multi-packet message header
    MessageData = 2,   // Multi-packet message data
    Ack = 3,           // Acknowledgment packet
}

impl PacketType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(PacketType::Data),
            1 => Some(PacketType::MessageHead),
            2 => Some(PacketType::MessageData),
            3 => Some(PacketType::Ack),
            _ => None,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct PacketHeader {
    pub magic: u32,      // 4 bytes
    pub version: u8,     // 1 byte
    pub pkt_type: u8,    // 1 byte - Packet type
    pub seq: u32,        // 4 bytes
    pub length: u16,     // 2 bytes
    pub crc32: u32,      // 4 bytes
}

impl PacketHeader {
    pub fn new(pkt_type: PacketType, seq: u32, length: u16) -> Self {
        PacketHeader {
            magic: MAGIC,
            version: VERSION,
            pkt_type: pkt_type as u8,
            seq,
            length,
            crc32: 0,
        }
    }

    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic.to_le_bytes());
        buf[4] = self.version;
        buf[5] = self.pkt_type;
        buf[6..10].copy_from_slice(&self.seq.to_le_bytes());
        buf[10..12].copy_from_slice(&self.length.to_le_bytes());
        buf[12..16].copy_from_slice(&self.crc32.to_le_bytes());
        buf
    }

    pub fn from_bytes(buf: &[u8; HEADER_SIZE]) -> Result<Self> {
        let magic = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if magic != MAGIC {
            return Err(Error::new(ErrorKind::InvalidMagic));
        }

        let version = buf[4];
        if version != VERSION {
            return Err(Error::new(ErrorKind::InvalidVersion));
        }

        let pkt_type = buf[5];
        let seq = u32::from_le_bytes([buf[6], buf[7], buf[8], buf[9]]);
        let length = u16::from_le_bytes([buf[10], buf[11]]);
        let crc32 = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);

        Ok(PacketHeader {
            magic,
            version,
            pkt_type,
            seq,
            length,
            crc32,
        })
    }
}

#[repr(C)]
pub struct MessageHead {
    pub total_length: u64,   // 8 bytes - Total message length
    pub message_id: u64,     // 8 bytes - Unique message ID
    pub packet_count: u32,   // 4 bytes - Total packet count
    pub flags: u32,          // 4 bytes - Message flags
    pub reserved: [u8; 8],   // 8 bytes - Reserved for extension
}

impl MessageHead {
    pub fn new(total_length: u64, message_id: u64, packet_count: u32) -> Self {
        MessageHead {
            total_length,
            message_id,
            packet_count,
            flags: 0,
            reserved: [0; 8],
        }
    }

    pub fn to_bytes(&self) -> [u8; MESSAGE_HEAD_SIZE] {
        let mut buf = [0u8; MESSAGE_HEAD_SIZE];
        buf[0..8].copy_from_slice(&self.total_length.to_le_bytes());
        buf[8..16].copy_from_slice(&self.message_id.to_le_bytes());
        buf[16..20].copy_from_slice(&self.packet_count.to_le_bytes());
        buf[20..24].copy_from_slice(&self.flags.to_le_bytes());
        buf[24..32].copy_from_slice(&self.reserved);
        buf
    }

    pub fn from_bytes(buf: &[u8; MESSAGE_HEAD_SIZE]) -> Result<Self> {
        let total_length = u64::from_le_bytes([
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ]);
        let message_id = u64::from_le_bytes([
            buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
        ]);
        let packet_count = u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
        let flags = u32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
        let mut reserved = [0u8; 8];
        reserved.copy_from_slice(&buf[24..32]);

        Ok(MessageHead {
            total_length,
            message_id,
            packet_count,
            flags,
            reserved,
        })
    }
}

pub struct Packet {
    pub header: PacketHeader,
    pub data: Vec<u8>,
}

impl Packet {
    pub fn new(pkt_type: PacketType, seq: u32, data: Vec<u8>) -> Self {
        let length = data.len() as u16;
        let mut header = PacketHeader::new(pkt_type, seq, length);
        
        // Calculate CRC32
        let mut hasher = Hasher::new();
        hasher.update(&data);
        header.crc32 = hasher.finalize();

        Packet { header, data }
    }

    pub fn verify_crc(&self) -> bool {
        let mut hasher = Hasher::new();
        hasher.update(&self.data);
        let computed_crc = hasher.finalize();
        computed_crc == self.header.crc32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{HEADER_SIZE, MAGIC, MESSAGE_HEAD_SIZE, VERSION};
    use crate::error::ErrorKind;
    use alloc::vec;
    
    // ==================== PacketType tests ====================

    #[test]
    fn packet_type_from_u8_valid() {
        assert_eq!(PacketType::from_u8(0), Some(PacketType::Data));
        assert_eq!(PacketType::from_u8(1), Some(PacketType::MessageHead));
        assert_eq!(PacketType::from_u8(2), Some(PacketType::MessageData));
        assert_eq!(PacketType::from_u8(3), Some(PacketType::Ack));
    }

    #[test]
    fn packet_type_from_u8_invalid() {
        assert_eq!(PacketType::from_u8(4), None);
        assert_eq!(PacketType::from_u8(255), None);
        assert_eq!(PacketType::from_u8(128), None);
    }

    #[test]
    fn packet_type_as_u8_roundtrip() {
        for val in 0..=3u8 {
            let pt = PacketType::from_u8(val).unwrap();
            assert_eq!(pt as u8, val);
        }
    }

    #[test]
    fn packet_type_equality() {
        assert_eq!(PacketType::Data, PacketType::Data);
        assert_ne!(PacketType::Data, PacketType::Ack);
    }

    // ==================== PacketHeader tests ====================

    #[test]
    fn packet_header_new_sets_fields() {
        let header = PacketHeader::new(PacketType::Data, 42, 100);
        assert_eq!(header.magic, MAGIC);
        assert_eq!(header.version, VERSION);
        assert_eq!(header.pkt_type, PacketType::Data as u8);
        assert_eq!(header.seq, 42);
        assert_eq!(header.length, 100);
        assert_eq!(header.crc32, 0);
    }

    #[test]
    fn packet_header_to_bytes_from_bytes_roundtrip() {
        let header = PacketHeader::new(PacketType::MessageHead, 12345, 256);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), HEADER_SIZE);

        let restored = PacketHeader::from_bytes(&bytes).unwrap();
        assert_eq!(restored.magic, MAGIC);
        assert_eq!(restored.version, VERSION);
        assert_eq!(restored.pkt_type, PacketType::MessageHead as u8);
        assert_eq!(restored.seq, 12345);
        assert_eq!(restored.length, 256);
        assert_eq!(restored.crc32, 0);
    }

    #[test]
    fn packet_header_to_bytes_from_bytes_all_types() {
        let types = [
            PacketType::Data,
            PacketType::MessageHead,
            PacketType::MessageData,
            PacketType::Ack,
        ];
        for pt in types {
            let header = PacketHeader::new(pt, 999, 50);
            let bytes = header.to_bytes();
            let restored = PacketHeader::from_bytes(&bytes).unwrap();
            assert_eq!(restored.pkt_type, pt as u8);
        }
    }

    #[test]
    fn packet_header_from_bytes_invalid_magic() {
        let mut bytes = PacketHeader::new(PacketType::Data, 0, 0).to_bytes();
        bytes[0] = 0xFF;
        let err = PacketHeader::from_bytes(&bytes).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidMagic);
    }

    #[test]
    fn packet_header_from_bytes_invalid_version() {
        let mut bytes = PacketHeader::new(PacketType::Data, 0, 0).to_bytes();
        bytes[4] = 0xFF;
        let err = PacketHeader::from_bytes(&bytes).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidVersion);
    }

    #[test]
    fn packet_header_with_max_seq() {
        let header = PacketHeader::new(PacketType::Data, u32::MAX, 0);
        let bytes = header.to_bytes();
        let restored = PacketHeader::from_bytes(&bytes).unwrap();
        assert_eq!(restored.seq, u32::MAX);
    }

    #[test]
    fn packet_header_with_max_length() {
        let header = PacketHeader::new(PacketType::Data, 0, u16::MAX);
        let bytes = header.to_bytes();
        let restored = PacketHeader::from_bytes(&bytes).unwrap();
        assert_eq!(restored.length, u16::MAX);
    }

    #[test]
    fn packet_header_preserves_crc32() {
        let mut header = PacketHeader::new(PacketType::Data, 0, 0);
        header.crc32 = 0xDEADBEEF;
        let bytes = header.to_bytes();
        let restored = PacketHeader::from_bytes(&bytes).unwrap();
        assert_eq!(restored.crc32, 0xDEADBEEF);
    }

    // ==================== MessageHead tests ====================

    #[test]
    fn message_head_new_sets_fields() {
        let head = MessageHead::new(1024, 42, 10);
        assert_eq!(head.total_length, 1024);
        assert_eq!(head.message_id, 42);
        assert_eq!(head.packet_count, 10);
        assert_eq!(head.flags, 0);
        assert_eq!(head.reserved, [0u8; 8]);
    }

    #[test]
    fn message_head_to_bytes_from_bytes_roundtrip() {
        let head = MessageHead::new(65536, 99, 100);
        let bytes = head.to_bytes();
        assert_eq!(bytes.len(), MESSAGE_HEAD_SIZE);

        let restored = MessageHead::from_bytes(&bytes).unwrap();
        assert_eq!(restored.total_length, 65536);
        assert_eq!(restored.message_id, 99);
        assert_eq!(restored.packet_count, 100);
        assert_eq!(restored.flags, 0);
        assert_eq!(restored.reserved, [0u8; 8]);
    }

    #[test]
    fn message_head_max_values() {
        let head = MessageHead::new(u64::MAX, u64::MAX, u32::MAX);
        let bytes = head.to_bytes();
        let restored = MessageHead::from_bytes(&bytes).unwrap();
        assert_eq!(restored.total_length, u64::MAX);
        assert_eq!(restored.message_id, u64::MAX);
        assert_eq!(restored.packet_count, u32::MAX);
    }

    #[test]
    fn message_head_zero_values() {
        let head = MessageHead::new(0, 0, 0);
        let bytes = head.to_bytes();
        let restored = MessageHead::from_bytes(&bytes).unwrap();
        assert_eq!(restored.total_length, 0);
        assert_eq!(restored.message_id, 0);
        assert_eq!(restored.packet_count, 0);
    }

    // ==================== Packet tests ====================

    #[test]
    fn packet_new_computes_crc() {
        let data = vec![1, 2, 3, 4, 5];
        let packet = Packet::new(PacketType::Data, 0, data.clone());
        assert_eq!(packet.header.pkt_type, PacketType::Data as u8);
        assert_eq!(packet.header.length, 5);
        assert_eq!(packet.data, data);
        assert_ne!(packet.header.crc32, 0);
    }

    #[test]
    fn packet_verify_crc_valid() {
        let data = vec![10, 20, 30, 40, 50];
        let packet = Packet::new(PacketType::Data, 1, data);
        assert!(packet.verify_crc());
    }

    #[test]
    fn packet_verify_crc_corrupted_data() {
        let data = vec![10, 20, 30, 40, 50];
        let mut packet = Packet::new(PacketType::Data, 1, data);
        packet.data[0] = 0xFF;
        assert!(!packet.verify_crc());
    }

    #[test]
    fn packet_verify_crc_corrupted_crc() {
        let data = vec![10, 20, 30];
        let mut packet = Packet::new(PacketType::Data, 1, data);
        packet.header.crc32 ^= 0xFFFFFFFF;
        assert!(!packet.verify_crc());
    }

    #[test]
    fn packet_empty_data() {
        let packet = Packet::new(PacketType::Data, 0, vec![]);
        assert_eq!(packet.header.length, 0);
        assert!(packet.verify_crc());
    }

    #[test]
    fn packet_large_data() {
        let data = vec![0xAB; 4096];
        let packet = Packet::new(PacketType::MessageData, 100, data.clone());
        assert_eq!(packet.header.length, 4096);
        assert_eq!(packet.data.len(), 4096);
        assert!(packet.verify_crc());
    }

    #[test]
    fn packet_different_data_different_crc() {
        let p1 = Packet::new(PacketType::Data, 0, vec![1, 2, 3]);
        let p2 = Packet::new(PacketType::Data, 0, vec![4, 5, 6]);
        assert_ne!(p1.header.crc32, p2.header.crc32);
    }

    #[test]
    fn packet_same_data_same_crc() {
        let data = vec![1, 2, 3, 4, 5];
        let p1 = Packet::new(PacketType::Data, 0, data.clone());
        let p2 = Packet::new(PacketType::Data, 99, data);
        assert_eq!(p1.header.crc32, p2.header.crc32);
    }

    #[test]
    fn packet_ack_type() {
        let ack_data = 42u32.to_le_bytes().to_vec();
        let packet = Packet::new(PacketType::Ack, 5, ack_data.clone());
        assert_eq!(packet.header.pkt_type, PacketType::Ack as u8);
        assert!(packet.verify_crc());
        assert_eq!(packet.data, ack_data);
    }
}
