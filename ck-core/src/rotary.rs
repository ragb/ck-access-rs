//! Live Set **Rotary Speaker** block (`46 20 00`, 24 bytes) — added in CK
//! firmware **v1.10**.
//!
//! The v1.10 supplementary manual documents the parameters but gives no SysEx
//! address table, so this layout was reverse-engineered by capture-and-diff on a
//! real CK88 (change a value, Store the Live Set, dump, diff). The block holds
//! two independent speaker models, **Rotary A** then **Rotary B**, each with
//! Balance, Stereo/Mono, four Speed settings and (A) four Acceleration settings
//! or (B) two Transition settings.
//!
//! Confidence: the **Balance** offsets (`0x00`/`0x0D`), **Stereo/Mono** toggles
//! (`0x01`/`0x0E`) and **Rotary B Transition** (`0x13`/`0x14`) are confirmed by
//! their v1.10 default values; the Speed/Acceleration offsets follow the manual's
//! documented parameter order. Any byte not named here (`0x0A..0x0C`, `0x15..`)
//! is preserved verbatim, so the block round-trips byte-exact regardless.

use serde::{Deserialize, Serialize};

use crate::address::ROTARY_LEN;
use crate::codec::{apply_reserved, byte_enum, capture_reserved, ranged, CodecError, RawByte};

byte_enum! {
    /// Rotary output mode (offsets 0x01 / 0x0E).
    StereoMono { Stereo = 0, Mono = 1 }
    valid = "0=stereo, 1=mono"
}

/// Centre balance byte: horn (treble) = rotor (bass).
pub const BALANCE_CENTER: u8 = 0x40;

/// Human label for a Rotary **Balance** byte, matching the panel notation
/// `R63>H – R=H – R<H63` (R = rotor/bass, H = horn/treble). Centre `0x40` is
/// `"R=H"`; bytes above centre tilt toward the horn (`"R<H{n}"`), below toward
/// the rotor (`"R{n}>H"`). Device-confirmed by the documented defaults
/// (`0x46` → `"R<H6"`, `0x50` → `"R<H16"`).
pub fn balance_label(byte: u8) -> String {
    let d = byte as i32 - BALANCE_CENTER as i32;
    match d.cmp(&0) {
        std::cmp::Ordering::Equal => "R=H".to_string(),
        std::cmp::Ordering::Greater => format!("R<H{d}"),
        std::cmp::Ordering::Less => format!("R{}>H", -d),
    }
}

/// The Rotary Speaker block (Rotary A + Rotary B).
///
/// Balance is a centred value (`0x40` = R=H — see [`balance_label`]); Stereo/Mono
/// is typed. Speed / Acceleration / Transition stay raw `0..=127` indices: the
/// manual documents only their rpm/ratio range + default (exposed via
/// [`ROTARY_SPECS`]), and the device's per-step curve isn't published or
/// derivable (Rotary A and B use different byte→rpm mappings), so no per-byte
/// conversion is provided. Transition is itself a raw `0..=127` value.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// Restores the per-field `"default":` entries in the emitted JSON Schema. Only
// under `schema`: `tsify` reads a container default as "every field optional" and
// would loosen the editor's TypeScript. Partial-document leniency lives at the
// document boundary instead (see `document::from_value_over_default`).
#[cfg_attr(feature = "schema", serde(default))]
pub struct RotarySpeaker {
    /// Rotary A balance (horn vs rotor), raw 0..=127 (R63>H .. R=H .. R<H63).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub a_balance: u8,
    /// Rotary A output: stereo or mono.
    pub a_stereo_mono: StereoMono,
    /// Rotary A horn slow speed index (23.0–89.6 rpm).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub a_horn_slow: u8,
    /// Rotary A rotor slow speed index (22.7–88.3 rpm).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub a_rotor_slow: u8,
    /// Rotary A horn fast speed index (209.4–817.6 rpm).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub a_horn_fast: u8,
    /// Rotary A rotor fast speed index (189.3–736.8 rpm).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub a_rotor_fast: u8,
    /// Rotary A horn acceleration index (0.21–2.00).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub a_horn_acceleration: u8,
    /// Rotary A rotor acceleration index (0.21–2.00).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub a_rotor_acceleration: u8,
    /// Rotary A horn deceleration index (0.21–2.00).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub a_horn_deceleration: u8,
    /// Rotary A rotor deceleration index (0.21–2.00).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub a_rotor_deceleration: u8,
    /// Rotary B balance, raw 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub b_balance: u8,
    /// Rotary B output: stereo or mono.
    pub b_stereo_mono: StereoMono,
    /// Rotary B horn slow speed index (2.5–159.0 rpm).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub b_horn_slow: u8,
    /// Rotary B rotor slow speed index (2.5–159.0 rpm).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub b_rotor_slow: u8,
    /// Rotary B horn fast speed index (161.5–2382 rpm).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub b_horn_fast: u8,
    /// Rotary B rotor fast speed index (161.5–2382 rpm).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub b_rotor_fast: u8,
    /// Rotary B horn transition, 0..=127 (default 118).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub b_horn_transition: u8,
    /// Rotary B rotor transition, 0..=127 (default 116).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub b_rotor_transition: u8,
    /// Reserved/undocumented bytes captured verbatim so writes round-trip exactly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reserved: Vec<RawByte>,
}

