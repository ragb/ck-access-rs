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
    AddressSpace, BULK_FOOTER_BASE, BULK_HEADER_BASE, MASTER_EQ_BASE, SYSTEM_COMMON_BASE,
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
                address: BULK_HEADER_BASE,
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
                    address: BULK_HEADER_BASE,
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
                        address: BULK_FOOTER_BASE,
                        data: Vec::new(),
                    }
                    .encode(),
                );
                Ok(out)
            }
            other => Err(DeviceError::UnknownArea(other.to_string())),
        }
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
