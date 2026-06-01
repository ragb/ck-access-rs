//! Yamaha CK SysEx framing.
//!
//! The CK uses Yamaha's address-based parameter protocol with a 3-byte address
//! and a fixed header `Group Number = 7F 1C`, `Model ID = 0B`. The high nibble
//! of the device byte selects the message type:
//!
//! ```text
//! Parameter Change   F0 43 1n 7F 1C 0B  ah am al  dd..dd       F7
//! Bulk Dump          F0 43 0n 7F 1C bh bl 0B  ah am al dd..dd cc  F7
//! Bulk Request       F0 43 2n 7F 1C 0B  ah am al               F7
//! Parameter Request  F0 43 3n 7F 1C 0B  ah am al               F7
//! ```
//!
//! `n` = device number (0..=15). `bh bl` = 14-bit Byte Count (= data length).
//! `cc` = checksum. Source: *CK88/CK61 Owner's Manual*, "MIDI Data Format"
//! §3-4 and "MIDI Data Table". Not yet round-trip-verified against a hardware
//! capture — see `docs/sysex-notes.md`.

use thiserror::Error;

pub const SYSEX_START: u8 = 0xF0;
pub const SYSEX_END: u8 = 0xF7;
/// Yamaha MIDI manufacturer ID.
pub const YAMAHA_ID: u8 = 0x43;
/// Group number that precedes the model ID (`Group High`, `Group Low`).
pub const GROUP: [u8; 2] = [0x7F, 0x1C];
/// Model ID shared by the CK61 and CK88.
pub const MODEL_ID: u8 = 0x0B;

/// Device-byte high nibbles, one per message type.
const KIND_BULK_DUMP: u8 = 0x00;
const KIND_PARAMETER_CHANGE: u8 = 0x10;
const KIND_BULK_REQUEST: u8 = 0x20;
const KIND_PARAMETER_REQUEST: u8 = 0x30;

/// Identity-reply device family code for the CK61 (`62 06`) and CK88 (`63 06`).
pub const FAMILY_CK61: [u8; 2] = [0x62, 0x06];
pub const FAMILY_CK88: [u8; 2] = [0x63, 0x06];

/// A decoded CK SysEx message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// `1n` — set the parameter(s) at `address` to `data`.
    ParameterChange {
        device: u8,
        address: [u8; 3],
        data: Vec<u8>,
    },
    /// `0n` — a block dump: `data` is the whole area, guarded by a checksum.
    BulkDump {
        device: u8,
        address: [u8; 3],
        data: Vec<u8>,
    },
    /// `3n` — ask the device to send back the parameter(s) at `address`.
    ParameterRequest { device: u8, address: [u8; 3] },
    /// `2n` — ask the device to send back the block at `address` as a Bulk Dump.
    BulkRequest { device: u8, address: [u8; 3] },
}

impl Message {
    /// Device number (0..=15) this message is addressed to / from.
    pub fn device(&self) -> u8 {
        match self {
            Message::ParameterChange { device, .. }
            | Message::BulkDump { device, .. }
            | Message::ParameterRequest { device, .. }
            | Message::BulkRequest { device, .. } => *device,
        }
    }

    /// 3-byte parameter address.
    pub fn address(&self) -> [u8; 3] {
        match self {
            Message::ParameterChange { address, .. }
            | Message::BulkDump { address, .. }
            | Message::ParameterRequest { address, .. }
            | Message::BulkRequest { address, .. } => *address,
        }
    }

