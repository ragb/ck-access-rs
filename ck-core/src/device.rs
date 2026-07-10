//! [`Device`] implementation for the Yamaha CK — the adapter that plugs the
//! CK's typed codec into the generic CLI engine (`midi-access-cli`) and editor
//! tooling, using `serde_yaml::Value` as the document lingua franca.
//!
//! Two areas: `system` (System Common + Master EQ, two bulk blocks) and
//! `live-set` (the framed Bulk Header → content blocks → Bulk Footer sequence).
//! `request`/`encode` return all the SysEx frames for an operation concatenated;
//! `decode` splits and reassembles a collected dump stream. No on-the-wire
//! behaviour changes — this only re-expresses the existing block codec.

use serde_yaml::Value;

use midi_access_core::{Area, Catalogs, Device, DeviceError, Inbound, Params};

use crate::address::{
    AddressSpace, BULK_FOOTER_CURRENT_BUFFER, BULK_HEADER_CURRENT_BUFFER, MASTER_EQ_BASE,
    STORE_TO_FLASH_BASE, SYSTEM_COMMON_BASE,
};
use crate::document::{from_value_over_default, AddressedBlock};
use crate::sysex::Message;
use crate::{
    classify_inbound, identify_reply, InboundMessage, LiveSet, MasterEq, System, SystemCommon,
};

/// The Yamaha CK61 / CK88.
pub struct Ck;

const AREAS: &[Area] = &[
    Area {
        name: "system",
        label: "System",
        about: "Global settings + Master EQ",
    },
    Area {
        name: "live-set",
        label: "Live Set",
        about: "One patch: common settings, EQ, 4 Zones, 3 Parts",
    },
];

/// Map a CLI area token to its canonical name, or an [`DeviceError::UnknownArea`].
fn canon(area: &str) -> Result<&'static str, DeviceError> {
    AREAS
        .iter()
        .find(|a| a.matches(area))
        .map(|a| a.name)
        .ok_or_else(|| DeviceError::UnknownArea(area.to_string()))
}

fn dec(e: impl std::fmt::Display) -> DeviceError {
    DeviceError::Decode(e.to_string())
}
fn enc(e: impl std::fmt::Display) -> DeviceError {
    DeviceError::Encode(e.to_string())
}

/// Parse a `--store` destination like `"20-8"` into `(page, sound)`, both
/// 1-based. Accepts `-`, `/`, `.`, or `,` as the separator.
fn parse_slot(dest: &str) -> Result<(u8, u8), DeviceError> {
    let dest = dest.trim();
    let (p, s) = dest
        .split_once(['-', '/', '.', ','])
        .ok_or_else(|| enc(format!("store needs a slot like `20-8`, got {dest:?}")))?;
    let page = p
        .trim()
        .parse::<u8>()
        .map_err(|_| enc(format!("bad page in {dest:?}")))?;
    let sound = s
        .trim()
        .parse::<u8>()
        .map_err(|_| enc(format!("bad sound in {dest:?}")))?;
    Ok((page, sound))
}

/// Collect the `(address, data)` of every Bulk Dump frame in a raw dump stream.
fn collect_blocks(dump: &[u8]) -> Vec<AddressedBlock> {
    midi_access_core::split_sysex(dump)
        .into_iter()
        .filter_map(|frame| match Message::decode(&frame) {
            Ok(Message::BulkDump { address, data, .. }) => Some((address, data)),
            _ => None,
        })
        .collect()
}

impl Device for Ck {
    const NAME: &'static str = "ck";

