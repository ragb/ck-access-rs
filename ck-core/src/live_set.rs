//! Live Set **Common** (`46 00 00`, 83 bytes) and **EQ** (`46 40 00`, 20 bytes).
//!
//! The Live Set Common block holds everything shared by the three Parts: the
//! Live Set name, layer/split layout, tempo, the master Delay/Reverb sends, the
//! Rotary state, and the A/D (audio-in) channel. Offsets/ranges from the
//! *Owner's Manual* "MIDI Data Table → Live Set Common".

use serde::{Deserialize, Serialize};

use crate::address::{LIVE_SET_COMMON_LEN, LIVE_SET_EQ_LEN};
use crate::codec::{
    apply_reserved, bool_byte, byte_enum, capture_reserved, ranged, read_ascii, read_u14,
    signed_center, to_signed_center, write_ascii, write_u14, CodecError, RawByte,
};

/// Number of ASCII characters in a Live Set name.
pub const NAME_LEN: usize = 15;

byte_enum! {
    /// Master Delay algorithm (offset 0x28).
    DelayType { Digital = 0, Analog = 1, Cross = 2, Tempo = 3 }
    valid = "0..=3"
}

byte_enum! {
    /// Master Reverb algorithm (offset 0x2D).
    ReverbType { Hall = 0, Room = 1, Plate = 2 }
    valid = "0..=2"
}

byte_enum! {
    /// Rotary speaker speed (offset 0x30).
    RotarySpeed { Slow = 0, Fast = 1 }
    valid = "0=slow, 1=fast"
}

byte_enum! {
    /// Which held key triggers the audio sample (offset 0x35).
    AudioTriggerKey { Lowest = 0, Highest = 1 }
    valid = "0=lowest, 1=highest"
}

byte_enum! {
    /// Audio-trigger playback behaviour (offset 0x36). `Hold` added in CK
    /// firmware v1.10.
    AudioTriggerPlayMode { OneShot = 0, PlayStop = 1, PlayPause = 2, Hold = 3 }
    valid = "0..=3"
}

