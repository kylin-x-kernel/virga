// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

use crate::{
    config::{TransportConfig, HEADER_SIZE, MESSAGE_HEAD_SIZE},
    error::{Error, ErrorKind},
    io::{Read, Write},
    protocol::{Packet, PacketHeader, PacketType, MessageHead},
    Result,
};
use alloc::vec::Vec;

pub struct XTransport<T> {
    inner: T,
    send_seq: u32,
    recv_seq: u32,
    next_message_id: u64,
    recv_buffer: Vec<u8>,
    recv_pos: usize,
    recv_available: usize,
    config: TransportConfig,
}

impl<T: Read + Write> XTransport<T> {
    pub fn new(inner: T, config: TransportConfig) -> Self {
        XTransport {
            inner,
            send_seq: 0,
            recv_seq: 0,
            next_message_id: 1,
            recv_buffer: Vec::new(),
            recv_pos: 0,
            recv_available: 0,
            config,
        }
    }

    fn send_packet(&mut self, pkt_type: PacketType, data: &[u8]) -> Result<()> {
        let packet = Packet::new(pkt_type, self.send_seq, data.to_vec());
        let seq = packet.header.seq;
        self.send_seq = self.send_seq.wrapping_add(1);

        // Combine header and data into a single buffer for atomic send
        let header_bytes = packet.header.to_bytes();
        let mut combined = Vec::with_capacity(header_bytes.len() + packet.data.len());
        combined.extend_from_slice(&header_bytes);
        combined.extend_from_slice(&packet.data);
        
        // Send combined buffer in one write call
        self.inner.write_all(&combined)?;
        
        log::trace!("Sent packet type={:?}, seq={}, len={}", pkt_type, seq, packet.data.len());
        
        // Wait for ACK if configured and not sending an ACK itself
        if self.config.wait_for_ack && pkt_type != PacketType::Ack {
            let ack_packet = self.recv_packet_internal()?;
            if ack_packet.header.pkt_type != PacketType::Ack as u8 {
                return Err(Error::new(ErrorKind::InvalidPacket));
            }
            if ack_packet.data.len() < 4 {
                return Err(Error::new(ErrorKind::InvalidPacket));
            }
            let ack_seq = u32::from_le_bytes([ack_packet.data[0], ack_packet.data[1], ack_packet.data[2], ack_packet.data[3]]);
            if ack_seq != seq {
                log::warn!("ACK seq mismatch: expected {}, got {}", seq, ack_seq);
                return Err(Error::new(ErrorKind::InvalidPacket));
            }
            log::trace!("Received ACK for seq={}", seq);
        }
        
        Ok(())
    }

    fn send_ack(&mut self, seq: u32) -> Result<()> {
        let ack_data = seq.to_le_bytes();
        let ack_packet = Packet::new(PacketType::Ack, self.send_seq, ack_data.to_vec());
        self.send_seq = self.send_seq.wrapping_add(1);
        
        let header_bytes = ack_packet.header.to_bytes();
        let mut combined = Vec::with_capacity(header_bytes.len() + ack_packet.data.len());
        combined.extend_from_slice(&header_bytes);
        combined.extend_from_slice(&ack_packet.data);
        self.inner.write_all(&combined)?;
        
        log::trace!("Sent ACK for seq={}", seq);
        Ok(())
    }

    fn recv_packet_internal(&mut self) -> Result<Packet> {
        // Read header
        let mut header_buf = [0u8; HEADER_SIZE];
        self.inner.read_exact(&mut header_buf)?;
        let header = PacketHeader::from_bytes(&header_buf)?;

        // Read data
        let mut data = alloc::vec![0u8; header.length as usize];
        self.inner.read_exact(&mut data)?;

        let packet = Packet { header, data };

        // Verify CRC
        if !packet.verify_crc() {
            return Err(Error::new(ErrorKind::CrcMismatch));
        }

        log::trace!("Received packet seq={}, len={}", packet.header.seq, packet.data.len());

        Ok(packet)
    }

