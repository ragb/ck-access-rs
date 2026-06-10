//! Control Change descriptions for the CK's fixed CC map.
//!
//! The CK assigns a fixed function to each Control Change number (per Part
//! A/B/C, plus global controls). Transcribed from the *Owner's Manual* "Data
//! List → Control Change Number" table. The editor uses this so CC assignments
//! (Modulation Wheel / Foot Pedal destinations, incoming-CC labels) show names
//! instead of bare numbers.
//!
//! The assignable controllers ([`crate::live_set::LiveSetCommon`]'s
//! `mod_wheel_assign` / `foot_pedal_1_assign` / `foot_pedal_2_assign`) take a
//! value `0..=120`: `0..=119` selects the CC of that number, and `120` is the
//! special **USB Audio Volume** destination. [`assign_target_name`] /
//! [`assignable_targets`] turn those into labels.
//!
//! (CC 10 is Pan, per the manual. The per-part pan added in v1.10 is a stored
//! setting — [`crate::part::Part::pan`] — not a Control Change.)

/// Description of one Control Change number on the CK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct CcInfo {
    pub number: u8,
    /// Human label, e.g. `"A: Cutoff"`, `"Modulation"`, `"Sustain"`.
    pub name: &'static str,
    /// `false` for CCs the manual lists in parentheses — received/transmitted
    /// but with no effect on the internal tone generator (Pan, Portamento Time,
    /// RPN/NRPN, …).
    pub affects_tone_generator: bool,
}

/// Assign value that selects the special **USB Audio Volume** destination.
pub const USB_AUDIO_VOLUME: u8 = 120;

const fn cc(number: u8, name: &'static str, affects_tone_generator: bool) -> CcInfo {
    CcInfo {
        number,
        name,
        affects_tone_generator,
    }
}

/// Every documented Control Change, in ascending number order.
pub static CONTROL_CHANGES: &[CcInfo] = &[
    cc(0, "Bank Select MSB", true),
    cc(1, "Modulation", true),
    cc(4, "Pedal Wah", true),
    cc(5, "Portamento Time", false),
    cc(6, "Data Entry MSB", false),
    cc(7, "All Volume", true),
    cc(9, "Rotary Slow/Fast", true),
    cc(10, "Pan", false),
    cc(11, "Expression", true),
    cc(12, "A: Volume", true),
    cc(13, "A: Drive Depth", true),
    cc(14, "A: Effect1 Depth", true),
    cc(15, "A: Effect1 Rate", true),
    cc(16, "A: Effect2 Depth", true),
    cc(17, "A: Effect2 Rate", true),
    cc(18, "A: Drawbar 16'", true),
    cc(19, "A: Drawbar 5 1/3'", true),
    cc(20, "A: Drawbar 8'", true),
    cc(21, "A: Drawbar 4'", true),
    cc(22, "A: Drawbar 2 2/3'", true),
    cc(23, "A: Drawbar 2'", true),
    cc(24, "A: Drawbar 1 3/5'", true),
    cc(25, "A: Drawbar 1 1/3'", true),
    cc(26, "A: Drawbar 1'", true),
    cc(27, "B: Volume", true),
    cc(28, "B: Attack", true),
    cc(29, "B: Release", true),
    cc(30, "B: Cutoff", true),
    cc(31, "B: Resonance", true),
    cc(32, "Bank Select LSB", false),
    cc(38, "Data Entry LSB", false),
    cc(64, "Sustain", true),
    cc(65, "Portamento", false),
    cc(66, "Sostenuto", true),
    cc(67, "Soft", true),
    cc(68, "B: Drive Depth", true),
    cc(69, "B: Effect1 Depth", true),
    cc(70, "B: Effect1 Rate", true),
    cc(71, "A: Resonance", true),
    cc(72, "A: Release", true),
    cc(73, "A: Attack", true),
    cc(74, "A: Cutoff", true),
    cc(75, "B: Effect2 Depth", true),
    cc(76, "B: Effect2 Rate", true),
    cc(77, "B: Drawbar 16'", true),
    cc(78, "B: Drawbar 5 1/3'", true),
    cc(79, "B: Drawbar 8'", true),
    cc(80, "B: Drawbar 4'", true),
    cc(81, "B: Drawbar 2 2/3'", true),
    cc(82, "B: Drawbar 2'", true),
    cc(83, "B: Drawbar 1 3/5'", true),
    cc(84, "Portamento Control", false),
    cc(85, "B: Drawbar 1 1/3'", true),
    cc(86, "B: Drawbar 1'", true),
    cc(87, "C: Volume", true),
    cc(88, "Equalizer High", true),
    cc(89, "Equalizer Mid", true),
    cc(90, "Equalizer Low", true),
    cc(91, "Reverb Depth", true),
    cc(92, "Delay Time", true),
    cc(93, "Delay Depth", true),
    cc(95, "Effect5 Depth", false),
    cc(96, "Data Increment", false),
    cc(97, "Data Decrement", false),
    cc(98, "NRPN LSB", false),
    cc(99, "NRPN MSB", false),
    cc(100, "RPN LSB", false),
    cc(101, "RPN MSB", false),
    cc(102, "C: Attack", true),
    cc(103, "C: Release", true),
    cc(104, "C: Cutoff", true),
    cc(105, "C: Resonance", true),
    cc(106, "C: Drive Depth", true),
    cc(107, "C: Effect1 Depth", true),
    cc(108, "C: Effect1 Rate", true),
    cc(109, "C: Effect2 Depth", true),
    cc(110, "C: Effect2 Rate", true),
    cc(111, "C: Drawbar 16'", true),
    cc(112, "C: Drawbar 5 1/3'", true),
    cc(113, "C: Drawbar 8'", true),
    cc(114, "C: Drawbar 4'", true),
    cc(115, "C: Drawbar 2 2/3'", true),
    cc(116, "C: Drawbar 2'", true),
    cc(117, "C: Drawbar 1 3/5'", true),
    cc(118, "C: Drawbar 1 1/3'", true),
    cc(119, "C: Drawbar 1'", true),
];

