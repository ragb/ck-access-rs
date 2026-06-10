//! Insert-effect / A/D-input effect type catalog.
//!
//! [`crate::part::Part`]'s `effect_1_type` / `effect_2_type` and
//! [`crate::live_set::LiveSetCommon`]'s `ad_effect_1_type` / `ad_effect_2_type`
//! store an effect algorithm as a bare byte. This module turns that byte into
//! the panel name (e.g. "VCM Flanger", "Hall Reverb") so the editor can render
//! type pickers.
//!
//! Transcribed from the *Owner's Manual* "Data List" effect-type footnotes
//! (`*2` for the Part list, `*1` for the A/D list), verified by three
//! independent reads of the manual PDF. The **Part** list has 36 entries
//! (`0x00..=0x23`). The **A/D** list has 34 entries: the Part list without
//! **"Auto Wah"** and without the voice-only **"Damper Resonance"**.
//!
//! Note: the manual's A/D *range column* reads `0x00..=0x22` (35) but its `*1`
//! footnote lists only 34 names; we follow the footnote and treat `0x22` as an
//! unnamed/undocumented value ([`ad_effect_name`] returns `None`). The
//! `LiveSetCommon` decoder still accepts the byte so device data round-trips.
//! Not yet device-verified.

/// Description of one effect algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct EffectInfo {
    pub number: u8,
    /// Human label matching the panel, e.g. `"VCM Stereo Phaser"`.
    pub name: &'static str,
}

const fn fx(number: u8, name: &'static str) -> EffectInfo {
    EffectInfo { number, name }
}

/// Part insert-effect algorithms (`Part.effect_1_type` / `effect_2_type`),
/// `0x00..=0x23`.
pub static PART_EFFECTS: &[EffectInfo] = &[
    fx(0x00, "G Chorus"),
    fx(0x01, "SPX Chorus"),
    fx(0x02, "Symphonic"),
    fx(0x03, "816 Chorus"),
    fx(0x04, "VCM Flanger"),
    fx(0x05, "Cross FB Flanger"),
    fx(0x06, "VCM Stereo Phaser"),
    fx(0x07, "Small Phaser"),
    fx(0x08, "Max90"),
    fx(0x09, "Dual Phaser"),
    fx(0x0A, "Tremolo"),
    fx(0x0B, "Auto Pan"),
    fx(0x0C, "Simple Rotary"),
    fx(0x0D, "British Combo"),
    fx(0x0E, "British Lead"),
    fx(0x0F, "Small Stereo"),
    fx(0x10, "Compressor"),
    fx(0x11, "Tone Control"),
    fx(0x12, "1 BandEQ Narrow"),
    fx(0x13, "1 BandEQ Wide"),
    fx(0x14, "Auto Wah"),
    fx(0x15, "Touch Wah"),
    fx(0x16, "Pedal Wah"),
    fx(0x17, "Cross Delay"),
    fx(0x18, "Digital Delay"),
    fx(0x19, "Analog Delay"),
    fx(0x1A, "Room Reverb"),
    fx(0x1B, "Hall Reverb"),
    fx(0x1C, "Reverse Reverb"),
    fx(0x1D, "Ring Modulator"),
    fx(0x1E, "Slicer"),
    fx(0x1F, "LP Filter"),
    fx(0x20, "HP Filter"),
    fx(0x21, "Lo-Fi"),
    fx(0x22, "Damper Resonance"),
    fx(0x23, "Harmonic Enhancer"),
];