/// The Live Set **Common** block.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LiveSetCommon {
    /// Live Set name, up to 15 ASCII characters.
    pub name: String,
    /// Enables the per-Live-Set 3-band EQ.
    pub live_set_eq_mode: bool,
    /// Enables Master Keyboard mode (external-zone control).
    pub master_keyboard_mode: bool,
    /// Enables advanced zone settings for the external keyboard.
    pub advanced_zone: bool,
    /// Tempo ×10 (e.g. `1200` = 120.0 BPM); raw range 420..=2400.
    #[cfg_attr(feature = "schema", schemars(range(min = 420, max = 2400)))]
    pub tempo_x10: u16,
    /// Sound transpose, −12..=+12 semitones.
    #[cfg_attr(feature = "schema", schemars(range(min = -12, max = 12)))]
    pub sound_transpose: i8,
    /// Layer/split layout, raw 0..=3: 0=ABC, 1=A/BC, 2=AB/C, 3=A/B/C.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 3)))]
    pub layer_split_mode: u8,
    /// Split point (two-way split), raw 1..=127 (note number).
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 127)))]
    pub split_point: u8,
    /// Split point A–B (three-way), raw 1..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 127)))]
    pub split_point_a_b: u8,
    /// Split point B–C (three-way), raw 2..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 2, max = 127)))]
    pub split_point_b_c: u8,
    /// Modulation wheel assignment, raw 0..=120 (control number; 120 = USB audio volume).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 120)))]
    pub mod_wheel_assign: u8,
    /// Foot Pedal 1 assignment, raw 0..=120.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 120)))]
    pub foot_pedal_1_assign: u8,
    /// Foot Pedal 1 lower limit, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub foot_pedal_1_limit_low: u8,
    /// Foot Pedal 1 upper limit, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub foot_pedal_1_limit_high: u8,
    /// Foot Pedal 2 assignment, raw 0..=120.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 120)))]
    pub foot_pedal_2_assign: u8,
    /// Foot Pedal 2 lower limit, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub foot_pedal_2_limit_low: u8,
    /// Foot Pedal 2 upper limit, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub foot_pedal_2_limit_high: u8,
    /// Master delay on/off.
    pub delay_switch: bool,
    /// Delay algorithm (Digital, Analog, Cross, Tempo).
    pub delay_type: DelayType,
    /// Master Delay depth (send/mix amount), 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub delay_depth: u8,
    /// Master Delay time, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub delay_time: u8,
    /// Tempo-delay note value, raw 0..=14.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 14)))]
    pub delay_tempo_time: u8,
    /// Master reverb on/off.
    pub reverb_switch: bool,
    /// Reverb algorithm (Hall, Room, Plate).
    pub reverb_type: ReverbType,
    /// Master Reverb depth (send/mix amount), 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub reverb_depth: u8,
    /// Rotary speaker speed (slow/fast).
    pub rotary_speed: RotarySpeed,
    /// Stops the rotary speaker (brake).
    pub rotary_stop: bool,
    /// Audio-trigger sample playback on/off.
    pub audio_trigger_switch: bool,
    /// Audio-trigger sample volume, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub audio_trigger_volume: u8,
    /// Which held key triggers the audio sample (lowest/highest).
    pub audio_trigger_key: AudioTriggerKey,
    /// Audio-trigger playback behaviour (OneShot, PlayStop, PlayPause, Hold).
    pub audio_trigger_play_mode: AudioTriggerPlayMode,
    /// A/D input EQ low frequency, raw 0x04..=0x28.
    #[cfg_attr(feature = "schema", schemars(range(min = 4, max = 40)))]
    pub ad_eq_low_freq: u8,
    /// A/D input EQ low gain, −12..=+12 dB.
    #[cfg_attr(feature = "schema", schemars(range(min = -12, max = 12)))]
    pub ad_eq_low_gain: i8,
    /// A/D input EQ mid frequency, raw 0x0E..=0x36.
    #[cfg_attr(feature = "schema", schemars(range(min = 14, max = 54)))]
    pub ad_eq_mid_freq: u8,
    /// A/D input EQ mid gain, −12..=+12 dB.
    #[cfg_attr(feature = "schema", schemars(range(min = -12, max = 12)))]
    pub ad_eq_mid_gain: i8,
    /// A/D input EQ high frequency, raw 0x1C..=0x3A.
    #[cfg_attr(feature = "schema", schemars(range(min = 28, max = 58)))]
    pub ad_eq_high_freq: u8,
    /// A/D input EQ high gain, −12..=+12 dB.
    #[cfg_attr(feature = "schema", schemars(range(min = -12, max = 12)))]
    pub ad_eq_high_gain: i8,
    /// A/D input noise gate on/off.
    pub ad_noise_gate_switch: bool,
    /// Noise-gate threshold, raw 0x36..=0x61 (−73 .. −30 dB).
    #[cfg_attr(feature = "schema", schemars(range(min = 54, max = 97)))]
    pub ad_noise_gate_threshold: u8,
    /// A/D effect 1 type, raw 0x00..=0x22.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 34)))]
    pub ad_effect_1_type: u8,
    /// A/D effect 1 depth, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub ad_effect_1_depth: u8,
    /// A/D effect 1 rate, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub ad_effect_1_rate: u8,
    /// A/D effect 2 type, raw 0x00..=0x22.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 34)))]
    pub ad_effect_2_type: u8,
    /// A/D effect 2 depth, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub ad_effect_2_depth: u8,
    /// A/D effect 2 rate, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub ad_effect_2_rate: u8,
    /// A/D input channel volume, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub ad_volume: u8,
    /// Reserved/undocumented bytes captured verbatim so writes round-trip exactly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reserved: Vec<RawByte>,
}