    fn recv_packet(&mut self) -> Result<Packet> {
        let packet = self.recv_packet_internal()?;
        
        // Send ACK if configured and not receiving an ACK itself
        let pkt_type = PacketType::from_u8(packet.header.pkt_type)
            .ok_or_else(|| Error::new(ErrorKind::InvalidPacket))?;
        
        if self.config.wait_for_ack && pkt_type != PacketType::Ack {
            self.send_ack(packet.header.seq)?;
        }
        
        // Update receive sequence
        self.recv_seq = packet.header.seq.wrapping_add(1);

        Ok(packet)
    }

    /// Send a complete message (automatically handles fragmentation)
    pub fn send_message(&mut self, data: &[u8]) -> Result<()> {
        if data.len() <= self.config.max_payload_size {
            // Small message: single Data packet
            self.send_packet(PacketType::Data, data)?;
            log::debug!("Sent single-packet message: {} bytes", data.len());
        } else {
            // Large message: MessageHead + multiple MessageData packets
            let message_id = self.next_message_id;
            self.next_message_id = self.next_message_id.wrapping_add(1);
            
            let packet_count = ((data.len() + self.config.max_payload_size - 1) / self.config.max_payload_size) as u32;
            
            // Send MessageHead
            let head = MessageHead::new(data.len() as u64, message_id, packet_count);
            self.send_packet(PacketType::MessageHead, &head.to_bytes())?;
            
            log::debug!("Sending large message: id={}, total={} bytes, packets={}", 
                       message_id, data.len(), packet_count);
            
            // Send MessageData packets
            for chunk in data.chunks(self.config.max_payload_size) {
                self.send_packet(PacketType::MessageData, chunk)?;
            }
            
            log::debug!("Large message sent: id={}", message_id);
        }
        
        self.inner.flush()?;
        Ok(())
    }

    /// Receive a complete message (automatically handles reassembly)
    pub fn recv_message(&mut self) -> Result<Vec<u8>> {
        // Read first packet to determine type
        let mut header_buf = [0u8; HEADER_SIZE];
        self.inner.read_exact(&mut header_buf)?;
        let header = PacketHeader::from_bytes(&header_buf)?;
        
        let pkt_type = PacketType::from_u8(header.pkt_type)
            .ok_or_else(|| Error::new(ErrorKind::InvalidPacket))?;
        
        match pkt_type {
            PacketType::Data => {
                // Single packet message
                let mut data = alloc::vec![0u8; header.length as usize];
                self.inner.read_exact(&mut data)?;
                
                let packet = Packet { header, data };
                if !packet.verify_crc() {
                    return Err(Error::new(ErrorKind::CrcMismatch));
                }
                
                // Send ACK if configured
                if self.config.wait_for_ack {
                    self.send_ack(packet.header.seq)?;
                }
                
                log::debug!("Received single-packet message: {} bytes", packet.data.len());
                Ok(packet.data)
            }
            PacketType::MessageHead => {
                // Multi-packet message
                let mut head_data = alloc::vec![0u8; header.length as usize];
                self.inner.read_exact(&mut head_data)?;
                
                let packet = Packet { header, data: head_data };
                if !packet.verify_crc() {
                    return Err(Error::new(ErrorKind::CrcMismatch));
                }
                
                // Send ACK for MessageHead if configured
                if self.config.wait_for_ack {
                    self.send_ack(packet.header.seq)?;
                }
                
                if packet.data.len() < MESSAGE_HEAD_SIZE {
                    return Err(Error::new(ErrorKind::InvalidPacket));
                }
                
                let mut head_bytes = [0u8; MESSAGE_HEAD_SIZE];
                head_bytes.copy_from_slice(&packet.data[..MESSAGE_HEAD_SIZE]);
                let msg_head = MessageHead::from_bytes(&head_bytes)?;
                
                log::debug!("Receiving large message: id={}, total={} bytes, packets={}", 
                           msg_head.message_id, msg_head.total_length, msg_head.packet_count);
                
                // Receive all data packets
                let mut result = alloc::vec![0u8; msg_head.total_length as usize];
                let mut offset = 0;
                
                for i in 0..msg_head.packet_count {
                    let mut data_header_buf = [0u8; HEADER_SIZE];
                    self.inner.read_exact(&mut data_header_buf)?;
                    let data_header = PacketHeader::from_bytes(&data_header_buf)?;
                    
                    let data_type = PacketType::from_u8(data_header.pkt_type)
                        .ok_or_else(|| Error::new(ErrorKind::InvalidPacket))?;
                    
                    if data_type != PacketType::MessageData {
                        return Err(Error::new(ErrorKind::InvalidPacket));
                    }
                    
                    let mut chunk = alloc::vec![0u8; data_header.length as usize];
                    self.inner.read_exact(&mut chunk)?;
                    
                    let data_packet = Packet { header: data_header, data: chunk };
                    if !data_packet.verify_crc() {
                        return Err(Error::new(ErrorKind::CrcMismatch));
                    }
                    
                    // Send ACK for each MessageData if configured
                    if self.config.wait_for_ack {
                        self.send_ack(data_packet.header.seq)?;
                    }
                    
                    let to_copy = core::cmp::min(data_packet.data.len(), result.len() - offset);
                    result[offset..offset + to_copy].copy_from_slice(&data_packet.data[..to_copy]);
                    offset += to_copy;
                    
                    if (i + 1) % 100 == 0 || i + 1 == msg_head.packet_count {
                        log::debug!("Progress: {}/{} packets received", i + 1, msg_head.packet_count);
                    }
                }
                
                log::debug!("Large message received: id={}, {} bytes", msg_head.message_id, result.len());
                Ok(result)
            }
            PacketType::MessageData | PacketType::Ack => {
                // Unexpected: should not receive MessageData or Ack as first packet
                Err(Error::new(ErrorKind::InvalidPacket))
            }
        }
    }
}