impl Default for RotarySpeaker {
    /// Factory defaults (from the v1.10 manual / captured factory block).
    fn default() -> Self {
        Self {
            a_balance: 0x46,
            a_stereo_mono: StereoMono::Stereo,
            a_horn_slow: 0x40,
            a_rotor_slow: 0x40,
            a_horn_fast: 0x48,
            a_rotor_fast: 0x48,
            a_horn_acceleration: 0x46,
            a_rotor_acceleration: 0x40,
            a_horn_deceleration: 0x40,
            a_rotor_deceleration: 0x40,
            b_balance: 0x50,
            b_stereo_mono: StereoMono::Stereo,
            b_horn_slow: 0x16,
            b_rotor_slow: 0x18,
            b_horn_fast: 0x5B,
            b_rotor_fast: 0x59,
            b_horn_transition: 0x76,
            b_rotor_transition: 0x74,
            reserved: Vec::new(),
        }
    }
}

impl RotarySpeaker {
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() < ROTARY_LEN {
            return Err(CodecError::WrongLength {
                expected: ROTARY_LEN,
                actual: b.len(),
            });
        }
        let mut value = Self {
            a_balance: ranged(b[0x00], 0x00, 0x7F, "a_balance")?,
            a_stereo_mono: StereoMono::from_byte(b[0x01])?,
            a_horn_slow: ranged(b[0x02], 0x00, 0x7F, "a_horn_slow")?,
            a_rotor_slow: ranged(b[0x03], 0x00, 0x7F, "a_rotor_slow")?,
            a_horn_fast: ranged(b[0x04], 0x00, 0x7F, "a_horn_fast")?,
            a_rotor_fast: ranged(b[0x05], 0x00, 0x7F, "a_rotor_fast")?,
            a_horn_acceleration: ranged(b[0x06], 0x00, 0x7F, "a_horn_acceleration")?,
            a_rotor_acceleration: ranged(b[0x07], 0x00, 0x7F, "a_rotor_acceleration")?,
            a_horn_deceleration: ranged(b[0x08], 0x00, 0x7F, "a_horn_deceleration")?,
            a_rotor_deceleration: ranged(b[0x09], 0x00, 0x7F, "a_rotor_deceleration")?,
            b_balance: ranged(b[0x0D], 0x00, 0x7F, "b_balance")?,
            b_stereo_mono: StereoMono::from_byte(b[0x0E])?,
            b_horn_slow: ranged(b[0x0F], 0x00, 0x7F, "b_horn_slow")?,
            b_rotor_slow: ranged(b[0x10], 0x00, 0x7F, "b_rotor_slow")?,
            b_horn_fast: ranged(b[0x11], 0x00, 0x7F, "b_horn_fast")?,
            b_rotor_fast: ranged(b[0x12], 0x00, 0x7F, "b_rotor_fast")?,
            b_horn_transition: ranged(b[0x13], 0x00, 0x7F, "b_horn_transition")?,
            b_rotor_transition: ranged(b[0x14], 0x00, 0x7F, "b_rotor_transition")?,
            reserved: Vec::new(),
        };
        let typed_only = value.to_bytes()?;
        value.reserved = capture_reserved(b, &typed_only);
        Ok(value)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        let mut b = vec![0u8; ROTARY_LEN];
        b[0x00] = ranged(self.a_balance, 0x00, 0x7F, "a_balance")?;
        b[0x01] = self.a_stereo_mono.to_byte();
        b[0x02] = ranged(self.a_horn_slow, 0x00, 0x7F, "a_horn_slow")?;
        b[0x03] = ranged(self.a_rotor_slow, 0x00, 0x7F, "a_rotor_slow")?;
        b[0x04] = ranged(self.a_horn_fast, 0x00, 0x7F, "a_horn_fast")?;
        b[0x05] = ranged(self.a_rotor_fast, 0x00, 0x7F, "a_rotor_fast")?;
        b[0x06] = ranged(self.a_horn_acceleration, 0x00, 0x7F, "a_horn_acceleration")?;
        b[0x07] = ranged(
            self.a_rotor_acceleration,
            0x00,
            0x7F,
            "a_rotor_acceleration",
        )?;
        b[0x08] = ranged(self.a_horn_deceleration, 0x00, 0x7F, "a_horn_deceleration")?;
        b[0x09] = ranged(
            self.a_rotor_deceleration,
            0x00,
            0x7F,
            "a_rotor_deceleration",
        )?;
        b[0x0D] = ranged(self.b_balance, 0x00, 0x7F, "b_balance")?;
        b[0x0E] = self.b_stereo_mono.to_byte();
        b[0x0F] = ranged(self.b_horn_slow, 0x00, 0x7F, "b_horn_slow")?;
        b[0x10] = ranged(self.b_rotor_slow, 0x00, 0x7F, "b_rotor_slow")?;
        b[0x11] = ranged(self.b_horn_fast, 0x00, 0x7F, "b_horn_fast")?;
        b[0x12] = ranged(self.b_rotor_fast, 0x00, 0x7F, "b_rotor_fast")?;
        b[0x13] = ranged(self.b_horn_transition, 0x00, 0x7F, "b_horn_transition")?;
        b[0x14] = ranged(self.b_rotor_transition, 0x00, 0x7F, "b_rotor_transition")?;
        apply_reserved(&mut b, &self.reserved);
        Ok(b)
    }
}

