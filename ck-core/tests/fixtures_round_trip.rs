//! Byte-exact round-trip tests against **real device captures** taken from a
//! CK88 over USB MIDI (Bulk Dumps to `system_common` and `master_eq`). These
//! confirm the SysEx framing, Byte Count, and checksum against hardware — not
//! just synthetic payloads.

use ck_core::{MasterEq, Message, SystemCommon};

const SYSTEM_COMMON_DUMP: &[u8] = include_bytes!("fixtures/system_common_dump.syx");
const MASTER_EQ_DUMP: &[u8] = include_bytes!("fixtures/master_eq_dump.syx");

#[test]
fn system_common_dump_decodes_and_round_trips() {
    let msg = Message::decode(SYSTEM_COMMON_DUMP).expect("decode bulk dump frame");
    let (address, data) = match &msg {
        Message::BulkDump { address, data, .. } => (*address, data.clone()),
        other => panic!("expected BulkDump, got {other:?}"),
    };
    assert_eq!(address, [0x20, 0x00, 0x00]);
    assert_eq!(data.len(), 56);

    // Re-encoding the decoded frame reproduces the device's exact bytes
    // (byte count + checksum included).
    assert_eq!(msg.encode(), SYSTEM_COMMON_DUMP);

    // The typed model round-trips the payload byte-exact.
    let common = SystemCommon::from_bytes(&data).expect("decode SystemCommon");
    assert_eq!(common.to_bytes().unwrap(), data);
}

#[test]
fn master_eq_dump_decodes_and_round_trips() {
    let msg = Message::decode(MASTER_EQ_DUMP).expect("decode bulk dump frame");
    let (address, data) = match &msg {
        Message::BulkDump { address, data, .. } => (*address, data.clone()),
        other => panic!("expected BulkDump, got {other:?}"),
    };
    assert_eq!(address, [0x20, 0x40, 0x00]);
    assert_eq!(data.len(), 20);
    assert_eq!(msg.encode(), MASTER_EQ_DUMP);

    let eq = MasterEq::from_bytes(&data).expect("decode MasterEq");
    assert_eq!(eq.to_bytes().unwrap(), data);
}