    fn areas() -> &'static [Area] {
        AREAS
    }

    fn params() -> Params {
        crate::params::params()
    }

    fn catalogs() -> &'static dyn Catalogs {
        &crate::catalog::CK_CATALOGS
    }

    fn defaults(area: &str) -> Option<Value> {
        match canon(area).ok()? {
            "system" => serde_yaml::to_value(System::default()).ok(),
            "live-set" => serde_yaml::to_value(LiveSet::default()).ok(),
            _ => None,
        }
    }

    fn schema(area: &str) -> Option<String> {
        #[cfg(feature = "schema")]
        {
            use midi_access_core::schema::schema_json;
            match canon(area).ok()? {
                "system" => Some(schema_json::<System>()),
                "live-set" => Some(schema_json::<LiveSet>()),
                _ => None,
            }
        }
        #[cfg(not(feature = "schema"))]
        {
            let _ = area;
            None
        }
    }

    fn request(area: &str, ch: u8) -> Result<Vec<u8>, DeviceError> {
        match canon(area)? {
            // System blocks each answer an individual Bulk Request.
            "system" => {
                let mut out = Message::BulkRequest {
                    device: ch,
                    address: SYSTEM_COMMON_BASE,
                }
                .encode();
                out.extend(
                    Message::BulkRequest {
                        device: ch,
                        address: MASTER_EQ_BASE,
                    }
                    .encode(),
                );
                Ok(out)
            }
            // A whole Live Set is streamed back after a request to the Bulk Header.
            "live-set" => Ok(Message::BulkRequest {
                device: ch,
                address: BULK_HEADER_CURRENT_BUFFER,
            }
            .encode()),
            other => Err(DeviceError::UnknownArea(other.to_string())),
        }
    }

    fn decode(area: &str, dump: &[u8]) -> Result<Value, DeviceError> {
        let blocks = collect_blocks(dump);
        match canon(area)? {
            "system" => {
                let common = blocks
                    .iter()
                    .find(|(a, _)| *a == SYSTEM_COMMON_BASE)
                    .ok_or_else(|| dec("no System Common block in dump"))?;
                let master_eq = blocks
                    .iter()
                    .find(|(a, _)| *a == MASTER_EQ_BASE)
                    .ok_or_else(|| dec("no Master EQ block in dump"))?;
                let system = System {
                    common: SystemCommon::from_bytes(&common.1).map_err(dec)?,
                    master_eq: MasterEq::from_bytes(&master_eq.1).map_err(dec)?,
                };
                serde_yaml::to_value(system).map_err(dec)
            }
            "live-set" => {
                let ls = LiveSet::from_blocks(&blocks).map_err(dec)?;
                serde_yaml::to_value(ls).map_err(dec)
            }
            other => Err(DeviceError::UnknownArea(other.to_string())),
        }
    }

    fn encode(area: &str, doc: &Value, ch: u8) -> Result<Vec<u8>, DeviceError> {
        match canon(area)? {
            "system" => {
                let s: System = from_value_over_default(doc.clone()).map_err(enc)?;
                let mut out = Message::BulkDump {
                    device: ch,
                    address: SYSTEM_COMMON_BASE,
                    data: s.common.to_bytes().map_err(enc)?,
                }
                .encode();
                out.extend(
                    Message::BulkDump {
                        device: ch,
                        address: MASTER_EQ_BASE,
                        data: s.master_eq.to_bytes().map_err(enc)?,
                    }
                    .encode(),
                );
                Ok(out)
            }
            "live-set" => {
                let ls: LiveSet = from_value_over_default(doc.clone()).map_err(enc)?;
                // `sync` writes the volatile edit buffer (audible immediately).
                // Persisting to a slot is a separate `--store` step; see `store`.
                live_set_frames(
                    &ls,
                    ch,
                    BULK_HEADER_CURRENT_BUFFER,
                    BULK_FOOTER_CURRENT_BUFFER,
                )
            }
            other => Err(DeviceError::UnknownArea(other.to_string())),
        }
    }

    /// Persist a Live Set to a non-volatile slot (the panel STORE button, and
    /// the Melas editor's "Store to…"). `dest` is the destination slot as
    /// `"page-sound"`, 1-based (e.g. `"20-8"`).
    ///
    /// A *copy-to-slot* store: the CK has no "commit the edit buffer in place"
    /// command, so — exactly as the Melas editor does — we re-encode `doc`
    /// bracketed to the target **slot** (`0E pp 0n` … `0F pp 0n`) and then send
    /// the data-less Store To Flash commit (`0D 00 00`). The slot-bracketed
    /// dump alone only touches working RAM; the commit is what survives a power
    /// cycle. `doc` is the same Live Set the engine handed to [`encode`].
    fn store(area: &str, doc: &Value, dest: &str, ch: u8) -> Option<Result<Vec<u8>, DeviceError>> {
        Some((|| {
            if canon(area)? != "live-set" {
                return Err(enc("store is only defined for the live-set area"));
            }
            let (page, sound) = parse_slot(dest)?;
            let header = crate::address::bulk_header_for_slot(page, sound).ok_or_else(|| {
                enc(format!(
                    "slot {dest:?} out of range (page 1..=20, sound 1..=8)"
                ))
            })?;
            let footer = crate::address::bulk_footer_for_slot(page, sound).unwrap();
            let ls: LiveSet = from_value_over_default(doc.clone()).map_err(enc)?;
            let mut out = live_set_frames(&ls, ch, header, footer)?;
            out.extend(Ck::store_to_flash(ch));
            Ok(out)
        })())
    }

    /// Load a stored Live Set into the edit buffer by the standard MIDI dance
    /// of Bank Select MSB/LSB + Program Change (see [`live_set_select`]). `dest`
    /// is the slot as `"page-sound"`, 1-based (e.g. `"20-8"`); `channel` is the
    /// 1-based MIDI channel to address — the engine auto-detects it from the
    /// System `rx_channel` (see [`Ck::recall_channel`]).
    ///
    /// Both the System "Tx/Rx Bank" and "Tx/Rx Pgm" switches must be on for the
    /// CK to act on these (they are on by factory default).
    ///
    /// [`live_set_select`]: crate::live_set_select
    fn recall(dest: &str, channel: u8) -> Option<Result<Vec<Vec<u8>>, DeviceError>> {
        Some((|| {
            let (page, sound) = parse_slot(dest)?;
            crate::live_set_select::select_live_set_messages(channel, page, sound)
                .map(|frames| frames.to_vec())
                .map_err(|e| enc(e.to_string()))
        })())
    }

    /// The CK's receive channel lives in the System document, so dump that to
    /// learn which channel a recall should address.
    fn recall_channel_area() -> Option<&'static str> {
        Some("system")
    }

    /// Read the 1-based MIDI channel from a decoded System document. The raw
    /// `rx_channel` is `0..=15` for channels 1–16 and `0x10` for "All"; "All"
    /// (and anything unexpected) maps to channel 1, matching the editor.
    fn recall_channel(doc: &Value) -> Option<u8> {
        let rx = doc.get("common")?.get("rx_channel")?.as_u64()?;
        Some(if rx <= 15 { rx as u8 + 1 } else { 1 })
    }

    fn classify_inbound(bytes: &[u8]) -> Inbound {
        match classify_inbound(bytes) {
            InboundMessage::BulkDump {
                space,
                address,
                data,
                ..
            } => Inbound::Dump {
                area: area_for_space(space),
                address: address.to_vec(),
                data,
            },
            InboundMessage::ParameterChange { address, data, .. } => Inbound::Parameter {
                address: address.to_vec(),
                data,
            },
            InboundMessage::Request { address, .. } => Inbound::Request {
                address: address.to_vec(),
            },
            InboundMessage::IdentityReply { bytes } => Inbound::Identity {
                model: identify_reply(&bytes).map(|m| m.name().to_string()),
                bytes,
            },
            InboundMessage::UnparseableSysEx { bytes, .. } => Inbound::Other(bytes),
            InboundMessage::NonSysEx(bytes) => Inbound::Other(bytes),
        }
    }

    fn accepts(area: &str, doc: &Value) -> bool {
        // Parse-level kind check (matches the pre-migration `show`/`lint`): a file
        // may deserialize into the typed model yet fail to byte-encode.
        match canon(area) {
            Ok("system") => from_value_over_default::<System>(doc.clone()).is_ok(),
            Ok("live-set") => from_value_over_default::<LiveSet>(doc.clone()).is_ok(),
            _ => false,
        }
    }
}

