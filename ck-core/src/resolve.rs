//! Name ↔ value resolution for catalog-backed fields.
//!
//! Editor-/LLM-authored presets are friendlier in names ("Hall Reverb", "78Rd",
//! "2.0 kHz") than raw indices. The canonical typed model stays numeric (so the
//! editor and byte codec are unaffected); this module is a translation layer that
//! turns names into numbers on the way in.
//!
//! The CK value lookups ([`resolve_name`] / [`label_value`]) live here and back
//! the device's [`crate::catalog::CkCatalogs`]; the document *walk* (which fields
//! are catalog-backed, and descending the YAML) is the shared, generic one in
//! `midi_access_core::resolve`, driven by [`crate::params`]'s `catalog` hints.

use crate::codec::CodecError;
use crate::{cc, effects, eq, voices};

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

/// Every catalog this device exposes, in [`crate::catalog::CkCatalogs::as_value`]
/// order. The first five are name-resolvable (a `ParamMeta`'s `catalog` hint
/// points at one of them); `control_changes` and `rotary_specs` are browse-only
/// reference tables, for which [`resolve_name`] returns `None`.
///
/// These must stay in step with `as_value`'s keys — a tool that reads
/// `catalog: "eq_freq"` off a parameter looks that name up in the bundle.
pub const CATALOG_NAMES: &[&str] = &[
    "voices",
    "part_effects",
    "ad_effects",
    "eq_freq",
    "control_changes",
    "assign_target",
    "rotary_specs",
];

/// Convert any name strings in a (possibly partial) **Live Set** document to
/// their numeric values, returning normalized YAML ready for the codec. Accepts
/// YAML or JSON input (YAML is a superset). Numbers pass through unchanged.
pub fn resolve_names_live_set(input: &str) -> Result<String, CodecError> {
    resolve_doc(input)
}

/// Convert any name strings in a (possibly partial) **System** document to their
/// numeric values, returning normalized YAML.
pub fn resolve_names_system(input: &str) -> Result<String, CodecError> {
    resolve_doc(input)
}

/// Both documents resolve through the same generic walk: it rewrites every
/// catalog-hinted field wherever it appears, so System and Live Set need no
/// per-area handling.
fn resolve_doc(input: &str) -> Result<String, CodecError> {
    midi_access_core::resolve_names_str(
        input,
        crate::params::params(),
        &crate::catalog::CK_CATALOGS,
    )
    .map_err(|e| CodecError::Yaml(e.to_string()))
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
        let ls: LiveSet = crate::yaml::live_set_from_yaml_str(&out).unwrap();
        assert_eq!(ls.parts[0].effect_1_type, 0x1B);
        assert_eq!(ls.parts[0].category_voices[0], 0);
        assert_eq!(ls.parts[0].category_voices[1], 13);
    }

    #[test]
    fn numbers_pass_through() {
        let yaml = "parts:\n- effect_1_type: 25\n";
        let out = resolve_names_live_set(yaml).unwrap();
        let ls: LiveSet = crate::yaml::live_set_from_yaml_str(&out).unwrap();
        assert_eq!(ls.parts[0].effect_1_type, 25);
    }
}