/// Full description of a Control Change number, if the CK documents one.
pub fn cc_info(number: u8) -> Option<&'static CcInfo> {
    CONTROL_CHANGES.iter().find(|c| c.number == number)
}

/// Human label for a Control Change number (`None` if undocumented).
pub fn cc_name(number: u8) -> Option<&'static str> {
    cc_info(number).map(|c| c.name)
}

/// A destination a controller (Mod Wheel / Foot Pedal) can be assigned to.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AssignTarget {
    /// The stored assign value (`0..=119` = that CC number, `120` = USB audio).
    pub value: u8,
    pub name: String,
    pub affects_tone_generator: bool,
}

/// Label for a controller assign value (`0..=120`).
///
/// `0..=119` resolve to the CC's name (or `"CC <n>"` if that number isn't in the
/// CK's table); `120` is `"USB Audio Volume"`. Returns `None` above `120`.
pub fn assign_target_name(value: u8) -> Option<String> {
    match value {
        USB_AUDIO_VOLUME => Some("USB Audio Volume".to_string()),
        0..=119 => Some(
            cc_name(value)
                .map(str::to_string)
                .unwrap_or_else(|| format!("CC {value}")),
        ),
        _ => None,
    }
}

/// The list of named assign destinations for an editor dropdown: every
/// documented CC in `0..=119` plus USB Audio Volume.
pub fn assignable_targets() -> Vec<AssignTarget> {
    let mut targets: Vec<AssignTarget> = CONTROL_CHANGES
        .iter()
        .filter(|c| c.number <= 119)
        .map(|c| AssignTarget {
            value: c.number,
            name: c.name.to_string(),
            affects_tone_generator: c.affects_tone_generator,
        })
        .collect();
    targets.push(AssignTarget {
        value: USB_AUDIO_VOLUME,
        name: "USB Audio Volume".to_string(),
        affects_tone_generator: false,
    });
    targets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_ccs_resolve() {
        assert_eq!(cc_name(1), Some("Modulation"));
        assert_eq!(cc_name(64), Some("Sustain"));
        assert_eq!(cc_name(74), Some("A: Cutoff"));
        assert_eq!(cc_name(119), Some("C: Drawbar 1'"));
        assert_eq!(cc_name(2), None); // not documented
    }

    #[test]
    fn parenthesised_ccs_flagged() {
        assert!(!cc_info(10).unwrap().affects_tone_generator); // Pan
        assert!(cc_info(1).unwrap().affects_tone_generator); // Modulation
    }

    #[test]
    fn assign_defaults_have_names() {
        // Live Set Common defaults: mod wheel = 1, foot pedal 1 = 64, FP2 = 11.
        assert_eq!(assign_target_name(1).as_deref(), Some("Modulation"));
        assert_eq!(assign_target_name(64).as_deref(), Some("Sustain"));
        assert_eq!(assign_target_name(11).as_deref(), Some("Expression"));
        assert_eq!(assign_target_name(120).as_deref(), Some("USB Audio Volume"));
        assert_eq!(assign_target_name(3).as_deref(), Some("CC 3")); // undocumented but valid
        assert_eq!(assign_target_name(121), None);
    }

    #[test]
    fn numbers_are_sorted_and_unique() {
        for w in CONTROL_CHANGES.windows(2) {
            assert!(w[0].number < w[1].number, "CCs must be ascending/unique");
        }
    }

    #[test]
    fn assignable_targets_includes_usb_audio() {
        let t = assignable_targets();
        assert_eq!(t.last().unwrap().value, USB_AUDIO_VOLUME);
        assert!(t.iter().any(|a| a.name == "A: Cutoff"));
    }
}