impl Ck {
    /// The data-less **Store To Flash** commit (`0D 00 00`) — the frame that
    /// makes the CK write its pending Live Set changes to non-volatile memory,
    /// the SysEx equivalent of the panel STORE button.
    ///
    /// On its own this does nothing useful; it's the tail of the store
    /// sequence built by [`store`](Device::store): a Live Set dumped bracketed
    /// to the target slot (`0E pp 0n` … `0F pp 0n`), then this commit. A slot
    /// dump without it is lost on a power cycle.
    ///
    /// Wire format (verified by intercepting the Melas editor's output):
    /// `F0 43 00 7F 1C 00 04 0B 0D 00 00 68 F7`.
    pub fn store_to_flash(ch: u8) -> Vec<u8> {
        Message::BulkDump {
            device: ch,
            address: STORE_TO_FLASH_BASE,
            data: Vec::new(),
        }
        .encode()
    }
}

/// Encode a Live Set as the CK's bulk envelope: `header`, every content block,
/// `footer`. Use the current-buffer brackets ([`BULK_HEADER_CURRENT_BUFFER`])
/// to write the audible edit buffer, or the slot brackets
/// ([`bulk_header_for_slot`](crate::address::bulk_header_for_slot)) to write a
/// stored slot.
fn live_set_frames(
    ls: &LiveSet,
    ch: u8,
    header: [u8; 3],
    footer: [u8; 3],
) -> Result<Vec<u8>, DeviceError> {
    let blocks = ls.to_blocks().map_err(enc)?;
    let mut out = Message::BulkDump {
        device: ch,
        address: header,
        data: Vec::new(),
    }
    .encode();
    for (addr, data) in &blocks {
        out.extend(
            Message::BulkDump {
                device: ch,
                address: *addr,
                data: data.clone(),
            }
            .encode(),
        );
    }
    out.extend(
        Message::BulkDump {
            device: ch,
            address: footer,
            data: Vec::new(),
        }
        .encode(),
    );
    Ok(out)
}