    /// Serialize to wire bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![SYSEX_START, YAMAHA_ID];
        match self {
            Message::ParameterChange {
                device,
                address,
                data,
            } => {
                out.push(KIND_PARAMETER_CHANGE | (device & 0x0F));
                out.extend_from_slice(&GROUP);
                out.push(MODEL_ID);
                out.extend_from_slice(address);
                out.extend_from_slice(data);
            }
            Message::BulkDump {
                device,
                address,
                data,
            } => {
                out.push(KIND_BULK_DUMP | (device & 0x0F));
                out.extend_from_slice(&GROUP);
                // Byte Count = Model ID + Address (4 bytes) + Data — verified
                // against device captures (docs/sysex-notes.md §1).
                let byte_count = (data.len() + 4) as u16;
                let (bh, bl) = crate::codec::write_u14(byte_count);
                out.push(bh);
                out.push(bl);
                out.push(MODEL_ID);
                out.extend_from_slice(address);
                out.extend_from_slice(data);
                // Checksum covers Model ID + Address + Data (not the byte count) —
                // device-verified.
                let mut sum_region = vec![MODEL_ID];
                sum_region.extend_from_slice(address);
                sum_region.extend_from_slice(data);
                out.push(checksum(&sum_region));
            }
            Message::ParameterRequest { device, address } => {
                out.push(KIND_PARAMETER_REQUEST | (device & 0x0F));
                out.extend_from_slice(&GROUP);
                out.push(MODEL_ID);
                out.extend_from_slice(address);
            }
            Message::BulkRequest { device, address } => {
                out.push(KIND_BULK_REQUEST | (device & 0x0F));
                out.extend_from_slice(&GROUP);
                out.push(MODEL_ID);
                out.extend_from_slice(address);
            }
        }
        out.push(SYSEX_END);
        out
    }

    /// Parse wire bytes into a [`Message`].
    pub fn decode(bytes: &[u8]) -> Result<Self, SysExError> {
        if bytes.len() < 10 {
            return Err(SysExError::TooShort(bytes.len()));
        }
        if bytes[0] != SYSEX_START {
            return Err(SysExError::MissingStart);
        }
        if *bytes.last().unwrap() != SYSEX_END {
            return Err(SysExError::MissingEnd);
        }
        if bytes[1] != YAMAHA_ID {
            return Err(SysExError::NotYamaha(bytes[1]));
        }
        if bytes[3] != GROUP[0] || bytes[4] != GROUP[1] {
            return Err(SysExError::BadGroup([bytes[3], bytes[4]]));
        }
        let device = bytes[2] & 0x0F;
        let kind = bytes[2] & 0xF0;
        let body = &bytes[..bytes.len() - 1]; // drop F7

        match kind {
            KIND_PARAMETER_CHANGE => {
                // F0 43 1n 7F 1C 0B ah am al data...
                check_model(body, 5)?;
                let address = [body[6], body[7], body[8]];
                let data = body[9..].to_vec();
                guard_data(&data)?;
                Ok(Message::ParameterChange {
                    device,
                    address,
                    data,
                })
            }
            KIND_BULK_DUMP => {
                // F0 43 0n 7F 1C bh bl 0B ah am al data... cc
                if body.len() < 11 {
                    return Err(SysExError::TooShort(bytes.len()));
                }
                let byte_count = crate::codec::read_u14(body[5], body[6]) as usize;
                check_model_at(body[7])?;
                let address = [body[8], body[9], body[10]];
                let data_and_cksum = &body[11..];
                if data_and_cksum.is_empty() {
                    return Err(SysExError::TooShort(bytes.len()));
                }
                let (data, cksum_byte) = data_and_cksum.split_at(data_and_cksum.len() - 1);
                guard_data(data)?;
                // Byte Count counts Model ID + Address + Data.
                let expected_count = data.len() + 4;
                if byte_count != expected_count {
                    return Err(SysExError::ByteCountMismatch {
                        declared: byte_count,
                        actual: expected_count,
                    });
                }
                let mut sum_region = vec![MODEL_ID];
                sum_region.extend_from_slice(&address);
                sum_region.extend_from_slice(data);
                let expected = checksum(&sum_region);
                if cksum_byte[0] != expected {
                    return Err(SysExError::ChecksumMismatch {
                        expected,
                        actual: cksum_byte[0],
                    });
                }
                Ok(Message::BulkDump {
                    device,
                    address,
                    data: data.to_vec(),
                })
            }
            KIND_PARAMETER_REQUEST | KIND_BULK_REQUEST => {
                // F0 43 [23]n 7F 1C 0B ah am al
                check_model(body, 5)?;
                if body.len() < 9 {
                    return Err(SysExError::TooShort(bytes.len()));
                }
                let address = [body[6], body[7], body[8]];
                if kind == KIND_PARAMETER_REQUEST {
                    Ok(Message::ParameterRequest { device, address })
                } else {
                    Ok(Message::BulkRequest { device, address })
                }
            }
            other => Err(SysExError::UnknownKind(other)),
        }
    }
}