impl Default for LiveSetCommon {
    /// Factory defaults from the Owner's Manual "Default (HEX)" column.
    fn default() -> Self {
        Self {
            name: "Init".to_string(),
            live_set_eq_mode: false,
            master_keyboard_mode: false,
            advanced_zone: false,
            tempo_x10: 1200,
            sound_transpose: 0,
            layer_split_mode: 0,
            split_point: 0x37,
            split_point_a_b: 0x37,
            split_point_b_c: 0x4F,
            mod_wheel_assign: 1,
            foot_pedal_1_assign: 0x40,
            foot_pedal_1_limit_low: 0,
            foot_pedal_1_limit_high: 0x7F,
            foot_pedal_2_assign: 0x0B,
            foot_pedal_2_limit_low: 0,
            foot_pedal_2_limit_high: 0x7F,
            delay_switch: false,
            delay_type: DelayType::Digital,
            delay_depth: 0x40,
            delay_time: 0x40,
            delay_tempo_time: 0x0B,
            reverb_switch: true,
            reverb_type: ReverbType::Hall,
            reverb_depth: 0x14,
            rotary_speed: RotarySpeed::Slow,
            rotary_stop: false,
            audio_trigger_switch: false,
            audio_trigger_volume: 0x40,
            audio_trigger_key: AudioTriggerKey::Highest,
            audio_trigger_play_mode: AudioTriggerPlayMode::OneShot,
            ad_eq_low_freq: 0x12,
            ad_eq_low_gain: 0,
            ad_eq_mid_freq: 0x29,
            ad_eq_mid_gain: 0,
            ad_eq_high_freq: 0x34,
            ad_eq_high_gain: 0,
            ad_noise_gate_switch: false,
            ad_noise_gate_threshold: 0x52,
            ad_effect_1_type: 0,
            ad_effect_1_depth: 0x40,
            ad_effect_1_rate: 0x40,
            ad_effect_2_type: 0,
            ad_effect_2_depth: 0x40,
            ad_effect_2_rate: 0x40,
            ad_volume: 0x7F,
            reserved: Vec::new(),
        }
    }
}

