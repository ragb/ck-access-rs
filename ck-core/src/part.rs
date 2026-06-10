//! Live Set **Part** block (`50 0p 00`, p = 0..=2 for A/B/C, 105 bytes).
//!
//! A Part is one of the three layered/split sound engines. It carries the
//! per-category voice selection, the organ drawbars + percussion, and the
//! Filter / EG / Drive / Effect chain. Offsets/ranges from the *Owner's Manual*
//! "MIDI Data Table → Live Set Part".

use serde::{Deserialize, Serialize};

use crate::address::PART_LEN;
use crate::codec::{
    apply_reserved, bool_byte, byte_enum, capture_reserved, ranged, read_u14, signed_center,
    to_signed_center, write_u14, CodecError, RawByte,
};

/// Number of sound categories (each holds one selected voice number).
pub const CATEGORY_COUNT: usize = 10;

byte_enum! {
    /// Which insert-effect slot the part edits (offset 0x1B).
    EffectSelect { Effect1 = 0, Effect2 = 1 }
    valid = "0=effect1, 1=effect2"
}

byte_enum! {
    /// Voice allocation (offset 0x20).
    MonoPoly { Mono = 0, Poly = 1 }
    valid = "0=mono, 1=poly"
}

byte_enum! {
    /// Mono/portamento behaviour (offset 0x21).
    MonoType { Normal = 0, FingeredPortamento = 1, FullTimePortamento = 2 }
    valid = "0..=2"
}

byte_enum! {
    /// Portamento time interpretation (offset 0x24).
    PortamentoMode { Rate = 0, Time = 1 }
    valid = "0=rate, 1=time"
}

byte_enum! {
    /// Unison voicing (offset 0x2D).
    UnisonType { MultiLayer = 0, Harmonics = 1, SubHarmonics = 2 }
    valid = "0..=2"
}

byte_enum! {
    /// External-keyboard routing (offset 0x39).
    ExternalKeyboard { ExtInt = 0, ExtOnly = 1, Off = 2 }
    valid = "0..=2"
}

byte_enum! {
    /// Organ percussion harmonic (offset 0x4A).
    PercussionType { Third = 0, Second = 1 }
    valid = "0=third, 1=second"
}

byte_enum! {
    /// Organ percussion decay (offset 0x4B).
    PercussionDecay { Slow = 0, Fast = 1 }
    valid = "0=slow, 1=fast"
}

byte_enum! {
    /// Organ percussion volume (offset 0x4C).
    PercussionVolume { Normal = 0, Soft = 1 }
    valid = "0=normal, 1=soft"
}

byte_enum! {
    /// Vibrato/Chorus program (offset 0x4F).
    VibratoChorusType { V1 = 0, C1 = 1, V2 = 2, C2 = 3, V3 = 4, C3 = 5 }
    valid = "0..=5"
}

byte_enum! {
    /// Drive/amp model (offset 0x57).
    DriveType { OverDrive = 0, Distortion = 1, RotaryA = 2, RotaryB = 3, Comp = 4 }
    valid = "0..=4"
}

