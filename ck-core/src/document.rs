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

use serde::{de, Deserialize, Deserializer, Serialize};

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
    /// The four Zones (master-keyboard transmit configs). A preset may supply
    /// fewer (or partial) slots: each is merged over that slot's factory default
    /// and missing trailing slots are filled from it.
    #[serde(deserialize_with = "deser_zones")]
    pub zones: Vec<Zone>,
    /// The three Parts (A, B, C). A preset may supply fewer (or partial) slots:
    /// each is merged over that slot's factory default and missing trailing
    /// slots are filled from it.
    #[serde(deserialize_with = "deser_parts")]
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
        Self {
            common: LiveSetCommon::default(),
            eq: LiveSetEq::default(),
            audio_trigger_path: String::new(),
            zones: (0..Self::ZONES).map(factory_zone).collect(),
            parts: (0..Self::PARTS).map(factory_part).collect(),
            rotary: Some(RotarySpeaker::default()),
            extra_blocks: Vec::new(),
        }
    }
}

/// Factory default for one Zone slot (Zone 1 enabled; ascending transmit
/// channels). Shared by [`LiveSet::default`] and the padding deserializer.
fn factory_zone(i: usize) -> Zone {
    Zone {
        zone_switch: i == 0,
        transmit_channel: i as u8,
        ..Zone::default()
    }
}

/// Factory default for one Part slot (Part A enabled+selected; per-slot colours).
/// Shared by [`LiveSet::default`] and the padding deserializer.
fn factory_part(i: usize) -> Part {
    const COLORS: [u8; LiveSet::PARTS] = [0x02, 0x08, 0x04];
    Part {
        part_switch: i == 0,
        part_selected: i == 0,
        part_color: COLORS.get(i).copied().unwrap_or(0),
        ..Part::default()
    }
}

/// Deep-merge `overlay` onto `base`: mapping keys present in `overlay` win
/// (recursing into nested mappings); every other value replaces `base` wholesale
/// (so a supplied array like `category_voices` overrides, never element-merges).
fn merge_over(base: serde_yaml::Value, overlay: serde_yaml::Value) -> serde_yaml::Value {
    use serde_yaml::Value::Mapping;
    match (base, overlay) {
        (Mapping(mut b), Mapping(o)) => {
            for (k, ov) in o {
                let merged = match b.remove(&k) {
                    Some(bv) => merge_over(bv, ov),
                    None => ov,
                };
                b.insert(k, merged);
            }
            Mapping(b)
        }
        (_, overlay) => overlay,
    }
}

/// Build the canonical-count vector for one area: each slot `i` is the supplied
/// element (if any) merged over `factory(i)`, and missing trailing slots are the
/// bare factory slot. So a partial preset need only name the slots — and the
/// fields within them — that it changes; everything else fills from the factory
/// default for *that* slot. Rejects more elements than `count`.
fn slots<'de, D, T>(
    d: D,
    count: usize,
    label: &str,
    factory: impl Fn(usize) -> T,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Serialize + serde::de::DeserializeOwned,
{
    let raw = Vec::<serde_yaml::Value>::deserialize(d)?;
    if raw.len() > count {
        return Err(de::Error::custom(format!(
            "at most {count} {label}, got {}",
            raw.len()
        )));
    }
    (0..count)
        .map(|i| {
            let base = serde_yaml::to_value(factory(i)).map_err(de::Error::custom)?;
            let v = match raw.get(i) {
                Some(overlay) => merge_over(base, overlay.clone()),
                None => base,
            };
            serde_yaml::from_value(v).map_err(de::Error::custom)
        })
        .collect()
}

/// Deserialize `zones`: per-slot merge over [`factory_zone`], padded to
/// [`LiveSet::ZONES`].
fn deser_zones<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Zone>, D::Error> {
    slots(d, LiveSet::ZONES, "zones", factory_zone)
}

