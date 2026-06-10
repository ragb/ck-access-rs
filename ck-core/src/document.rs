//! Editable aggregate documents.
//!
//! The device exposes settings as several separately-addressed bulk blocks. For
//! editing it's friendlier to bundle them into two documents that match how a
//! player thinks: the global [`System`] and a single [`LiveSet`] patch (its
//! common settings, EQ, four Zones and three Parts). The CLI and wasm layers
//! dump/sync these as one YAML file each.
//!
//! A full Live Set is read by sending a Bulk Request to the **Bulk Header**
//! address; the device replies with the whole patch as a sequence of Bulk Dump
//! messages (header, Soundmondo version, Common, Audio Trigger, EQ, four Zones,
//! three Parts, footer). [`LiveSet::from_blocks`]/[`LiveSet::to_blocks`] convert
//! between that block sequence and the typed document, preserving any block this
//! crate doesn't model (Soundmondo version, firmware-specific blocks) verbatim
//! in [`LiveSet::extra_blocks`] so a dumped patch re-syncs byte-exact.

use serde::{Deserialize, Serialize};

use crate::address::{
    AUDIO_TRIGGER_PATH_BASE, LIVE_SET_COMMON_BASE, LIVE_SET_EQ_BASE, ROTARY_BASE, ZONE_COUNT,
};
use crate::codec::{read_ascii, CodecError};
use crate::live_set::{LiveSetCommon, LiveSetEq};
use crate::part::Part;
use crate::rotary::RotarySpeaker;
use crate::system::{MasterEq, SystemCommon};
use crate::zone::Zone;

/// The global System settings: Common block + Master EQ.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct System {
    pub common: SystemCommon,
    pub master_eq: MasterEq,
}

/// A `(3-byte address, data)` bulk block, as carried in a full Live Set dump.
pub type AddressedBlock = ([u8; 3], Vec<u8>);

/// A bulk block this crate doesn't model, kept verbatim for byte-exact re-sync.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawBlock {
    pub address: [u8; 3],
    pub data: Vec<u8>,
}

/// One Live Set patch: common settings, EQ, four Zones (1..4) and three Parts
/// (A/B/C). The Audio Trigger wave-file path is kept as a string.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LiveSet {
    pub common: LiveSetCommon,
    pub eq: LiveSetEq,
    /// Path to the audio-trigger sample file (may be empty).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub audio_trigger_path: String,
    /// The four Zones (master-keyboard transmit configs).
    pub zones: Vec<Zone>,
    /// The three Parts (A, B, C).
    pub parts: Vec<Part>,
    /// Rotary Speaker settings (firmware v1.10+). `None` on firmware that doesn't
    /// emit the `46 20 00` block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotary: Option<RotarySpeaker>,
    /// Blocks present in the dump that this crate doesn't model individually
    /// (Soundmondo format version, other firmware-specific blocks), kept verbatim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_blocks: Vec<RawBlock>,
}

impl Default for LiveSet {
    /// A complete, valid factory-style baseline: default Common/EQ/Rotary, four
    /// Zones (Zone 1 enabled, ascending transmit channels) and three Parts
    /// (Part A enabled+selected; per-slot colours). Edit-then-merge over this to
    /// build a preset without specifying every field.
    fn default() -> Self {
        let zones = (0..Self::ZONES as u8)
            .map(|i| Zone {
                zone_switch: i == 0,
                transmit_channel: i,
                ..Zone::default()
            })
            .collect();
        let colors = [0x02u8, 0x08, 0x04];
        let parts = (0..Self::PARTS)
            .map(|i| Part {
                part_switch: i == 0,
                part_selected: i == 0,
                part_color: colors[i],
                ..Part::default()
            })
            .collect();
        Self {
            common: LiveSetCommon::default(),
            eq: LiveSetEq::default(),
            audio_trigger_path: String::new(),
            zones,
            parts,
            rotary: Some(RotarySpeaker::default()),
            extra_blocks: Vec::new(),
        }
    }
}

impl LiveSet {
    /// Number of Zones a well-formed Live Set carries.
    pub const ZONES: usize = ZONE_COUNT as usize;
    /// Number of Parts a well-formed Live Set carries.
    pub const PARTS: usize = 3;

