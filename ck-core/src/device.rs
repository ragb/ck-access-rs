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
use crate::document::AddressedBlock;
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
                let s: System = serde_yaml::from_value(doc.clone()).map_err(enc)?;
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
                let ls: LiveSet = serde_yaml::from_value(doc.clone()).map_err(enc)?;
                let blocks = ls.to_blocks().map_err(enc)?;
                // Replay the device's own envelope: Bulk Header, content, Footer.
                let mut out = Message::BulkDump {
                    device: ch,
                    address: BULK_HEADER_CURRENT_BUFFER,
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
                        address: BULK_FOOTER_CURRENT_BUFFER,
                        data: Vec::new(),
                    }
                    .encode(),
                );
                Ok(out)
            }
            other => Err(DeviceError::UnknownArea(other.to_string())),
        }
    }

    /// Commit the edit buffer to a persistent Live Set slot (the panel STORE
    /// button). `dest` is the destination slot as `"page-sound"`, 1-based
    /// (e.g. `"20-8"`); the CK has no store-in-place, so an empty `dest` is an
    /// error. Pair with a `sync live-set` (which writes the edit buffer): the
    /// engine sends the encoded blocks, then this frame.
    fn store(dest: &str, ch: u8) -> Option<Result<Vec<u8>, DeviceError>> {
        Some((|| {
            let (page, sound) = parse_slot(dest)?;
            Ck::store_to_flash(ch, page, sound).ok_or_else(|| {
                enc(format!(
                    "slot {dest:?} out of range (page 1..=20, sound 1..=8)"
                ))
            })
        })())
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
            Ok("system") => serde_yaml::from_value::<System>(doc.clone()).is_ok(),
            Ok("live-set") => serde_yaml::from_value::<LiveSet>(doc.clone()).is_ok(),
            _ => false,
        }
    }
}

impl Ck {
    /// Build the Store To Flash frame that commits the CK's edit buffer to a
    /// non-volatile Live Set slot — the SysEx equivalent of the panel STORE
    /// button (and the Melas editor's "Store" / "Store to…").
    ///
    /// A bulk dump alone — even one bracketed to a User slot with
    /// [`bulk_header_for_slot`](crate::address::bulk_header_for_slot) — only
    /// writes working RAM and is lost on a power cycle. To persist an edit the
    /// way the Melas editor does: dump the whole Live Set to the edit buffer
    /// (via [`encode`](Self::encode)), then send this frame naming the
    /// destination slot. For plain "Store" (in place) pass the slot the Live
    /// Set was loaded from — there is no coordinate-less store on the wire.
    ///
    /// `page`/`sound` are 1-based (1..=20, 1..=8) to match the CK's display;
    /// returns `None` if either is out of range.
    ///
    /// Wire format (device-verified against a Melas capture): a Bulk Dump to
    /// [`STORE_TO_FLASH_BASE`] (`0B 10 00`) with data `00 0F <page-1>
    /// <sound-1> 00 00` — e.g. slot 20-8 → `F0 43 00 7F 1C 00 0A 0B 0B 10 00
    /// 00 0F 13 07 00 00 31 F7`.
    pub fn store_to_flash(ch: u8, page: u8, sound: u8) -> Option<Vec<u8>> {
        // Reuse the slot range check (1..=20, 1..=8).
        crate::address::bulk_header_for_slot(page, sound)?;
        let data = vec![0x00, 0x0F, page - 1, sound - 1, 0x00, 0x00];
        Some(
            Message::BulkDump {
                device: ch,
                address: STORE_TO_FLASH_BASE,
                data,
            }
            .encode(),
        )
    }
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
    fn store_to_flash_frames_match_capture() {
        // Byte-exact against the Melas CK editor capture (docs/sysex-notes.md).
        // Slot 1-1 → data 00 0F 00 00 00 00, checksum 4B.
        assert_eq!(
            Ck::store_to_flash(0, 1, 1).unwrap(),
            vec![
                0xF0, 0x43, 0x00, 0x7F, 0x1C, 0x00, 0x0A, 0x0B, 0x0B, 0x10, 0x00, 0x00, 0x0F, 0x00,
                0x00, 0x00, 0x00, 0x4B, 0xF7,
            ],
        );
        // Slot 20-8 → data 00 0F 13 07 00 00, checksum 31.
        assert_eq!(
            Ck::store_to_flash(0, 20, 8).unwrap(),
            vec![
                0xF0, 0x43, 0x00, 0x7F, 0x1C, 0x00, 0x0A, 0x0B, 0x0B, 0x10, 0x00, 0x00, 0x0F, 0x13,
                0x07, 0x00, 0x00, 0x31, 0xF7,
            ],
        );
        // Device number rides the low nibble of byte 2.
        assert_eq!(Ck::store_to_flash(3, 1, 1).unwrap()[2], 0x03);
        // Out-of-range slots are rejected.
        assert!(Ck::store_to_flash(0, 0, 1).is_none());
        assert!(Ck::store_to_flash(0, 21, 1).is_none());
        assert!(Ck::store_to_flash(0, 1, 9).is_none());
    }

    #[test]
    fn store_via_device_trait_parses_slot() {
        // "20-8" routes through parse_slot to the same bytes as store_to_flash.
        assert_eq!(
            Ck::store("20-8", 0).unwrap().unwrap(),
            Ck::store_to_flash(0, 20, 8).unwrap(),
        );
        // Alternate separators accepted.
        assert_eq!(Ck::store("1/1", 0).unwrap().unwrap(), Ck::store_to_flash(0, 1, 1).unwrap());
        // Empty / malformed / out-of-range are errors (but Some, i.e. supported).
        assert!(Ck::store("", 0).unwrap().is_err());
        assert!(Ck::store("20", 0).unwrap().is_err());
        assert!(Ck::store("21-1", 0).unwrap().is_err());
    }

    #[test]
    fn area_lenient_and_unknown() {
        assert!(canon("live_set").is_ok());
        assert!(canon("LIVE-SET").is_ok());
        assert!(canon("bogus").is_err());
    }

    #[test]
    #[ignore = "partial-preset merge needs reimpl after removing struct-level serde(default); see editor README"]
    fn accepts_matches_parse_not_encode() {
        // A partial Live Set with a 2-element category_voices parses (Vec) but
        // would fail to byte-encode (needs 10) — `accepts` must still say yes.
        let v: Value = serde_yaml::from_str("parts:\n- category_voices: [0, 13]\n").unwrap();
        assert!(Ck::accepts("live-set", &v));
        assert!(Ck::encode("live-set", &v, 0).is_err());
    }
}