/// Deserialize `parts`: per-slot merge over [`factory_part`], padded to
/// [`LiveSet::PARTS`].
fn deser_parts<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Part>, D::Error> {
    slots(d, LiveSet::PARTS, "parts", factory_part)
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
        // Sparse Live Set: only a name and one part's cutoff. The single part is
        // padded to the full count and the rest fills from the factory default.
        let yaml = "common:\n  name: My Patch\nparts:\n- filter_cutoff: 100\n";
        let ls: LiveSet = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(ls.common.name, "My Patch");
        assert_eq!(ls.common.reverb_depth, 0x14); // from default
        assert_eq!(ls.parts[0].filter_cutoff, 100); // overridden
        assert_eq!(ls.parts[0].filter_resonance, 0x40); // from factory slot 0
        assert!(ls.parts[0].filter_switch); // from factory slot 0
        ls.to_blocks().unwrap();
    }

    #[test]
    fn mentioned_slot_inherits_factory_slot_defaults() {
        // A part that only sets a voice still comes up switched on, because the
        // mentioned slot 0 merges over factory_part(0) (Part A on/selected), not
        // over the bare Part::default (off). Explicit fields still win.
        let yaml = "parts:\n- current_category: 1\n- part_switch: true\n";
        let ls: LiveSet = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(ls.parts[0].current_category, 1); // explicit
        assert!(ls.parts[0].part_switch); // from factory slot 0
        assert!(ls.parts[0].part_selected); // from factory slot 0
        assert_eq!(ls.parts[0].part_color, 0x02); // factory slot 0 colour
        assert!(ls.parts[1].part_switch); // explicit override on slot 1
        assert_eq!(ls.parts[1].part_color, 0x08); // factory slot 1 colour
                                                  // Zone 1 still comes up enabled when mentioned-but-empty.
        let ls2: LiveSet = serde_yaml::from_str("zones:\n- {}\n").unwrap();
        assert!(ls2.zones[0].zone_switch);
    }

    #[test]
    fn short_arrays_pad_to_full_count() {
        // One part, two zones supplied → padded to PARTS / ZONES with factory
        // slots, then fully encodable.
        let yaml = "parts:\n- {}\nzones:\n- {}\n- {}\n";
        let ls: LiveSet = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(ls.parts.len(), LiveSet::PARTS);
        assert_eq!(ls.zones.len(), LiveSet::ZONES);
        // Padded (unmentioned) slots get factory per-slot defaults.
        assert_eq!(ls.parts[1].part_color, 0x08);
        assert_eq!(ls.parts[2].part_color, 0x04);
        assert!(!ls.parts[2].part_switch);
        assert_eq!(ls.zones[2].transmit_channel, 2); // padded slot
        assert_eq!(ls.zones[3].transmit_channel, 3);
        assert!(!ls.zones[3].zone_switch);
        ls.to_blocks().unwrap();
    }

    #[test]
    fn empty_arrays_pad_to_full_count() {
        let ls: LiveSet = serde_yaml::from_str("parts: []\nzones: []\n").unwrap();
        assert_eq!(ls.parts.len(), LiveSet::PARTS);
        assert_eq!(ls.zones.len(), LiveSet::ZONES);
        ls.to_blocks().unwrap();
    }

    #[test]
    fn too_many_parts_is_rejected() {
        let err = serde_yaml::from_str::<LiveSet>("parts: [{}, {}, {}, {}]\n");
        assert!(err.is_err());
    }

    #[test]
    fn complete_live_set_round_trips_unchanged() {
        // The per-slot merge must be identity for a fully-specified document:
        // serialize the factory default, deserialize it back, expect equality.
        let ls = LiveSet::default();
        let yaml = serde_yaml::to_string(&ls).unwrap();
        let back: LiveSet = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(ls, back);
    }

    #[test]
    fn partial_system_yaml_merges() {
        let s: System = serde_yaml::from_str("common:\n  master_tune: 1024\n").unwrap();
        assert!(s.common.local_control); // default true, not bool::default false
        assert_eq!(s.common.output_gain, 0x3E);
    }
}