/// A/D-input effect algorithms (`LiveSetCommon.ad_effect_1_type` /
/// `ad_effect_2_type`), `0x00..=0x21` (34 entries). Same as [`PART_EFFECTS`]
/// without "Auto Wah" and the voice-only "Damper Resonance".
pub static AD_EFFECTS: &[EffectInfo] = &[
    fx(0x00, "G Chorus"),
    fx(0x01, "SPX Chorus"),
    fx(0x02, "Symphonic"),
    fx(0x03, "816 Chorus"),
    fx(0x04, "VCM Flanger"),
    fx(0x05, "Cross FB Flanger"),
    fx(0x06, "VCM Stereo Phaser"),
    fx(0x07, "Small Phaser"),
    fx(0x08, "Max90"),
    fx(0x09, "Dual Phaser"),
    fx(0x0A, "Tremolo"),
    fx(0x0B, "Auto Pan"),
    fx(0x0C, "Simple Rotary"),
    fx(0x0D, "British Combo"),
    fx(0x0E, "British Lead"),
    fx(0x0F, "Small Stereo"),
    fx(0x10, "Compressor"),
    fx(0x11, "Tone Control"),
    fx(0x12, "1 BandEQ Narrow"),
    fx(0x13, "1 BandEQ Wide"),
    fx(0x14, "Touch Wah"),
    fx(0x15, "Pedal Wah"),
    fx(0x16, "Cross Delay"),
    fx(0x17, "Digital Delay"),
    fx(0x18, "Analog Delay"),
    fx(0x19, "Room Reverb"),
    fx(0x1A, "Hall Reverb"),
    fx(0x1B, "Reverse Reverb"),
    fx(0x1C, "Ring Modulator"),
    fx(0x1D, "Slicer"),
    fx(0x1E, "LP Filter"),
    fx(0x1F, "HP Filter"),
    fx(0x20, "Lo-Fi"),
    fx(0x21, "Harmonic Enhancer"),
];

/// Name of a Part insert-effect type (`0x00..=0x23`), or `None` if out of range.
pub fn part_effect_name(number: u8) -> Option<&'static str> {
    PART_EFFECTS.get(number as usize).map(|e| e.name)
}

/// Name of an A/D-input effect type (`0x00..=0x22`), or `None` if out of range.
pub fn ad_effect_name(number: u8) -> Option<&'static str> {
    AD_EFFECTS.get(number as usize).map(|e| e.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lengths_match_documented_ranges() {
        assert_eq!(PART_EFFECTS.len(), 0x24); // 0x00..=0x23 (36)
        assert_eq!(AD_EFFECTS.len(), 0x22); // 0x00..=0x21 (34)
    }

    #[test]
    fn landmark_names() {
        assert_eq!(part_effect_name(0x00), Some("G Chorus"));
        assert_eq!(part_effect_name(0x14), Some("Auto Wah"));
        assert_eq!(part_effect_name(0x22), Some("Damper Resonance"));
        assert_eq!(part_effect_name(0x23), Some("Harmonic Enhancer"));
        assert_eq!(part_effect_name(0x24), None);
        assert_eq!(ad_effect_name(0x00), Some("G Chorus"));
        assert_eq!(ad_effect_name(0x14), Some("Touch Wah")); // no "Auto Wah" in A/D
        assert_eq!(ad_effect_name(0x21), Some("Harmonic Enhancer"));
        assert_eq!(ad_effect_name(0x22), None); // range says 0x22 but footnote stops at 0x21
    }

    #[test]
    fn ad_is_part_without_auto_wah_and_damper_resonance() {
        let ad: Vec<&str> = AD_EFFECTS.iter().map(|e| e.name).collect();
        let part_minus: Vec<&str> = PART_EFFECTS
            .iter()
            .map(|e| e.name)
            .filter(|&n| n != "Auto Wah" && n != "Damper Resonance")
            .collect();
        assert_eq!(ad, part_minus);
    }

    #[test]
    fn numbers_are_dense_and_ordered() {
        for (i, e) in PART_EFFECTS.iter().enumerate() {
            assert_eq!(e.number as usize, i);
        }
        for (i, e) in AD_EFFECTS.iter().enumerate() {
            assert_eq!(e.number as usize, i);
        }
    }

    #[test]
    fn names_unique_within_each_list() {
        for list in [PART_EFFECTS, AD_EFFECTS] {
            let mut seen = std::collections::HashSet::new();
            for e in list {
                assert!(seen.insert(e.name), "duplicate effect name: {}", e.name);
            }
        }
    }
}