impl LiveSetCommon {
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() < LIVE_SET_COMMON_LEN {
            return Err(CodecError::WrongLength {
                expected: LIVE_SET_COMMON_LEN,
                actual: b.len(),
            });
        }
        let mut value = Self {
            name: read_ascii(&b[0x00..0x0F], "name")?,
            live_set_eq_mode: bool_byte(b[0x10], "live_set_eq_mode")?,
            master_keyboard_mode: bool_byte(b[0x11], "master_keyboard_mode")?,
            advanced_zone: bool_byte(b[0x12], "advanced_zone")?,
            tempo_x10: read_u14(b[0x13], b[0x14]),
            sound_transpose: signed_center(b[0x15], 0x40, -12, 12, "sound_transpose")?,
            layer_split_mode: ranged(b[0x16], 0x00, 0x03, "layer_split_mode")?,
            split_point: ranged(b[0x18], 0x01, 0x7F, "split_point")?,
            split_point_a_b: ranged(b[0x19], 0x01, 0x7F, "split_point_a_b")?,
            split_point_b_c: ranged(b[0x1A], 0x02, 0x7F, "split_point_b_c")?,
            mod_wheel_assign: ranged(b[0x1C], 0x00, 0x78, "mod_wheel_assign")?,
            foot_pedal_1_assign: ranged(b[0x1F], 0x00, 0x78, "foot_pedal_1_assign")?,
            foot_pedal_1_limit_low: ranged(b[0x20], 0x00, 0x7F, "foot_pedal_1_limit_low")?,
            foot_pedal_1_limit_high: ranged(b[0x21], 0x00, 0x7F, "foot_pedal_1_limit_high")?,
            foot_pedal_2_assign: ranged(b[0x22], 0x00, 0x78, "foot_pedal_2_assign")?,
            foot_pedal_2_limit_low: ranged(b[0x23], 0x00, 0x7F, "foot_pedal_2_limit_low")?,
            foot_pedal_2_limit_high: ranged(b[0x24], 0x00, 0x7F, "foot_pedal_2_limit_high")?,
            delay_switch: bool_byte(b[0x27], "delay_switch")?,
            delay_type: DelayType::from_byte(b[0x28])?,
            delay_depth: ranged(b[0x29], 0x00, 0x7F, "delay_depth")?,
            delay_time: ranged(b[0x2A], 0x00, 0x7F, "delay_time")?,
            delay_tempo_time: ranged(b[0x2B], 0x00, 0x0E, "delay_tempo_time")?,
            reverb_switch: bool_byte(b[0x2C], "reverb_switch")?,
            reverb_type: ReverbType::from_byte(b[0x2D])?,
            reverb_depth: ranged(b[0x2E], 0x00, 0x7F, "reverb_depth")?,
            rotary_speed: RotarySpeed::from_byte(b[0x30])?,
            rotary_stop: bool_byte(b[0x31], "rotary_stop")?,
            audio_trigger_switch: bool_byte(b[0x33], "audio_trigger_switch")?,
            audio_trigger_volume: ranged(b[0x34], 0x00, 0x7F, "audio_trigger_volume")?,
            audio_trigger_key: AudioTriggerKey::from_byte(b[0x35])?,
            audio_trigger_play_mode: AudioTriggerPlayMode::from_byte(b[0x36])?,
            ad_eq_low_freq: ranged(b[0x38], 0x04, 0x28, "ad_eq_low_freq")?,
            ad_eq_low_gain: signed_center(b[0x39], 0x40, -12, 12, "ad_eq_low_gain")?,
            ad_eq_mid_freq: ranged(b[0x3A], 0x0E, 0x36, "ad_eq_mid_freq")?,
            ad_eq_mid_gain: signed_center(b[0x3B], 0x40, -12, 12, "ad_eq_mid_gain")?,
            ad_eq_high_freq: ranged(b[0x3C], 0x1C, 0x3A, "ad_eq_high_freq")?,
            ad_eq_high_gain: signed_center(b[0x3D], 0x40, -12, 12, "ad_eq_high_gain")?,
            ad_noise_gate_switch: bool_byte(b[0x3E], "ad_noise_gate_switch")?,
            ad_noise_gate_threshold: ranged(b[0x3F], 0x36, 0x61, "ad_noise_gate_threshold")?,
            ad_effect_1_type: ranged(b[0x45], 0x00, 0x22, "ad_effect_1_type")?,
            ad_effect_1_depth: ranged(b[0x46], 0x00, 0x7F, "ad_effect_1_depth")?,
            ad_effect_1_rate: ranged(b[0x47], 0x00, 0x7F, "ad_effect_1_rate")?,
            ad_effect_2_type: ranged(b[0x49], 0x00, 0x22, "ad_effect_2_type")?,
            ad_effect_2_depth: ranged(b[0x4A], 0x00, 0x7F, "ad_effect_2_depth")?,
            ad_effect_2_rate: ranged(b[0x4B], 0x00, 0x7F, "ad_effect_2_rate")?,
            ad_volume: ranged(b[0x4C], 0x00, 0x7F, "ad_volume")?,
            reserved: Vec::new(),
        };
        let typed_only = value.to_bytes()?;
        value.reserved = capture_reserved(b, &typed_only);
        Ok(value)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        let mut b = vec![0u8; LIVE_SET_COMMON_LEN];
        b[0x00..0x0F].copy_from_slice(&write_ascii(&self.name, NAME_LEN, "name")?);
        b[0x10] = self.live_set_eq_mode as u8;
        b[0x11] = self.master_keyboard_mode as u8;
        b[0x12] = self.advanced_zone as u8;
        let (th, tl) = write_u14(self.tempo_x10);
        b[0x13] = th;
        b[0x14] = tl;
        b[0x15] = to_signed_center(self.sound_transpose, 0x40, -12, 12, "sound_transpose")?;
        b[0x16] = ranged(self.layer_split_mode, 0x00, 0x03, "layer_split_mode")?;
        b[0x18] = ranged(self.split_point, 0x01, 0x7F, "split_point")?;
        b[0x19] = ranged(self.split_point_a_b, 0x01, 0x7F, "split_point_a_b")?;
        b[0x1A] = ranged(self.split_point_b_c, 0x02, 0x7F, "split_point_b_c")?;
        b[0x1C] = ranged(self.mod_wheel_assign, 0x00, 0x78, "mod_wheel_assign")?;
        b[0x1F] = ranged(self.foot_pedal_1_assign, 0x00, 0x78, "foot_pedal_1_assign")?;
        b[0x20] = ranged(
            self.foot_pedal_1_limit_low,
            0x00,
            0x7F,
            "foot_pedal_1_limit_low",
        )?;
        b[0x21] = ranged(
            self.foot_pedal_1_limit_high,
            0x00,
            0x7F,
            "foot_pedal_1_limit_high",
        )?;
        b[0x22] = ranged(self.foot_pedal_2_assign, 0x00, 0x78, "foot_pedal_2_assign")?;
        b[0x23] = ranged(
            self.foot_pedal_2_limit_low,
            0x00,
            0x7F,
            "foot_pedal_2_limit_low",
        )?;
        b[0x24] = ranged(
            self.foot_pedal_2_limit_high,
            0x00,
            0x7F,
            "foot_pedal_2_limit_high",
        )?;
        b[0x27] = self.delay_switch as u8;
        b[0x28] = self.delay_type.to_byte();
        b[0x29] = ranged(self.delay_depth, 0x00, 0x7F, "delay_depth")?;
        b[0x2A] = ranged(self.delay_time, 0x00, 0x7F, "delay_time")?;
        b[0x2B] = ranged(self.delay_tempo_time, 0x00, 0x0E, "delay_tempo_time")?;
        b[0x2C] = self.reverb_switch as u8;
        b[0x2D] = self.reverb_type.to_byte();
        b[0x2E] = ranged(self.reverb_depth, 0x00, 0x7F, "reverb_depth")?;
        b[0x30] = self.rotary_speed.to_byte();
        b[0x31] = self.rotary_stop as u8;
        b[0x33] = self.audio_trigger_switch as u8;
        b[0x34] = ranged(
            self.audio_trigger_volume,
            0x00,
            0x7F,
            "audio_trigger_volume",
        )?;
        b[0x35] = self.audio_trigger_key.to_byte();
        b[0x36] = self.audio_trigger_play_mode.to_byte();
        b[0x38] = ranged(self.ad_eq_low_freq, 0x04, 0x28, "ad_eq_low_freq")?;
        b[0x39] = to_signed_center(self.ad_eq_low_gain, 0x40, -12, 12, "ad_eq_low_gain")?;
        b[0x3A] = ranged(self.ad_eq_mid_freq, 0x0E, 0x36, "ad_eq_mid_freq")?;
        b[0x3B] = to_signed_center(self.ad_eq_mid_gain, 0x40, -12, 12, "ad_eq_mid_gain")?;
        b[0x3C] = ranged(self.ad_eq_high_freq, 0x1C, 0x3A, "ad_eq_high_freq")?;
        b[0x3D] = to_signed_center(self.ad_eq_high_gain, 0x40, -12, 12, "ad_eq_high_gain")?;
        b[0x3E] = self.ad_noise_gate_switch as u8;
        b[0x3F] = ranged(
            self.ad_noise_gate_threshold,
            0x36,
            0x61,
            "ad_noise_gate_threshold",
        )?;
        b[0x45] = ranged(self.ad_effect_1_type, 0x00, 0x22, "ad_effect_1_type")?;
        b[0x46] = ranged(self.ad_effect_1_depth, 0x00, 0x7F, "ad_effect_1_depth")?;
        b[0x47] = ranged(self.ad_effect_1_rate, 0x00, 0x7F, "ad_effect_1_rate")?;
        b[0x49] = ranged(self.ad_effect_2_type, 0x00, 0x22, "ad_effect_2_type")?;
        b[0x4A] = ranged(self.ad_effect_2_depth, 0x00, 0x7F, "ad_effect_2_depth")?;
        b[0x4B] = ranged(self.ad_effect_2_rate, 0x00, 0x7F, "ad_effect_2_rate")?;
        b[0x4C] = ranged(self.ad_volume, 0x00, 0x7F, "ad_volume")?;
        apply_reserved(&mut b, &self.reserved);
        Ok(b)
    }
}