/// Documented engineering-unit range + default for a Rotary parameter, from the
/// CK v1.10 supplementary manual. The stored value is a raw `0..=127` index; the
/// device maps it to this unit via an internal (non-linear) curve that isn't
/// published, so these specs are for display (range/default) — like CK Editor —
/// rather than an exact byte↔unit conversion.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct RotaryParamSpec {
    /// Field name on [`RotarySpeaker`].
    pub key: &'static str,
    pub label: &'static str,
    /// `"rpm"`, `"ratio"`, or `""` (raw index).
    pub unit: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

/// Reference ranges/defaults for the Rotary speed/acceleration/transition params.
pub static ROTARY_SPECS: &[RotaryParamSpec] = &[
    spec("a_horn_slow", "Rotary A Horn Slow", "rpm", 23.0, 89.6, 45.4),
    spec(
        "a_rotor_slow",
        "Rotary A Rotor Slow",
        "rpm",
        22.7,
        88.3,
        44.8,
    ),
    spec(
        "a_horn_fast",
        "Rotary A Horn Fast",
        "rpm",
        209.4,
        817.6,
        454.2,
    ),
    spec(
        "a_rotor_fast",
        "Rotary A Rotor Fast",
        "rpm",
        189.3,
        736.8,
        413.8,
    ),
    spec(
        "a_horn_acceleration",
        "Rotary A Horn Acceleration",
        "ratio",
        0.21,
        2.00,
        1.10,
    ),
    spec(
        "a_rotor_acceleration",
        "Rotary A Rotor Acceleration",
        "ratio",
        0.21,
        2.00,
        1.00,
    ),
    spec(
        "a_horn_deceleration",
        "Rotary A Horn Deceleration",
        "ratio",
        0.21,
        2.00,
        1.00,
    ),
    spec(
        "a_rotor_deceleration",
        "Rotary A Rotor Deceleration",
        "ratio",
        0.21,
        2.00,
        1.00,
    ),
    spec("b_horn_slow", "Rotary B Horn Slow", "rpm", 2.5, 159.0, 55.5),
    spec(
        "b_rotor_slow",
        "Rotary B Rotor Slow",
        "rpm",
        2.5,
        159.0,
        60.5,
    ),
    spec(
        "b_horn_fast",
        "Rotary B Horn Fast",
        "rpm",
        161.5,
        2382.0,
        403.7,
    ),
    spec(
        "b_rotor_fast",
        "Rotary B Rotor Fast",
        "rpm",
        161.5,
        2382.0,
        363.4,
    ),
    spec(
        "b_horn_transition",
        "Rotary B Horn Transition",
        "",
        0.0,
        127.0,
        118.0,
    ),
    spec(
        "b_rotor_transition",
        "Rotary B Rotor Transition",
        "",
        0.0,
        127.0,
        116.0,
    ),
];

