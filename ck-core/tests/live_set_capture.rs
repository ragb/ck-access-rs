//! Decodes a **real CK88 full Live Set bulk dump** (the 14-message sequence the
//! device emits in response to a Bulk Header request) and round-trips every
//! known block byte-exact. This validates the typed Live Set / Zone / Part
//! models against actual hardware data, not just synthetic payloads.

use ck_core::{LiveSet, LiveSetCommon, LiveSetEq, Message, Part, Zone};

const FULL: &[u8] = include_bytes!("fixtures/live_set_full.syx");

/// Split a concatenation of SysEx frames (each F0..F7) into individual frames.
fn split_frames(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut frames = Vec::new();
    let mut cur = Vec::new();
    for &b in bytes {
        if b == 0xF0 {
            cur.clear();
        }
        cur.push(b);
        if b == 0xF7 {
            frames.push(std::mem::take(&mut cur));
        }
    }
    frames
}

#[test]
fn full_live_set_blocks_decode_and_round_trip() {
    let frames = split_frames(FULL);
    assert_eq!(frames.len(), 14, "expected 14 framed messages");

    let mut saw_common = false;
    let mut zones = 0;
    let mut parts = 0;

    for frame in &frames {
        let msg = Message::decode(frame).expect("decode bulk dump frame");
        let (address, data) = match &msg {
            Message::BulkDump { address, data, .. } => (*address, data.clone()),
            other => panic!("expected BulkDump, got {other:?}"),
        };
        // Every frame must re-encode to the device's exact bytes.
        assert_eq!(
            &msg.encode(),
            frame,
            "frame re-encode mismatch at {address:02X?}"
        );

        match address {
            [0x46, 0x00, 0x00] => {
                let c = LiveSetCommon::from_bytes(&data).expect("LiveSetCommon decodes");
                assert_eq!(c.to_bytes().unwrap(), data);
                saw_common = true;
            }
            [0x46, 0x40, 0x00] => {
                let eq = LiveSetEq::from_bytes(&data).expect("LiveSetEq decodes");
                assert_eq!(eq.to_bytes().unwrap(), data);
            }
            [0x4A, _, 0x00] => {
                let z = Zone::from_bytes(&data).expect("Zone decodes");
                assert_eq!(z.to_bytes().unwrap(), data);
                zones += 1;
            }
            [0x50, _, 0x00] => {
                let p = Part::from_bytes(&data).expect("Part decodes");
                assert_eq!(p.to_bytes().unwrap(), data);
                parts += 1;
            }
            _ => {} // header/footer/soundmondo/audio-trigger/undocumented blocks
        }
    }

    assert!(saw_common);
    assert_eq!(zones, 4);
    assert_eq!(parts, 3);
}

#[test]
fn full_live_set_assembles_and_reserializes_to_same_blocks() {
    // The (address, data) content blocks from the dump (excluding header/footer).
    let mut content: Vec<([u8; 3], Vec<u8>)> = Vec::new();
    for frame in split_frames(FULL) {
        if let Message::BulkDump { address, data, .. } = Message::decode(&frame).unwrap() {
            if !matches!(address, [0x0E, _, _] | [0x0F, _, _]) {
                content.push((address, data));
            }
        }
    }
    content.sort_by_key(|(a, _)| *a);

    let live_set = LiveSet::from_blocks(&content).expect("assemble LiveSet from dump");
    assert_eq!(live_set.zones.len(), 4);
    assert_eq!(live_set.parts.len(), 3);
    // The v1.10 Rotary block (46 20 00) is typed; defaults confirm the layout.
    let rotary = live_set.rotary.as_ref().expect("rotary block present");
    assert_eq!(rotary.b_horn_transition, 118);
    assert_eq!(rotary.b_rotor_transition, 116);
    // The Soundmondo version block is still preserved verbatim.
    assert!(live_set
        .extra_blocks
        .iter()
        .any(|b| b.address == [0x00, 0x7F, 0x00]));

    // Re-encoding yields exactly the same content blocks the device sent.
    assert_eq!(live_set.to_blocks().unwrap(), content);
}