impl<T: Read + Write> Read for XTransport<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.recv_pos >= self.recv_available {
            // Need to receive a new packet
            let packet = self.recv_packet()?;
            self.recv_buffer = packet.data;
            self.recv_pos = 0;
            self.recv_available = self.recv_buffer.len();
        }

        // Copy data from receive buffer
        let to_copy = core::cmp::min(buf.len(), self.recv_available - self.recv_pos);
        buf[..to_copy].copy_from_slice(&self.recv_buffer[self.recv_pos..self.recv_pos + to_copy]);
        self.recv_pos += to_copy;

        Ok(to_copy)
    }
}

impl<T: Read + Write> Write for XTransport<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        // Send first chunk (up to max_payload_size)
        let to_send = core::cmp::min(buf.len(), self.config.max_payload_size);
        self.send_packet(PacketType::Data, &buf[..to_send])?;

        Ok(to_send)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::config::TransportConfig;
    use std::io::Cursor;

    /// Helper: send a message through one XTransport, then recv on another
    /// both backed by the same byte buffer.
    fn roundtrip_message(data: &[u8], max_frame_size: usize) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        // Sender side: write to buf
        {
            let cursor = Cursor::new(&mut buf);
            let config = TransportConfig::default().with_max_frame_size(max_frame_size);
            let mut sender = XTransport::new(cursor, config);
            sender.send_message(data).unwrap();
        }

        // Receiver side: read from buf
        {
            let cursor = Cursor::new(buf);
            let config = TransportConfig::default().with_max_frame_size(max_frame_size);
            let mut receiver = XTransport::new(cursor, config);
            receiver.recv_message().unwrap()
        }
    }

    #[test]
    fn send_recv_small_message() {
        let data = vec![1, 2, 3, 4, 5];
        let received = roundtrip_message(&data, 4096);
        assert_eq!(received, data);
    }

    #[test]
    fn send_recv_empty_message() {
        let data = vec![];
        let received = roundtrip_message(&data, 4096);
        assert_eq!(received, data);
    }

    #[test]
    fn send_recv_exact_payload_size() {
        let data = vec![0xAB; 1024];
        let received = roundtrip_message(&data, 1040);
        assert_eq!(received, data);
    }

    #[test]
    fn send_recv_large_message_fragmented() {
        let data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
        let received = roundtrip_message(&data, 1024);
        assert_eq!(received, data);
    }

    #[test]
    fn send_recv_large_message_exact_multiple() {
        let payload_size = 100 - 16;
        let data = vec![0x55; payload_size * 10];
        let received = roundtrip_message(&data, 100);
        assert_eq!(received, data);
    }

    #[test]
    fn send_recv_large_message_not_exact_multiple() {
        let payload_size = 100 - 16;
        let data = vec![0x55; payload_size * 10 + 1];
        let received = roundtrip_message(&data, 100);
        assert_eq!(received, data);
    }

    #[test]
    fn send_recv_one_byte() {
        let data = vec![42];
        let received = roundtrip_message(&data, 4096);
        assert_eq!(received, data);
    }

    #[test]
    fn send_recv_multiple_messages_sequential() {
        let mut buf: Vec<u8> = Vec::new();
        let messages = vec![
            vec![1, 2, 3],
            vec![4, 5, 6, 7, 8],
            vec![9],
        ];

        {
            let cursor = Cursor::new(&mut buf);
            let config = TransportConfig::default().with_max_frame_size(4096);
            let mut sender = XTransport::new(cursor, config);
            for msg in &messages {
                sender.send_message(msg).unwrap();
            }
        }

        {
            let cursor = Cursor::new(buf);
            let config = TransportConfig::default().with_max_frame_size(4096);
            let mut receiver = XTransport::new(cursor, config);
            for expected in &messages {
                let received = receiver.recv_message().unwrap();
                assert_eq!(&received, expected);
            }
        }
    }

    #[test]
    fn send_recv_with_ack_mode() {
        let (c2s_reader, c2s_writer) = std::io::pipe().unwrap();
        let (s2c_reader, s2c_writer) = std::io::pipe().unwrap();

        let data = vec![0xAB; 512];
        let data_clone = data.clone();

        let sender_handle = std::thread::spawn(move || {
            let duplex = DuplexStream {
                reader: s2c_reader,
                writer: c2s_writer,
            };
            let config = TransportConfig::default()
                .with_max_frame_size(4096)
                .with_ack(true);
            let mut sender = XTransport::new(duplex, config);
            sender.send_message(&data_clone).unwrap();
        });

        let duplex = DuplexStream {
            reader: c2s_reader,
            writer: s2c_writer,
        };
        let config = TransportConfig::default()
            .with_max_frame_size(4096)
            .with_ack(true);
        let mut receiver = XTransport::new(duplex, config);
        let received = receiver.recv_message().unwrap();

        sender_handle.join().unwrap();
        assert_eq!(received, data);
    }

    #[test]
    fn send_recv_large_with_ack_mode() {
        let (c2s_reader, c2s_writer) = std::io::pipe().unwrap();
        let (s2c_reader, s2c_writer) = std::io::pipe().unwrap();

        let data: Vec<u8> = (0..3000).map(|i| (i % 256) as u8).collect();
        let data_clone = data.clone();

        let sender_handle = std::thread::spawn(move || {
            let duplex = DuplexStream {
                reader: s2c_reader,
                writer: c2s_writer,
            };
            let config = TransportConfig::default()
                .with_max_frame_size(256)
                .with_ack(true);
            let mut sender = XTransport::new(duplex, config);
            sender.send_message(&data_clone).unwrap();
        });

        let duplex = DuplexStream {
            reader: c2s_reader,
            writer: s2c_writer,
        };
        let config = TransportConfig::default()
            .with_max_frame_size(256)
            .with_ack(true);
        let mut receiver = XTransport::new(duplex, config);
        let received = receiver.recv_message().unwrap();

        sender_handle.join().unwrap();
        assert_eq!(received, data);
    }

    #[test]
    fn recv_message_truncated_header() {
        let buf = vec![0u8; 8];
        let cursor = Cursor::new(buf);
        let config = TransportConfig::default();
        let mut receiver = XTransport::new(cursor, config);
        let result = receiver.recv_message();
        assert!(result.is_err());
    }

    #[test]
    fn recv_message_invalid_magic() {
        let mut buf = vec![0u8; 32];
        buf[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
        let cursor = Cursor::new(buf);
        let config = TransportConfig::default();
        let mut receiver = XTransport::new(cursor, config);
        let result = receiver.recv_message();
        assert!(result.is_err());
    }

    #[test]
    fn transport_read_write_trait() {
        let mut buf: Vec<u8> = Vec::new();
        let write_data = vec![10, 20, 30, 40, 50];

        {
            let cursor = Cursor::new(&mut buf);
            let config = TransportConfig::default().with_max_frame_size(4096);
            let mut transport = XTransport::new(cursor, config);
            let written = Write::write(&mut transport, &write_data).unwrap();
            assert_eq!(written, write_data.len());
            Write::flush(&mut transport).unwrap();
        }

        {
            let cursor = Cursor::new(buf);
            let config = TransportConfig::default().with_max_frame_size(4096);
            let mut transport = XTransport::new(cursor, config);
            let mut read_buf = vec![0u8; 5];
            let n = Read::read(&mut transport, &mut read_buf).unwrap();
            assert_eq!(n, 5);
            assert_eq!(&read_buf[..n], &write_data);
        }
    }

    #[test]
    fn transport_write_empty() {
        let mut buf: Vec<u8> = Vec::new();
        let cursor = Cursor::new(&mut buf);
        let config = TransportConfig::default();
        let mut transport = XTransport::new(cursor, config);
        let written = Write::write(&mut transport, &[]).unwrap();
        assert_eq!(written, 0);
    }

    #[test]
    fn transport_write_large_chunk_truncated() {
        let mut buf: Vec<u8> = Vec::new();
        let config = TransportConfig::default().with_max_frame_size(100);
        let cursor = Cursor::new(&mut buf);
        let mut transport = XTransport::new(cursor, config);

        let data = vec![0xCC; 200];
        let written = Write::write(&mut transport, &data).unwrap();
        assert_eq!(written, 84);
    }

    // Helper: duplex stream combining separate reader and writer.
    struct DuplexStream<R, W> {
        reader: R,
        writer: W,
    }

    impl<R: std::io::Read, W: std::io::Write> std::io::Read for DuplexStream<R, W> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.reader.read(buf)
        }
    }

    impl<R: std::io::Read, W: std::io::Write> std::io::Write for DuplexStream<R, W> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.writer.write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.writer.flush()
        }
    }

    /// Helper: build a raw packet into bytes (header + data)
    fn build_raw_packet(pkt_type: PacketType, seq: u32, data: &[u8]) -> Vec<u8> {
        let packet = Packet::new(pkt_type, seq, data.to_vec());
        let header_bytes = packet.header.to_bytes();
        let mut buf = Vec::with_capacity(header_bytes.len() + packet.data.len());
        buf.extend_from_slice(&header_bytes);
        buf.extend_from_slice(&packet.data);
        buf
    }

    /// Helper: build a raw packet with corrupted CRC
    fn build_corrupted_packet(pkt_type: PacketType, seq: u32, data: &[u8]) -> Vec<u8> {
        let mut packet = Packet::new(pkt_type, seq, data.to_vec());
        packet.header.crc32 ^= 0xFFFFFFFF; // corrupt CRC
        let header_bytes = packet.header.to_bytes();
        let mut buf = Vec::with_capacity(header_bytes.len() + packet.data.len());
        buf.extend_from_slice(&header_bytes);
        buf.extend_from_slice(&packet.data);
        buf
    }

    #[test]
    fn recv_message_corrupted_crc_data_packet() {
        let buf = build_corrupted_packet(PacketType::Data, 0, &[1, 2, 3]);
        let cursor = Cursor::new(buf);
        let config = TransportConfig::default();
        let mut receiver = XTransport::new(cursor, config);
        let result = receiver.recv_message();
        assert!(result.is_err());
    }

    #[test]
    fn recv_message_unexpected_message_data_type() {
        // Sending MessageData as first packet should fail
        let buf = build_raw_packet(PacketType::MessageData, 0, &[1, 2, 3]);
        let cursor = Cursor::new(buf);
        let config = TransportConfig::default();
        let mut receiver = XTransport::new(cursor, config);
        let result = receiver.recv_message();
        assert!(result.is_err());
    }

    #[test]
    fn recv_message_unexpected_ack_type() {
        // Sending Ack as first packet should fail
        let ack_data = 0u32.to_le_bytes().to_vec();
        let buf = build_raw_packet(PacketType::Ack, 0, &ack_data);
        let cursor = Cursor::new(buf);
        let config = TransportConfig::default();
        let mut receiver = XTransport::new(cursor, config);
        let result = receiver.recv_message();
        assert!(result.is_err());
    }

    #[test]
    fn recv_message_corrupted_crc_message_head() {
        // Build a corrupted MessageHead packet
        let head = MessageHead::new(100, 1, 2);
        let buf = build_corrupted_packet(PacketType::MessageHead, 0, &head.to_bytes());
        let cursor = Cursor::new(buf);
        let config = TransportConfig::default();
        let mut receiver = XTransport::new(cursor, config);
        let result = receiver.recv_message();
        assert!(result.is_err());
    }

    #[test]
    fn recv_message_too_small_message_head() {
        // MessageHead with data smaller than MESSAGE_HEAD_SIZE
        let small_data = vec![0u8; 4]; // too small for MessageHead (needs 32 bytes)
        let buf = build_raw_packet(PacketType::MessageHead, 0, &small_data);
        let cursor = Cursor::new(buf);
        let config = TransportConfig::default();
        let mut receiver = XTransport::new(cursor, config);
        let result = receiver.recv_message();
        assert!(result.is_err());
    }

    #[test]
    fn recv_message_wrong_data_packet_type_in_sequence() {
        // Valid MessageHead followed by a Data packet instead of MessageData
        let head = MessageHead::new(5, 1, 1);
        let mut buf = build_raw_packet(PacketType::MessageHead, 0, &head.to_bytes());
        // Append a Data packet (wrong type, should be MessageData)
        buf.extend_from_slice(&build_raw_packet(PacketType::Data, 1, &[1, 2, 3, 4, 5]));

        let cursor = Cursor::new(buf);
        let config = TransportConfig::default();
        let mut receiver = XTransport::new(cursor, config);
        let result = receiver.recv_message();
        assert!(result.is_err());
    }

    #[test]
    fn recv_message_corrupted_crc_in_data_sequence() {
        // Valid MessageHead followed by corrupted MessageData
        let head = MessageHead::new(5, 1, 1);
        let mut buf = build_raw_packet(PacketType::MessageHead, 0, &head.to_bytes());
        buf.extend_from_slice(&build_corrupted_packet(PacketType::MessageData, 1, &[1, 2, 3, 4, 5]));

        let cursor = Cursor::new(buf);
        let config = TransportConfig::default();
        let mut receiver = XTransport::new(cursor, config);
        let result = receiver.recv_message();
        assert!(result.is_err());
    }

    #[test]
    fn recv_packet_internal_crc_mismatch() {
        // Build a corrupted raw packet and try to receive via recv_packet
        let mut packet = Packet::new(PacketType::Data, 0, vec![1, 2, 3]);
        packet.header.crc32 ^= 1; // corrupt CRC
        let header_bytes = packet.header.to_bytes();
        let mut buf = Vec::new();
        buf.extend_from_slice(&header_bytes);
        buf.extend_from_slice(&packet.data);

        let cursor = Cursor::new(buf);
        let config = TransportConfig::default();
        let mut transport = XTransport::new(cursor, config);
        // Use crate::io::Read trait which internally calls recv_packet -> recv_packet_internal
        let mut read_buf = [0u8; 10];
        let result = crate::io::Read::read(&mut transport, &mut read_buf);
        assert!(result.is_err());
    }

    #[test]
    fn recv_message_with_invalid_packet_type_byte() {
        // Construct a header with invalid packet type (e.g., 255)
        let mut header = PacketHeader::new(PacketType::Data, 0, 3);
        header.pkt_type = 255; // invalid type
        // Compute CRC for the data
        let data = vec![1, 2, 3];
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&data);
        header.crc32 = hasher.finalize();

        let header_bytes = header.to_bytes();
        let mut buf = Vec::new();
        buf.extend_from_slice(&header_bytes);
        buf.extend_from_slice(&data);

        let cursor = Cursor::new(buf);
        let config = TransportConfig::default();
        let mut receiver = XTransport::new(cursor, config);
        let result = receiver.recv_message();
        assert!(result.is_err());
    }
}