/// Which area a dump's address space belongs to, for inbound classification.
fn area_for_space(space: AddressSpace) -> Option<String> {
    match space {
        AddressSpace::System => Some("system".to_string()),
        AddressSpace::LiveSetCommon | AddressSpace::Zone | AddressSpace::Part => {
            Some("live-set".to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_round_trips_through_value() {
        let s = System::default();
        let doc = serde_yaml::to_value(&s).unwrap();
        let bytes = Ck::encode("system", &doc, 0).unwrap();
        let back = Ck::decode("system", &bytes).unwrap();
        let s2: System = serde_yaml::from_value(back).unwrap();
        assert_eq!(s, s2);
    }

    #[test]
    fn live_set_round_trips_through_value() {
        let ls = LiveSet::default();
        let doc = serde_yaml::to_value(&ls).unwrap();
        let bytes = Ck::encode("live-set", &doc, 0).unwrap();
        let back = Ck::decode("live-set", &bytes).unwrap();
        let ls2: LiveSet = serde_yaml::from_value(back).unwrap();
        assert_eq!(ls, ls2);
    }

    #[test]
    fn store_to_flash_commit_frame_matches_capture() {
        // The data-less Store To Flash commit, byte-exact against the Melas
        // capture: F0 43 00 7F 1C 00 04 0B 0D 00 00 68 F7.
        assert_eq!(
            Ck::store_to_flash(0),
            vec![0xF0, 0x43, 0x00, 0x7F, 0x1C, 0x00, 0x04, 0x0B, 0x0D, 0x00, 0x00, 0x68, 0xF7],
        );
        // Device number rides the low nibble of byte 2.
        assert_eq!(Ck::store_to_flash(3)[2], 0x03);
    }

    #[test]
    fn store_dumps_to_the_slot_then_commits() {
        let ls = LiveSet::default();
        let doc = serde_yaml::to_value(&ls).unwrap();
        let bytes = Ck::store("live-set", &doc, "20-1", 0).unwrap().unwrap();
        let frames = midi_access_core::split_sysex(&bytes);

        // First frame is the Bulk Header bracketed to the SLOT (0E 13 00 for
        // page 20 / sound 1) — NOT the edit buffer (0E 7F 00).
        assert_eq!(
            Message::decode(&frames[0]).unwrap().address(),
            [0x0E, 0x13, 0x00]
        );
        // Last frame is the Store To Flash commit.
        assert_eq!(frames.last().unwrap(), &Ck::store_to_flash(0));
        // The frame before it is the slot footer 0F 13 00.
        let footer = &frames[frames.len() - 2];
        assert_eq!(
            Message::decode(footer).unwrap().address(),
            [0x0F, 0x13, 0x00]
        );

        // Bad area / slot are errors (but Some — the device supports store).
        assert!(Ck::store("system", &doc, "1-1", 0).unwrap().is_err());
        assert!(Ck::store("live-set", &doc, "", 0).unwrap().is_err());
        assert!(Ck::store("live-set", &doc, "21-1", 0).unwrap().is_err());
    }

    #[test]
    fn recall_builds_bank_and_program_change_from_a_slot() {
        // "20-8" on channel 1 → the same three frames as select_live_set_messages.
        let frames = Ck::recall("20-8", 1).unwrap().unwrap();
        assert_eq!(
            frames,
            crate::live_set_select::select_live_set_messages(1, 20, 8)
                .unwrap()
                .to_vec()
        );
        // Alternate separators route through parse_slot too.
        assert_eq!(
            Ck::recall("1/1", 10).unwrap().unwrap(),
            crate::live_set_select::select_live_set_messages(10, 1, 1)
                .unwrap()
                .to_vec()
        );
        // Malformed / out-of-range slots and channels are supported-but-errors.
        assert!(Ck::recall("nope", 1).unwrap().is_err());
        assert!(Ck::recall("21-1", 1).unwrap().is_err());
        assert!(Ck::recall("1-1", 17).unwrap().is_err());
    }

    #[test]
    fn recall_channel_reads_rx_channel_from_system() {
        assert_eq!(Ck::recall_channel_area(), Some("system"));
        // rx_channel 0 (ch 1) .. 15 (ch 16) map to 1-based; 0x10 "All" → 1.
        let doc = |rx: u8| {
            serde_yaml::to_value(System::default())
                .map(|mut v| {
                    v["common"]["rx_channel"] = rx.into();
                    v
                })
                .unwrap()
        };
        assert_eq!(Ck::recall_channel(&doc(0)), Some(1));
        assert_eq!(Ck::recall_channel(&doc(9)), Some(10));
        assert_eq!(Ck::recall_channel(&doc(15)), Some(16));
        assert_eq!(Ck::recall_channel(&doc(16)), Some(1)); // All → 1
    }

    #[test]
    fn area_lenient_and_unknown() {
        assert!(canon("live_set").is_ok());
        assert!(canon("LIVE-SET").is_ok());
        assert!(canon("bogus").is_err());
    }

    #[test]
    fn accepts_matches_parse_not_encode() {
        // A partial Live Set with a 2-element category_voices parses (Vec) but
        // would fail to byte-encode (needs 10) — `accepts` must still say yes.
        let v: Value = serde_yaml::from_str("parts:\n- category_voices: [0, 13]\n").unwrap();
        assert!(Ck::accepts("live-set", &v));
        assert!(Ck::encode("live-set", &v, 0).is_err());
    }
}