const fn spec(
    key: &'static str,
    label: &'static str,
    unit: &'static str,
    min: f32,
    max: f32,
    default: f32,
) -> RotaryParamSpec {
    RotaryParamSpec {
        key,
        label,
        unit,
        min,
        max,
        default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Rotary block from a real CK88 dump (factory defaults).
    const DEFAULTS: [u8; ROTARY_LEN] = [
        0x46, 0x00, 0x40, 0x40, 0x48, 0x48, 0x46, 0x40, 0x40, 0x40, 0x00, 0x00, 0x00, 0x50, 0x00,
        0x16, 0x18, 0x5B, 0x59, 0x76, 0x74, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn decodes_factory_defaults() {
        let r = RotarySpeaker::from_bytes(&DEFAULTS).unwrap();
        assert_eq!(r.a_balance, 0x46); // "R<H6"
        assert_eq!(r.a_stereo_mono, StereoMono::Stereo);
        assert_eq!(r.b_balance, 0x50); // "R<H16"
        assert_eq!(r.b_stereo_mono, StereoMono::Stereo);
        assert_eq!(r.b_horn_transition, 118);
        assert_eq!(r.b_rotor_transition, 116);
        assert_eq!(r.to_bytes().unwrap(), DEFAULTS);
    }

    #[test]
    fn specs_cover_all_speed_params() {
        assert_eq!(ROTARY_SPECS.len(), 14);
        let horn = ROTARY_SPECS
            .iter()
            .find(|s| s.key == "a_horn_slow")
            .unwrap();
        assert_eq!(horn.unit, "rpm");
        assert_eq!(horn.default, 45.4);
    }

    #[test]
    fn balance_labels_match_documented_defaults() {
        assert_eq!(balance_label(0x40), "R=H");
        assert_eq!(balance_label(0x46), "R<H6"); // Rotary A default
        assert_eq!(balance_label(0x50), "R<H16"); // Rotary B default
        assert_eq!(balance_label(0x01), "R63>H"); // extreme toward rotor
        assert_eq!(balance_label(0x7F), "R<H63"); // extreme toward horn
    }

    #[test]
    fn preserves_reserved_gap() {
        // The 0x0A..0x0C gap (Rotary A has no transition) is reserved; ensure a
        // non-zero value there round-trips.
        let mut bytes = DEFAULTS;
        bytes[0x0B] = 0x07;
        let r = RotarySpeaker::from_bytes(&bytes).unwrap();
        assert_eq!(r.to_bytes().unwrap(), bytes);
    }
}