/// One Live Set Part.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Part {
    /// Active category, 0..=9 (indexes `category_voices`).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 9)))]
    pub current_category: u8,
    /// Selected voice number per category (10 entries). Each is the absolute
    /// voice number for that category — see [`crate::voices`].
    pub category_voices: Vec<u16>,
    /// Note shift, −24..=+24 semitones.
    #[cfg_attr(feature = "schema", schemars(range(min = -24, max = 24)))]
    pub note_shift: i8,
    /// Part volume, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub part_volume: u8,
    /// Part colour, 0..=11 (0=Red .. 11=Rose).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 11)))]
    pub part_color: u8,
    /// Turns this part on or off.
    pub part_switch: bool,
    /// Marks this part as the currently selected/edited part.
    pub part_selected: bool,
    /// Which insert-effect slot this part edits (Effect1/Effect2).
    pub effect_select: EffectSelect,
    /// Stereo pan, −63..=+63 (L63 .. C .. R63). Added in CK firmware v1.10
    /// (Part block offset 0x1C; device-verified by capture-and-diff).
    #[cfg_attr(feature = "schema", schemars(range(min = -63, max = 63)))]
    pub pan: i8,
    /// Plays this part monophonically or polyphonically.
    pub mono_poly: MonoPoly,
    /// Mono/portamento behaviour (Normal, Fingered, Full-time).
    pub mono_type: MonoType,
    /// Portamento time, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub portamento_time: u8,
    /// Portamento time interpretation (Rate or Time).
    pub portamento_mode: PortamentoMode,
    /// Enables unison voicing.
    pub unison_switch: bool,
    /// Unison voicing (MultiLayer, Harmonics, SubHarmonics).
    pub unison_type: UnisonType,
    /// Unison volume, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub unison_volume: u8,
    /// Unison detune, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub unison_detune: u8,
    /// Pitch bend range, −24..=+24 semitones.
    #[cfg_attr(feature = "schema", schemars(range(min = -24, max = 24)))]
    pub pitch_bend_range: i8,
    /// Pitch modulation depth, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub pitch_modulation_depth: u8,
    /// Amplifier modulation depth, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub amplifier_modulation_depth: u8,
    /// Filter modulation depth, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub filter_modulation_depth: u8,
    /// Modulation speed, −64..=+63.
    #[cfg_attr(feature = "schema", schemars(range(min = -64, max = 63)))]
    pub modulation_speed: i8,
    /// This part responds to Expression (CC 11).
    pub receive_expression: bool,
    /// This part responds to the Sustain pedal (CC 64).
    pub receive_sustain: bool,
    /// This part responds to the Sostenuto pedal (CC 66).
    pub receive_sostenuto: bool,
    /// This part responds to the Soft pedal (CC 67).
    pub receive_soft: bool,
    /// External-keyboard routing (ExtInt, ExtOnly, Off).
    pub external_keyboard: ExternalKeyboard,
    /// Touch sensitivity depth, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub touch_sensitivity_depth: u8,
    /// Touch sensitivity offset, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub touch_sensitivity_offset: u8,
    /// Organ drawbars, 9 entries: 16', 5⅓', 8', 4', 2⅔', 2', 1⅗', 1⅓', 1'.
    pub drawbars: Vec<u8>,
    /// Enables organ percussion.
    pub percussion_switch: bool,
    /// Organ percussion harmonic (Third or Second).
    pub percussion_type: PercussionType,
    /// Organ percussion decay (Slow or Fast).
    pub percussion_decay: PercussionDecay,
    /// Organ percussion volume (Normal or Soft).
    pub percussion_volume: PercussionVolume,
    /// Enables the organ vibrato/chorus scanner.
    pub vibrato_chorus_switch: bool,
    /// Vibrato/Chorus program (V1, C1, V2, C2, V3, C3).
    pub vibrato_chorus_type: VibratoChorusType,
    /// Enables the filter.
    pub filter_switch: bool,
    /// Filter cutoff, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub filter_cutoff: u8,
    /// Filter resonance, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub filter_resonance: u8,
    /// Enables the amplitude envelope generator.
    pub eg_switch: bool,
    /// EG attack, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub eg_attack: u8,
    /// EG release, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub eg_release: u8,
    /// Enables the drive/amp stage.
    pub drive_switch: bool,
    /// Drive/amp model.
    pub drive_type: DriveType,
    /// Drive depth, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub drive_depth: u8,
    /// Insert effect 1 on/off.
    pub effect_1_switch: bool,
    /// Effect 1 type, raw 0x00..=0x23.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 35)))]
    pub effect_1_type: u8,
    /// Effect 1 depth, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub effect_1_depth: u8,
    /// Effect 1 rate, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub effect_1_rate: u8,
    /// Insert effect 2 on/off.
    pub effect_2_switch: bool,
    /// Effect 2 type, raw 0x00..=0x23.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 35)))]
    pub effect_2_type: u8,
    /// Effect 2 depth, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub effect_2_depth: u8,
    /// Effect 2 rate, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub effect_2_rate: u8,
    /// Reserved/undocumented bytes captured verbatim so writes round-trip exactly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reserved: Vec<RawByte>,
}

