//! One-stop metadata bundle for tools and LLMs.
//!
//! Combines everything a generator needs to author a preset *by name* over a
//! sensible baseline: the field metadata ([`crate::params`]), the value
//! catalogs (voices / effects / EQ frequencies / control changes / assign
//! targets / rotary specs), and the factory [`System`]/[`LiveSet`] defaults.
//! The JSON Schemas are emitted separately (`ck schema …`).

use serde::Serialize;

use crate::document::{LiveSet, System};
use crate::{cc, effects, eq, params, rotary, voices};

/// A voice with its absolute number, name, and category.
#[derive(Serialize)]
pub struct VoiceEntry {
    pub number: u16,
    pub name: &'static str,
    pub category: &'static str,
}

/// The full metadata bundle.
#[derive(Serialize)]
pub struct Catalog {
    pub params: &'static [params::ParamMeta],
    pub voices: Vec<VoiceEntry>,
    pub part_effects: &'static [effects::EffectInfo],
    pub ad_effects: &'static [effects::EffectInfo],
    pub eq_frequencies: &'static [eq::EqFreq],
    pub control_changes: &'static [cc::CcInfo],
    pub assign_targets: Vec<cc::AssignTarget>,
    pub rotary_specs: &'static [rotary::RotaryParamSpec],
    pub default_system: System,
    pub default_live_set: LiveSet,
}

/// Build the metadata bundle.
pub fn catalog() -> Catalog {
    let voices = voices::VOICE_NAMES
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            let number = i as u16;
            VoiceEntry {
                number,
                name,
                category: voices::category_of(number).map(|c| c.name()).unwrap_or(""),
            }
        })
        .collect();
    Catalog {
        params: params::PARAMS,
        voices,
        part_effects: effects::PART_EFFECTS,
        ad_effects: effects::AD_EFFECTS,
        eq_frequencies: eq::EQ_FREQUENCIES,
        control_changes: cc::CONTROL_CHANGES,
        assign_targets: cc::assignable_targets(),
        rotary_specs: rotary::ROTARY_SPECS,
        default_system: System::default(),
        default_live_set: LiveSet::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_is_serializable_and_populated() {
        let c = catalog();
        assert_eq!(c.voices.len(), 363);
        assert_eq!(c.voices[13].name, "78Rd");
        assert_eq!(c.voices[13].category, "E.Piano");
        assert_eq!(c.default_live_set.parts.len(), 3);
        // Serializes cleanly (the LLM/CLI path).
        let out = serde_yaml::to_string(&c).unwrap();
        assert!(out.contains("default_live_set"));
        assert!(out.contains("Hall Reverb"));
    }
}