    /// Build a Live Set from the `(address, data)` blocks of a full bulk dump.
    /// Bulk Header/Footer (`0E`/`0F`) blocks are ignored; unmodelled blocks are
    /// preserved in [`Self::extra_blocks`].
    pub fn from_blocks(blocks: &[AddressedBlock]) -> Result<Self, CodecError> {
        let mut common = None;
        let mut eq = None;
        let mut audio_trigger_path = String::new();
        let mut zones: Vec<Option<Zone>> = vec![None; Self::ZONES];
        let mut parts: Vec<Option<Part>> = vec![None; Self::PARTS];
        let mut rotary = None;
        let mut extra_blocks = Vec::new();

        for (address, data) in blocks {
            match *address {
                LIVE_SET_COMMON_BASE => common = Some(LiveSetCommon::from_bytes(data)?),
                LIVE_SET_EQ_BASE => eq = Some(LiveSetEq::from_bytes(data)?),
                ROTARY_BASE => rotary = Some(RotarySpeaker::from_bytes(data)?),
                AUDIO_TRIGGER_PATH_BASE => {
                    audio_trigger_path = read_ascii(data, "audio_trigger_path")?
                }
                [0x4A, z, 0x00] if (z as usize) < Self::ZONES => {
                    zones[z as usize] = Some(Zone::from_bytes(data)?)
                }
                [0x50, p, 0x00] if (p as usize) < Self::PARTS => {
                    parts[p as usize] = Some(Part::from_bytes(data)?)
                }
                [0x0E, _, _] | [0x0F, _, _] => {} // bulk header / footer
                _ => extra_blocks.push(RawBlock {
                    address: *address,
                    data: data.clone(),
                }),
            }
        }

        Ok(Self {
            common: common.ok_or(CodecError::MissingBlock("Live Set Common"))?,
            eq: eq.ok_or(CodecError::MissingBlock("Live Set EQ"))?,
            audio_trigger_path,
            zones: zones
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .ok_or(CodecError::MissingBlock("Zone"))?,
            parts: parts
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .ok_or(CodecError::MissingBlock("Part"))?,
            rotary,
            extra_blocks,
        })
    }

    /// Encode the modelled + preserved blocks as `(address, data)` pairs, in the
    /// device's canonical (address-ascending) order. Does **not** include the
    /// Bulk Header/Footer — the MIDI layer adds those around the sequence.
    pub fn to_blocks(&self) -> Result<Vec<AddressedBlock>, CodecError> {
        if self.zones.len() != Self::ZONES {
            return Err(CodecError::WrongLength {
                expected: Self::ZONES,
                actual: self.zones.len(),
            });
        }
        if self.parts.len() != Self::PARTS {
            return Err(CodecError::WrongLength {
                expected: Self::PARTS,
                actual: self.parts.len(),
            });
        }

        let mut out: Vec<AddressedBlock> = Vec::new();
        out.push((LIVE_SET_COMMON_BASE, self.common.to_bytes()?));
        out.push((LIVE_SET_EQ_BASE, self.eq.to_bytes()?));
        // Audio Trigger path is variable-length ASCII (empty when unset).
        out.push((
            AUDIO_TRIGGER_PATH_BASE,
            self.audio_trigger_path.as_bytes().to_vec(),
        ));
        for (i, z) in self.zones.iter().enumerate() {
            out.push(([0x4A, i as u8, 0x00], z.to_bytes()?));
        }
        for (i, p) in self.parts.iter().enumerate() {
            out.push(([0x50, i as u8, 0x00], p.to_bytes()?));
        }
        if let Some(r) = &self.rotary {
            out.push((ROTARY_BASE, r.to_bytes()?));
        }
        for rb in &self.extra_blocks {
            out.push((rb.address, rb.data.clone()));
        }
        out.sort_by_key(|(a, _)| *a);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_default_is_byte_encodable() {
        let s = System::default();
        s.common.to_bytes().unwrap();
        s.master_eq.to_bytes().unwrap();
    }

    #[test]
    fn live_set_default_is_complete_and_encodable() {
        let ls = LiveSet::default();
        assert_eq!(ls.zones.len(), LiveSet::ZONES);
        assert_eq!(ls.parts.len(), LiveSet::PARTS);
        assert!(ls.zones[0].zone_switch); // Zone 1 on
        assert!(!ls.zones[1].zone_switch);
        assert_eq!(ls.zones[2].transmit_channel, 2);
        assert!(ls.parts[0].part_switch); // Part A on
        assert_eq!(ls.parts[1].part_color, 0x08);
        ls.to_blocks().unwrap(); // every block encodes
    }

    #[test]
    fn partial_yaml_merges_over_defaults() {
        // Sparse Live Set: only a name and one part's cutoff; the rest fills
        // from the factory default.
        let yaml = "common:\n  name: My Patch\nparts:\n- filter_cutoff: 100\n- {}\n- {}\nzones: [{}, {}, {}, {}]\n";
        let ls: LiveSet = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(ls.common.name, "My Patch");
        assert_eq!(ls.common.reverb_depth, 0x14); // from default
        assert_eq!(ls.parts[0].filter_cutoff, 100); // overridden
        assert_eq!(ls.parts[0].filter_resonance, 0x40); // from default
        assert!(ls.parts[0].filter_switch); // from default
        ls.to_blocks().unwrap();
    }

    #[test]
    fn partial_system_yaml_merges() {
        let s: System = serde_yaml::from_str("common:\n  master_tune: 1024\n").unwrap();
        assert!(s.common.local_control); // default true, not bool::default false
        assert_eq!(s.common.output_gain, 0x3E);
    }
}