/// The Live Set **EQ** block — a 3-band EQ, same layout as the System Master EQ.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LiveSetEq {
    /// Low gain, −12..=+12 dB.
    #[cfg_attr(feature = "schema", schemars(range(min = -12, max = 12)))]
    pub low_gain: i8,
    /// Low frequency, raw 0x04..=0x28.
    #[cfg_attr(feature = "schema", schemars(range(min = 4, max = 40)))]
    pub low_freq: u8,
    /// Mid gain, −12..=+12 dB.
    #[cfg_attr(feature = "schema", schemars(range(min = -12, max = 12)))]
    pub mid_gain: i8,
    /// Mid frequency, raw 0x0E..=0x36.
    #[cfg_attr(feature = "schema", schemars(range(min = 14, max = 54)))]
    pub mid_freq: u8,
    /// High gain, −12..=+12 dB.
    #[cfg_attr(feature = "schema", schemars(range(min = -12, max = 12)))]
    pub high_gain: i8,
    /// High frequency, raw 0x1C..=0x3A.
    #[cfg_attr(feature = "schema", schemars(range(min = 28, max = 58)))]
    pub high_freq: u8,
    /// Reserved/undocumented bytes captured verbatim so writes round-trip exactly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reserved: Vec<RawByte>,
}

impl Default for LiveSetEq {
    /// Factory defaults: flat gains, default band centres.
    fn default() -> Self {
        Self {
            low_gain: 0,
            low_freq: 0x0C,
            mid_gain: 0,
            mid_freq: 0x22,
            high_gain: 0,
            high_freq: 0x30,
            reserved: Vec::new(),
        }
    }
}