fn check_model(body: &[u8], idx: usize) -> Result<(), SysExError> {
    match body.get(idx) {
        Some(&MODEL_ID) => Ok(()),
        Some(&m) => Err(SysExError::WrongModel(m)),
        None => Err(SysExError::TooShort(body.len())),
    }
}

fn check_model_at(b: u8) -> Result<(), SysExError> {
    if b == MODEL_ID {
        Ok(())
    } else {
        Err(SysExError::WrongModel(b))
    }
}

fn guard_data(data: &[u8]) -> Result<(), SysExError> {
    if let Some(i) = data.iter().position(|&b| b >= 0x80) {
        return Err(SysExError::DataByteOutOfRange(i));
    }
    Ok(())
}

/// Yamaha bulk-dump checksum: the value that makes the lower 7 bits of the sum
/// of the supplied region plus the checksum equal zero.
pub fn checksum(region: &[u8]) -> u8 {
    let sum: u32 = region.iter().map(|&b| b as u32).sum();
    ((0x80 - (sum % 0x80)) % 0x80) as u8
}

/// The standard Universal Identity Request, broadcast to all devices:
/// `F0 7E 7F 06 01 F7`.
pub fn identity_request() -> [u8; 6] {
    [0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7]
}

/// Classify an Identity Reply's device-family bytes, if present.
///
/// The CK reply has the form
/// `F0 7E 0n 06 02 43 00 41 ff ff vv 00 00 7F F7`, where `ff ff` (bytes 8..=9)
/// is the device family: `62 06` = CK61, `63 06` = CK88.
pub fn identify_reply(reply: &[u8]) -> Option<CkModel> {
    if reply.len() < 10 || reply[0] != 0xF0 || reply[1] != 0x7E || reply[4] != 0x02 {
        return None;
    }
    let family = [reply[8], reply[9]];
    if family == FAMILY_CK61 {
        Some(CkModel::Ck61)
    } else if family == FAMILY_CK88 {
        Some(CkModel::Ck88)
    } else {
        None
    }
}

/// Which CK an Identity Reply came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CkModel {
    Ck61,
    Ck88,
}

impl CkModel {
    pub fn name(self) -> &'static str {
        match self {
            CkModel::Ck61 => "CK61",
            CkModel::Ck88 => "CK88",
        }
    }
}

