//! Name ↔ value resolution for catalog-backed fields.
//!
//! Editor-/LLM-authored presets are friendlier in names ("Hall Reverb",
//! "78Rd", "2.0 kHz") than raw indices. The canonical typed model stays numeric
//! (so the editor and byte codec are unaffected); this module is a translation
//! layer that turns names into numbers on the way in. Which fields are
//! catalog-backed — and which catalog — is declared by [`crate::params`]'s
//! `catalog` hint, so this stays in sync with the field metadata.

use serde_yaml::Value;

use crate::codec::CodecError;
use crate::{cc, effects, eq, params, voices};

/// Resolve a value *name* in a named catalog to its numeric value.
///
/// Catalogs: `"voices"`, `"part_effects"`, `"ad_effects"`, `"eq_freq"`,
/// `"assign_target"`. Case-insensitive. `None` if the name isn't found.
pub fn resolve_name(catalog: &str, name: &str) -> Option<i64> {
    let ci = |a: &str| a.eq_ignore_ascii_case(name);
    match catalog {
        "voices" => voices::VOICE_NAMES
            .iter()
            .position(|n| ci(n))
            .map(|i| i as i64),
        "part_effects" => effects::PART_EFFECTS
            .iter()
            .find(|e| ci(e.name))
            .map(|e| e.number as i64),
        "ad_effects" => effects::AD_EFFECTS
            .iter()
            .find(|e| ci(e.name))
            .map(|e| e.number as i64),
        "eq_freq" => eq::EQ_FREQUENCIES
            .iter()
            .find(|f| ci(f.label))
            .map(|f| f.index as i64),
        "assign_target" => cc::assignable_targets()
            .into_iter()
            .find(|t| ci(&t.name))
            .map(|t| t.value as i64),
        _ => None,
    }
}

/// Label a numeric value via a catalog (inverse of [`resolve_name`]).
pub fn label_value(catalog: &str, value: i64) -> Option<String> {
    let b = u8::try_from(value).ok();
    match catalog {
        "voices" => u16::try_from(value)
            .ok()
            .and_then(voices::voice_name)
            .map(str::to_string),
        "part_effects" => b.and_then(effects::part_effect_name).map(str::to_string),
        "ad_effects" => b.and_then(effects::ad_effect_name).map(str::to_string),
        "eq_freq" => b.and_then(eq::eq_freq_label).map(str::to_string),
        "assign_target" => b.and_then(cc::assign_target_name),
        _ => None,
    }
}

/// Replace a string value (or each string in a sequence — for `category_voices`)
/// with its resolved number. Numbers and unknown names are left untouched.
fn resolve_value(v: &mut Value, catalog: &str) {
    match v {
        Value::String(s) => {
            if let Some(n) = resolve_name(catalog, s) {
                *v = Value::Number(n.into());
            }
        }
        Value::Sequence(seq) => {
            for e in seq.iter_mut() {
                resolve_value(e, catalog);
            }
        }
        _ => {}
    }
}

/// Resolve every catalog-backed field of one area object (`area` = e.g.
/// `"live_set.part"`).
fn resolve_obj(obj: Option<&mut Value>, area: &str) {
    let Some(Value::Mapping(map)) = obj else {
        return;
    };
    for (k, v) in map.iter_mut() {
        if let Some(key) = k.as_str() {
            if let Some(meta) = params::field_meta(&format!("{area}.{key}")) {
                if let Some(catalog) = meta.catalog {
                    resolve_value(v, catalog);
                }
            }
        }
    }
}

/// Resolve each element of an array of area objects (zones, parts).
fn resolve_seq(obj: Option<&mut Value>, area: &str) {
    if let Some(Value::Sequence(seq)) = obj {
        for item in seq.iter_mut() {
            resolve_obj(Some(item), area);
        }
    }
}

/// Convert any name strings in a (possibly partial) **Live Set** document to
/// their numeric values, returning normalized YAML ready for the codec. Accepts
/// YAML or JSON input (YAML is a superset). Numbers pass through unchanged.
pub fn resolve_names_live_set(input: &str) -> Result<String, CodecError> {
    let mut v: Value = serde_yaml::from_str(input).map_err(|e| CodecError::Yaml(e.to_string()))?;
    resolve_obj(v.get_mut("common"), "live_set.common");
    resolve_obj(v.get_mut("eq"), "live_set.eq");
    resolve_obj(v.get_mut("rotary"), "live_set.rotary");
    resolve_seq(v.get_mut("zones"), "live_set.zone");
    resolve_seq(v.get_mut("parts"), "live_set.part");
    serde_yaml::to_string(&v).map_err(|e| CodecError::Yaml(e.to_string()))
}

/// Convert any name strings in a (possibly partial) **System** document to their
/// numeric values, returning normalized YAML.
pub fn resolve_names_system(input: &str) -> Result<String, CodecError> {
    let mut v: Value = serde_yaml::from_str(input).map_err(|e| CodecError::Yaml(e.to_string()))?;
    resolve_obj(v.get_mut("common"), "system.common");
    resolve_obj(v.get_mut("master_eq"), "system.master_eq");
    serde_yaml::to_string(&v).map_err(|e| CodecError::Yaml(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LiveSet;

    #[test]
    fn name_primitives() {
        assert_eq!(resolve_name("part_effects", "Hall Reverb"), Some(0x1B));
        assert_eq!(resolve_name("voices", "78Rd"), Some(13));
        assert_eq!(resolve_name("eq_freq", "2.0 kHz"), Some(0x28));
        assert_eq!(resolve_name("assign_target", "Sustain"), Some(64));
        assert_eq!(resolve_name("assign_target", "USB Audio Volume"), Some(120));
        assert_eq!(resolve_name("part_effects", "nope"), None);
        assert_eq!(
            label_value("part_effects", 0x1B).as_deref(),
            Some("Hall Reverb")
        );
        assert_eq!(label_value("voices", 13).as_deref(), Some("78Rd"));
    }

    #[test]
    fn resolves_names_in_partial_live_set() {
        let yaml = "parts:\n- effect_1_type: Hall Reverb\n  category_voices: [CFX Stereo, 78Rd]\n";
        let out = resolve_names_live_set(yaml).unwrap();
        // The normalized YAML must now parse straight into the typed model.
        let ls: LiveSet = serde_yaml::from_str(&out).unwrap();
        assert_eq!(ls.parts[0].effect_1_type, 0x1B);
        assert_eq!(ls.parts[0].category_voices[0], 0);
        assert_eq!(ls.parts[0].category_voices[1], 13);
    }

    #[test]
    fn numbers_pass_through() {
        let yaml = "parts:\n- effect_1_type: 25\n";
        let out = resolve_names_live_set(yaml).unwrap();
        let ls: LiveSet = serde_yaml::from_str(&out).unwrap();
        assert_eq!(ls.parts[0].effect_1_type, 25);
    }
}