impl LiveSetEq {
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() < LIVE_SET_EQ_LEN {
            return Err(CodecError::WrongLength {
                expected: LIVE_SET_EQ_LEN,
                actual: b.len(),
            });
        }
        let mut value = Self {
            low_gain: signed_center(b[0x00], 0x40, -12, 12, "low_gain")?,
            low_freq: ranged(b[0x01], 0x04, 0x28, "low_freq")?,
            mid_gain: signed_center(b[0x08], 0x40, -12, 12, "mid_gain")?,
            mid_freq: ranged(b[0x09], 0x0E, 0x36, "mid_freq")?,
            high_gain: signed_center(b[0x10], 0x40, -12, 12, "high_gain")?,
            high_freq: ranged(b[0x11], 0x1C, 0x3A, "high_freq")?,
            reserved: Vec::new(),
        };
        let typed_only = value.to_bytes()?;
        value.reserved = capture_reserved(b, &typed_only);
        Ok(value)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        let mut b = vec![0u8; LIVE_SET_EQ_LEN];
        b[0x00] = to_signed_center(self.low_gain, 0x40, -12, 12, "low_gain")?;
        b[0x01] = ranged(self.low_freq, 0x04, 0x28, "low_freq")?;
        b[0x08] = to_signed_center(self.mid_gain, 0x40, -12, 12, "mid_gain")?;
        b[0x09] = ranged(self.mid_freq, 0x0E, 0x36, "mid_freq")?;
        b[0x10] = to_signed_center(self.high_gain, 0x40, -12, 12, "high_gain")?;
        b[0x11] = ranged(self.high_freq, 0x1C, 0x3A, "high_freq")?;
        apply_reserved(&mut b, &self.reserved);
        Ok(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_common_bytes() -> Vec<u8> {
        let mut b = vec![0x20u8; NAME_LEN]; // name padded with spaces
        b.resize(LIVE_SET_COMMON_LEN, 0);
        b[0x00..0x04].copy_from_slice(b"Init");
        b[0x13] = 0x09; // tempo 120.0
        b[0x14] = 0x30;
        b[0x15] = 0x40; // transpose 0
        b[0x18] = 0x37;
        b[0x19] = 0x37;
        b[0x1A] = 0x4F;
        b[0x1C] = 0x01;
        b[0x1F] = 0x40;
        b[0x21] = 0x7F;
        b[0x22] = 0x0B;
        b[0x24] = 0x7F;
        b[0x2C] = 0x01; // reverb on
        b[0x2E] = 0x14;
        b[0x34] = 0x40;
        b[0x35] = 0x01;
        b[0x38] = 0x12;
        b[0x39] = 0x40;
        b[0x3A] = 0x29;
        b[0x3B] = 0x40;
        b[0x3C] = 0x34;
        b[0x3D] = 0x40;
        b[0x3F] = 0x52;
        b[0x46] = 0x40;
        b[0x47] = 0x40;
        b[0x4A] = 0x40;
        b[0x4B] = 0x40;
        b[0x4C] = 0x7F;
        b
    }

    #[test]
    fn live_set_common_round_trips() {
        let bytes = default_common_bytes();
        let c = LiveSetCommon::from_bytes(&bytes).unwrap();
        assert_eq!(c.name, "Init");
        assert_eq!(c.tempo_x10, 1200);
        assert_eq!(c.sound_transpose, 0);
        assert!(c.reverb_switch);
        assert_eq!(c.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn name_truncates_and_pads() {
        let mut c = LiveSetCommon::from_bytes(&default_common_bytes()).unwrap();
        c.name = "A very long live set name".to_string();
        let bytes = c.to_bytes().unwrap();
        assert_eq!(&bytes[0x00..0x0F], b"A very long liv");
    }

    #[test]
    fn live_set_eq_round_trips() {
        let mut b = vec![0u8; LIVE_SET_EQ_LEN];
        b[0x00] = 0x40;
        b[0x01] = 0x0C;
        b[0x08] = 0x40;
        b[0x09] = 0x22;
        b[0x10] = 0x40;
        b[0x11] = 0x30;
        let eq = LiveSetEq::from_bytes(&b).unwrap();
        assert_eq!(eq.to_bytes().unwrap(), b);
    }
}