/// Errors from [`Message::decode`].
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum SysExError {
    #[error("frame too short ({0} bytes)")]
    TooShort(usize),
    #[error("missing F0 start sentinel")]
    MissingStart,
    #[error("missing F7 end sentinel")]
    MissingEnd,
    #[error("not a Yamaha frame (manufacturer id {0:#04x})")]
    NotYamaha(u8),
    #[error("bad group number {0:02x?} (expected 7F 1C)")]
    BadGroup([u8; 2]),
    #[error("wrong model id {0:#04x} (expected CK {MODEL_ID:#04x})")]
    WrongModel(u8),
    #[error("unknown message kind (device-byte high nibble {0:#04x})")]
    UnknownKind(u8),
    #[error("data byte out of range (>= 0x80) at index {0}")]
    DataByteOutOfRange(usize),
    #[error("byte count mismatch: declared {declared}, got {actual} data bytes")]
    ByteCountMismatch { declared: usize, actual: usize },
    #[error("checksum mismatch: expected {expected:#04x}, got {actual:#04x}")]
    ChecksumMismatch { expected: u8, actual: u8 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_change_round_trip() {
        // Real-world example from the forums: parts on/off at 50 01 19 = 01.
        let m = Message::ParameterChange {
            device: 0,
            address: [0x50, 0x01, 0x19],
            data: vec![0x01],
        };
        let bytes = m.encode();
        assert_eq!(
            bytes,
            [0xF0, 0x43, 0x10, 0x7F, 0x1C, 0x0B, 0x50, 0x01, 0x19, 0x01, 0xF7]
        );
        assert_eq!(Message::decode(&bytes).unwrap(), m);
    }

    #[test]
    fn parameter_change_known_voice_example() {
        // F0 43 10 7F 1C 0B 50 00 00 03 F7 — select category at 50 00 00 = 03.
        let bytes = [
            0xF0, 0x43, 0x10, 0x7F, 0x1C, 0x0B, 0x50, 0x00, 0x00, 0x03, 0xF7,
        ];
        let m = Message::decode(&bytes).unwrap();
        assert_eq!(
            m,
            Message::ParameterChange {
                device: 0,
                address: [0x50, 0x00, 0x00],
                data: vec![0x03],
            }
        );
    }

    #[test]
    fn bulk_dump_round_trip_and_checksum() {
        let m = Message::BulkDump {
            device: 0,
            address: [0x20, 0x00, 0x00],
            data: vec![0x01, 0x02, 0x03, 0x7F],
        };
        let bytes = m.encode();
        // F0 43 0n 7F 1C bh bl 0B ah am al data cc F7
        assert_eq!(bytes[2], 0x00);
        assert_eq!(&bytes[3..5], &GROUP);
        assert_eq!(Message::decode(&bytes).unwrap(), m);
    }

    #[test]
    fn bulk_dump_rejects_bad_checksum() {
        let m = Message::BulkDump {
            device: 0,
            address: [0x20, 0x00, 0x00],
            data: vec![0x10, 0x20],
        };
        let mut bytes = m.encode();
        let cksum_idx = bytes.len() - 2;
        bytes[cksum_idx] ^= 0x01;
        assert!(matches!(
            Message::decode(&bytes),
            Err(SysExError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn requests_round_trip() {
        for m in [
            Message::ParameterRequest {
                device: 5,
                address: [0x20, 0x00, 0x00],
            },
            Message::BulkRequest {
                device: 5,
                address: [0x46, 0x00, 0x00],
            },
        ] {
            let bytes = m.encode();
            assert_eq!(Message::decode(&bytes).unwrap(), m);
        }
    }

    #[test]
    fn rejects_non_yamaha() {
        let mut bytes = Message::ParameterRequest {
            device: 0,
            address: [0x20, 0x00, 0x00],
        }
        .encode();
        bytes[1] = 0x42;
        assert_eq!(Message::decode(&bytes), Err(SysExError::NotYamaha(0x42)));
    }

    #[test]
    fn checksum_makes_sum_zero_mod_128() {
        let region = [0x05, 0x10, 0x0B, 0x20, 0x00, 0x00, 0x7F, 0x01];
        let cc = checksum(&region);
        let total: u32 = region.iter().map(|&b| b as u32).sum::<u32>() + cc as u32;
        assert_eq!(total % 0x80, 0);
    }

    #[test]
    fn identity_reply_detects_models() {
        let ck88 = [
            0xF0, 0x7E, 0x00, 0x06, 0x02, 0x43, 0x00, 0x41, 0x63, 0x06, 0x00, 0x00, 0x00, 0x7F,
            0xF7,
        ];
        assert_eq!(identify_reply(&ck88), Some(CkModel::Ck88));
        let ck61 = [
            0xF0, 0x7E, 0x00, 0x06, 0x02, 0x43, 0x00, 0x41, 0x62, 0x06, 0x00, 0x00, 0x00, 0x7F,
            0xF7,
        ];
        assert_eq!(identify_reply(&ck61), Some(CkModel::Ck61));
    }
}