impl Default for Part {
    /// Factory defaults from the Owner's Manual "Default (HEX)" column (Part A
    /// flavour: colour 2). `LiveSet::default` adjusts colour/switch per slot.
    fn default() -> Self {
        Self {
            current_category: 0,
            category_voices: vec![0; CATEGORY_COUNT],
            note_shift: 0,
            part_volume: 0x7F,
            part_color: 0x02,
            part_switch: false,
            part_selected: false,
            effect_select: EffectSelect::Effect1,
            pan: 0,
            mono_poly: MonoPoly::Poly,
            mono_type: MonoType::Normal,
            portamento_time: 0x14,
            portamento_mode: PortamentoMode::Rate,
            unison_switch: false,
            unison_type: UnisonType::MultiLayer,
            unison_volume: 0x7F,
            unison_detune: 0x40,
            pitch_bend_range: 2,
            pitch_modulation_depth: 0x0A,
            amplifier_modulation_depth: 0,
            filter_modulation_depth: 0,
            modulation_speed: 0,
            receive_expression: true,
            receive_sustain: true,
            receive_sostenuto: true,
            receive_soft: true,
            external_keyboard: ExternalKeyboard::ExtInt,
            touch_sensitivity_depth: 0x40,
            touch_sensitivity_offset: 0x40,
            drawbars: vec![0x7F, 0x7F, 0x7F, 0, 0, 0, 0, 0, 0],
            percussion_switch: false,
            percussion_type: PercussionType::Third,
            percussion_decay: PercussionDecay::Slow,
            percussion_volume: PercussionVolume::Normal,
            vibrato_chorus_switch: false,
            vibrato_chorus_type: VibratoChorusType::C3,
            filter_switch: true,
            filter_cutoff: 0x40,
            filter_resonance: 0x40,
            eg_switch: true,
            eg_attack: 0x40,
            eg_release: 0x40,
            drive_switch: false,
            drive_type: DriveType::OverDrive,
            drive_depth: 0x40,
            effect_1_switch: false,
            effect_1_type: 0,
            effect_1_depth: 0x40,
            effect_1_rate: 0x40,
            effect_2_switch: false,
            effect_2_type: 0,
            effect_2_depth: 0x40,
            effect_2_rate: 0x40,
            reserved: Vec::new(),
        }
    }
}

