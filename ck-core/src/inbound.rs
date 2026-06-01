//! Classify any byte sequence that arrived from the device into a single tagged
//! value, so MIDI callers don't have to know about message kinds and address
//! prefixes.

use crate::address::AddressSpace;
use crate::sysex::{Message, SysExError, SYSEX_START};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundMessage {
    /// A Bulk Dump, with the area it targets resolved from its address.
    BulkDump {
        device: u8,
        space: AddressSpace,
        address: [u8; 3],
        data: Vec<u8>,
    },
    /// A Parameter Change.
    ParameterChange {
        device: u8,
        space: AddressSpace,
        address: [u8; 3],
        data: Vec<u8>,
    },
    /// A request the device echoed back (Bulk or Parameter).
    Request {
        device: u8,
        address: [u8; 3],
        bulk: bool,
    },
    /// A Universal Identity Reply (`F0 7E .. 06 02 ..`).
    IdentityReply { bytes: Vec<u8> },
    /// SysEx we couldn't decode as a CK message.
    UnparseableSysEx { bytes: Vec<u8>, error: SysExError },
    /// Bytes that weren't SysEx at all (Note On/Off, CC, Program Change, …).
    NonSysEx(Vec<u8>),
}

/// Classify a complete inbound MIDI byte sequence.
pub fn classify_inbound(bytes: &[u8]) -> InboundMessage {
    if bytes.first() != Some(&SYSEX_START) {
        return InboundMessage::NonSysEx(bytes.to_vec());
    }
    // Universal Identity Reply: F0 7E <dev> 06 02 ...
    if bytes.len() >= 5 && bytes[1] == 0x7E && bytes[3] == 0x06 && bytes[4] == 0x02 {
        return InboundMessage::IdentityReply {
            bytes: bytes.to_vec(),
        };
    }
    match Message::decode(bytes) {
        Ok(Message::BulkDump {
            device,
            address,
            data,
        }) => InboundMessage::BulkDump {
            device,
            space: AddressSpace::classify(address),
            address,
            data,
        },
        Ok(Message::ParameterChange {
            device,
            address,
            data,
        }) => InboundMessage::ParameterChange {
            device,
            space: AddressSpace::classify(address),
            address,
            data,
        },
        Ok(Message::BulkRequest { device, address }) => InboundMessage::Request {
            device,
            address,
            bulk: true,
        },
        Ok(Message::ParameterRequest { device, address }) => InboundMessage::Request {
            device,
            address,
            bulk: false,
        },
        Err(error) => InboundMessage::UnparseableSysEx {
            bytes: bytes.to_vec(),
            error,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_parameter_change() {
        let bytes = Message::ParameterChange {
            device: 0,
            address: [0x50, 0x00, 0x00],
            data: vec![0x03],
        }
        .encode();
        match classify_inbound(&bytes) {
            InboundMessage::ParameterChange { space, address, .. } => {
                assert_eq!(space, AddressSpace::Part);
                assert_eq!(address, [0x50, 0x00, 0x00]);
            }
            other => panic!("expected ParameterChange, got {other:?}"),
        }
    }

    #[test]
    fn classifies_identity_reply() {
        let bytes = [
            0xF0, 0x7E, 0x00, 0x06, 0x02, 0x43, 0x00, 0x41, 0x63, 0x06, 0x00, 0x00, 0x00, 0x7F,
            0xF7,
        ];
        assert!(matches!(
            classify_inbound(&bytes),
            InboundMessage::IdentityReply { .. }
        ));
    }

    #[test]
    fn classifies_non_sysex() {
        assert!(matches!(
            classify_inbound(&[0x90, 0x40, 0x7F]),
            InboundMessage::NonSysEx(_)
        ));
    }

    #[test]
    fn classifies_unparseable() {
        assert!(matches!(
            classify_inbound(&[0xF0, 0x41, 0x00, 0xF7]),
            InboundMessage::UnparseableSysEx { .. }
        ));
    }
}