impl Part {
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() < PART_LEN {
            return Err(CodecError::WrongLength {
                expected: PART_LEN,
                actual: b.len(),
            });
        }
        let mut category_voices = Vec::with_capacity(CATEGORY_COUNT);
        for i in 0..CATEGORY_COUNT {
            let off = 0x01 + i * 2;
            category_voices.push(read_u14(b[off], b[off + 1]));
        }
        let drawbars = (0..9).map(|i| b[0x40 + i]).collect();
        let mut value = Self {
            current_category: ranged(b[0x00], 0x00, 0x09, "current_category")?,
            category_voices,
            note_shift: signed_center(b[0x16], 0x40, -24, 24, "note_shift")?,
            part_volume: ranged(b[0x17], 0x00, 0x7F, "part_volume")?,
            part_color: ranged(b[0x18], 0x00, 0x0B, "part_color")?,
            part_switch: bool_byte(b[0x19], "part_switch")?,
            part_selected: bool_byte(b[0x1A], "part_selected")?,
            effect_select: EffectSelect::from_byte(b[0x1B])?,
            pan: signed_center(b[0x1C], 0x40, -63, 63, "pan")?,
            mono_poly: MonoPoly::from_byte(b[0x20])?,
            mono_type: MonoType::from_byte(b[0x21])?,
            portamento_time: ranged(b[0x23], 0x00, 0x7F, "portamento_time")?,
            portamento_mode: PortamentoMode::from_byte(b[0x24])?,
            unison_switch: bool_byte(b[0x2C], "unison_switch")?,
            unison_type: UnisonType::from_byte(b[0x2D])?,
            unison_volume: ranged(b[0x2E], 0x00, 0x7F, "unison_volume")?,
            unison_detune: ranged(b[0x2F], 0x00, 0x7F, "unison_detune")?,
            pitch_bend_range: signed_center(b[0x30], 0x40, -24, 24, "pitch_bend_range")?,
            pitch_modulation_depth: ranged(b[0x31], 0x00, 0x7F, "pitch_modulation_depth")?,
            amplifier_modulation_depth: ranged(b[0x32], 0x00, 0x7F, "amplifier_modulation_depth")?,
            filter_modulation_depth: ranged(b[0x33], 0x00, 0x7F, "filter_modulation_depth")?,
            modulation_speed: signed_center(b[0x34], 0x40, -64, 63, "modulation_speed")?,
            receive_expression: bool_byte(b[0x35], "receive_expression")?,
            receive_sustain: bool_byte(b[0x36], "receive_sustain")?,
            receive_sostenuto: bool_byte(b[0x37], "receive_sostenuto")?,
            receive_soft: bool_byte(b[0x38], "receive_soft")?,
            external_keyboard: ExternalKeyboard::from_byte(b[0x39])?,
            touch_sensitivity_depth: ranged(b[0x3A], 0x00, 0x7F, "touch_sensitivity_depth")?,
            touch_sensitivity_offset: ranged(b[0x3B], 0x00, 0x7F, "touch_sensitivity_offset")?,
            drawbars,
            percussion_switch: bool_byte(b[0x49], "percussion_switch")?,
            percussion_type: PercussionType::from_byte(b[0x4A])?,
            percussion_decay: PercussionDecay::from_byte(b[0x4B])?,
            percussion_volume: PercussionVolume::from_byte(b[0x4C])?,
            vibrato_chorus_switch: bool_byte(b[0x4E], "vibrato_chorus_switch")?,
            vibrato_chorus_type: VibratoChorusType::from_byte(b[0x4F])?,
            filter_switch: bool_byte(b[0x50], "filter_switch")?,
            filter_cutoff: ranged(b[0x51], 0x00, 0x7F, "filter_cutoff")?,
            filter_resonance: ranged(b[0x52], 0x00, 0x7F, "filter_resonance")?,
            eg_switch: bool_byte(b[0x53], "eg_switch")?,
            eg_attack: ranged(b[0x54], 0x00, 0x7F, "eg_attack")?,
            eg_release: ranged(b[0x55], 0x00, 0x7F, "eg_release")?,
            drive_switch: bool_byte(b[0x56], "drive_switch")?,
            drive_type: DriveType::from_byte(b[0x57])?,
            drive_depth: ranged(b[0x58], 0x00, 0x7F, "drive_depth")?,
            effect_1_switch: bool_byte(b[0x59], "effect_1_switch")?,
            effect_1_type: ranged(b[0x5A], 0x00, 0x23, "effect_1_type")?,
            effect_1_depth: ranged(b[0x5B], 0x00, 0x7F, "effect_1_depth")?,
            effect_1_rate: ranged(b[0x5C], 0x00, 0x7F, "effect_1_rate")?,
            effect_2_switch: bool_byte(b[0x5D], "effect_2_switch")?,
            effect_2_type: ranged(b[0x5E], 0x00, 0x23, "effect_2_type")?,
            effect_2_depth: ranged(b[0x5F], 0x00, 0x7F, "effect_2_depth")?,
            effect_2_rate: ranged(b[0x60], 0x00, 0x7F, "effect_2_rate")?,
            reserved: Vec::new(),
        };
        let typed_only = value.to_bytes()?;
        value.reserved = capture_reserved(b, &typed_only);
        Ok(value)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        if self.category_voices.len() != CATEGORY_COUNT {
            return Err(CodecError::WrongLength {
                expected: CATEGORY_COUNT,
                actual: self.category_voices.len(),
            });
        }
        if self.drawbars.len() != 9 {
            return Err(CodecError::WrongLength {
                expected: 9,
                actual: self.drawbars.len(),
            });
        }
        let mut b = vec![0u8; PART_LEN];
        b[0x00] = ranged(self.current_category, 0x00, 0x09, "current_category")?;
        for (i, &v) in self.category_voices.iter().enumerate() {
            let off = 0x01 + i * 2;
            let (hi, lo) = write_u14(v);
            b[off] = hi;
            b[off + 1] = lo;
        }
        b[0x16] = to_signed_center(self.note_shift, 0x40, -24, 24, "note_shift")?;
        b[0x17] = ranged(self.part_volume, 0x00, 0x7F, "part_volume")?;
        b[0x18] = ranged(self.part_color, 0x00, 0x0B, "part_color")?;
        b[0x19] = self.part_switch as u8;
        b[0x1A] = self.part_selected as u8;
        b[0x1B] = self.effect_select.to_byte();
        b[0x1C] = to_signed_center(self.pan, 0x40, -63, 63, "pan")?;
        b[0x20] = self.mono_poly.to_byte();
        b[0x21] = self.mono_type.to_byte();
        b[0x23] = ranged(self.portamento_time, 0x00, 0x7F, "portamento_time")?;
        b[0x24] = self.portamento_mode.to_byte();
        b[0x2C] = self.unison_switch as u8;
        b[0x2D] = self.unison_type.to_byte();
        b[0x2E] = ranged(self.unison_volume, 0x00, 0x7F, "unison_volume")?;
        b[0x2F] = ranged(self.unison_detune, 0x00, 0x7F, "unison_detune")?;
        b[0x30] = to_signed_center(self.pitch_bend_range, 0x40, -24, 24, "pitch_bend_range")?;
        b[0x31] = ranged(
            self.pitch_modulation_depth,
            0x00,
            0x7F,
            "pitch_modulation_depth",
        )?;
        b[0x32] = ranged(
            self.amplifier_modulation_depth,
            0x00,
            0x7F,
            "amplifier_modulation_depth",
        )?;
        b[0x33] = ranged(
            self.filter_modulation_depth,
            0x00,
            0x7F,
            "filter_modulation_depth",
        )?;
        b[0x34] = to_signed_center(self.modulation_speed, 0x40, -64, 63, "modulation_speed")?;
        b[0x35] = self.receive_expression as u8;
        b[0x36] = self.receive_sustain as u8;
        b[0x37] = self.receive_sostenuto as u8;
        b[0x38] = self.receive_soft as u8;
        b[0x39] = self.external_keyboard.to_byte();
        b[0x3A] = ranged(
            self.touch_sensitivity_depth,
            0x00,
            0x7F,
            "touch_sensitivity_depth",
        )?;
        b[0x3B] = ranged(
            self.touch_sensitivity_offset,
            0x00,
            0x7F,
            "touch_sensitivity_offset",
        )?;
        for (i, &v) in self.drawbars.iter().enumerate() {
            b[0x40 + i] = ranged(v, 0x00, 0x7F, "drawbars")?;
        }
        b[0x49] = self.percussion_switch as u8;
        b[0x4A] = self.percussion_type.to_byte();
        b[0x4B] = self.percussion_decay.to_byte();
        b[0x4C] = self.percussion_volume.to_byte();
        b[0x4E] = self.vibrato_chorus_switch as u8;
        b[0x4F] = self.vibrato_chorus_type.to_byte();
        b[0x50] = self.filter_switch as u8;
        b[0x51] = ranged(self.filter_cutoff, 0x00, 0x7F, "filter_cutoff")?;
        b[0x52] = ranged(self.filter_resonance, 0x00, 0x7F, "filter_resonance")?;
        b[0x53] = self.eg_switch as u8;
        b[0x54] = ranged(self.eg_attack, 0x00, 0x7F, "eg_attack")?;
        b[0x55] = ranged(self.eg_release, 0x00, 0x7F, "eg_release")?;
        b[0x56] = self.drive_switch as u8;
        b[0x57] = self.drive_type.to_byte();
        b[0x58] = ranged(self.drive_depth, 0x00, 0x7F, "drive_depth")?;
        b[0x59] = self.effect_1_switch as u8;
        b[0x5A] = ranged(self.effect_1_type, 0x00, 0x23, "effect_1_type")?;
        b[0x5B] = ranged(self.effect_1_depth, 0x00, 0x7F, "effect_1_depth")?;
        b[0x5C] = ranged(self.effect_1_rate, 0x00, 0x7F, "effect_1_rate")?;
        b[0x5D] = self.effect_2_switch as u8;
        b[0x5E] = ranged(self.effect_2_type, 0x00, 0x23, "effect_2_type")?;
        b[0x5F] = ranged(self.effect_2_depth, 0x00, 0x7F, "effect_2_depth")?;
        b[0x60] = ranged(self.effect_2_rate, 0x00, 0x7F, "effect_2_rate")?;
        apply_reserved(&mut b, &self.reserved);
        Ok(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_part_bytes() -> Vec<u8> {
        let mut b = vec![0u8; PART_LEN];
        b[0x16] = 0x40; // note shift 0
        b[0x17] = 0x7F; // part volume
        b[0x18] = 0x02; // colour
        b[0x19] = 0x01; // part on
        b[0x1A] = 0x01; // selected
        b[0x1C] = 0x40; // pan center
        b[0x20] = 0x01; // poly
        b[0x23] = 0x14; // portamento time
        b[0x2E] = 0x7F; // unison volume
        b[0x2F] = 0x40; // unison detune
        b[0x30] = 0x42; // pitch bend +2
        b[0x31] = 0x0A;
        b[0x34] = 0x40; // mod speed 0
        b[0x35] = 0x01;
        b[0x36] = 0x01;
        b[0x37] = 0x01;
        b[0x38] = 0x01;
        b[0x3A] = 0x40;
        b[0x3B] = 0x40;
        b[0x40] = 0x7F; // drawbar 16'
        b[0x41] = 0x7F;
        b[0x42] = 0x7F;
        b[0x4F] = 0x05; // vibrato/chorus C3
        b[0x50] = 0x01; // filter on
        b[0x51] = 0x40;
        b[0x52] = 0x40;
        b[0x53] = 0x01; // eg on
        b[0x54] = 0x40;
        b[0x55] = 0x40;
        b[0x58] = 0x40;
        b[0x5B] = 0x40;
        b[0x5C] = 0x40;
        b[0x5F] = 0x40;
        b[0x60] = 0x40;
        b
    }

    #[test]
    fn part_round_trips() {
        let bytes = default_part_bytes();
        let p = Part::from_bytes(&bytes).unwrap();
        assert_eq!(p.category_voices.len(), CATEGORY_COUNT);
        assert_eq!(p.drawbars.len(), 9);
        assert_eq!(p.note_shift, 0);
        assert_eq!(p.pan, 0);
        assert_eq!(p.pitch_bend_range, 2);
        assert!(p.part_switch);
        assert_eq!(p.vibrato_chorus_type, VibratoChorusType::C3);
        assert_eq!(p.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn part_voice_numbers_round_trip() {
        let mut bytes = default_part_bytes();
        // Category 4 voice number 108 (max of Brs/Wind), stored 14-bit at 0x07.
        let (hi, lo) = write_u14(108);
        bytes[0x07] = hi;
        bytes[0x08] = lo;
        let p = Part::from_bytes(&bytes).unwrap();
        assert_eq!(p.category_voices[3], 108);
        assert_eq!(p.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn part_modulation_speed_extremes() {
        let mut bytes = default_part_bytes();
        bytes[0x34] = 0x00;
        assert_eq!(Part::from_bytes(&bytes).unwrap().modulation_speed, -64);
        bytes[0x34] = 0x7F;
        assert_eq!(Part::from_bytes(&bytes).unwrap().modulation_speed, 63);
    }
}
