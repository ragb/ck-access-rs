//! System area: **Common** (`20 00 00`, 56 bytes) and **Master EQ**
//! (`20 40 00`, 20 bytes).
//!
//! Offsets, ranges, and defaults are from the *CK88/CK61 Owner's Manual* "MIDI
//! Data Table → SYSTEM". Boolean (`Off/On`) and small, unambiguous enumerations
//! are typed; engineering-unit numerics (dB / Hz / index tables) whose exact
//! mapping isn't device-confirmed are kept as documented raw bytes so the codec
//! round-trips byte-exact. Reserved offsets are re-emitted as `0x00`.
//!
//! Forward-compat: `from_bytes` accepts payloads **longer** than the documented
//! length and preserves the surplus in `tail`, so dumps from newer firmware
//! (which appends parameters) still round-trip.

use serde::{Deserialize, Serialize};

use crate::address::{MASTER_EQ_LEN, SYSTEM_COMMON_LEN};
use crate::codec::{
    apply_reserved, bool_byte, byte_enum, capture_reserved, ranged, read_u16_nibbles,
    signed_center, to_signed_center, write_u16_nibbles, CodecError, RawByte,
};

byte_enum! {
    /// What "controller reset" does on Live Set change (offset 0x08).
    ControllerReset { Hold = 0, Reset = 1 }
    valid = "0=hold, 1=reset"
}

byte_enum! {
    /// Keyboard touch (velocity) curve (offset 0x10).
    TouchCurve { Normal = 0, Soft = 1, Hard = 2, Wide = 3, Fixed = 4 }
    valid = "0..=4"
}

byte_enum! {
    /// Knob pickup behaviour (offset 0x1C).
    ControllerMode { Jump = 0, Catch = 1 }
    valid = "0=jump, 1=catch"
}

byte_enum! {
    /// Foot-pedal hardware type (offsets 0x2A / 0x2C).
    FootPedalType {
        Fc3aHalfOn = 0,
        Fc3aHalfOff = 1,
        Fc4aFc5 = 2,
        Fc7 = 3,
    }
    valid = "0..=3"
}

byte_enum! {
    /// Foot-pedal Live Set increment/decrement assignment (offsets 0x2B / 0x2D).
    FootPedalLiveSet { Off = 0, Inc = 1, Dec = 2 }
    valid = "0..=2"
}

byte_enum! {
    /// Speaker voicing (offset 0x36).
    SpeakerMode { Normal = 0, Table = 1 }
    valid = "0=normal, 1=table"
}

byte_enum! {
    /// Speaker mute behaviour (offset 0x37).
    SpeakerMute { Auto = 0, Manual = 1 }
    valid = "0=auto, 1=manual"
}

/// The System **Common** block.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// Restores the per-field `"default":` entries in the emitted JSON Schema. Only
// under `schema`: `tsify` reads a container default as "every field optional" and
// would loosen the editor's TypeScript. Partial-document leniency lives at the
// document boundary instead (see `document::from_value_over_default`).
#[cfg_attr(feature = "schema", serde(default))]
pub struct SystemCommon {
    /// Master Tune, raw 16-bit value (`414.72–466.78 Hz`; default `0x0400`).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 65535)))]
    pub master_tune: u16,
    /// Keyboard octave shift, −3..=+3.
    #[cfg_attr(feature = "schema", schemars(range(min = -3, max = 3)))]
    pub keyboard_octave_shift: i8,
    /// Keyboard transpose, −12..=+12 semitones.
    #[cfg_attr(feature = "schema", schemars(range(min = -12, max = 12)))]
    pub keyboard_transpose: i8,
    /// What happens to controllers when the Live Set changes: hold or reset.
    pub controller_reset: ControllerReset,
    /// When off, the keyboard stops playing the internal tone generator directly (still sends MIDI).
    pub local_control: bool,
    /// MIDI transmit channel, raw `0..=0x7F` (`0..=15` = ch 1–16, higher = Off).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub tx_channel: u8,
    /// MIDI receive channel, raw `0..=0x10` (`0..=15` = ch 1–16, `0x10` = All).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 16)))]
    pub rx_channel: u8,
    /// Receive and act on MIDI control messages from external devices.
    pub midi_control: bool,
    /// Output gain, raw `0x38..=0x48` (`−24..0..+24 dB`, default `0x3E`).
    #[cfg_attr(feature = "schema", schemars(range(min = 56, max = 72)))]
    pub output_gain: u8,
    /// Keyboard velocity (touch) response curve.
    pub touch_curve: TouchCurve,
    /// Fixed velocity used when `touch_curve = fixed`, 1..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 127)))]
    pub fixed_velocity: u8,
    /// Send and receive Bank Select messages.
    pub tx_rx_bank_select: bool,
    /// Send and receive Program Change messages.
    pub tx_rx_program_change: bool,
    /// Route MIDI messages through the MIDI IN/OUT ports.
    pub midi_in_out: bool,
    /// Route MIDI messages through the USB TO HOST port.
    pub usb_in_out: bool,
    /// Show the current parameter value on the display when a control is moved.
    pub value_indication: bool,
    /// Knob pickup behaviour when the physical position differs from the stored value.
    pub controller_mode: ControllerMode,
    /// Turn the LCD backlight on or off.
    pub lcd_switch: bool,
    /// LCD contrast, −10..=+10.
    #[cfg_attr(feature = "schema", schemars(range(min = -10, max = 10)))]
    pub lcd_contrast: i8,
    /// Prevents Live Set selection from changing the sound.
    pub panel_lock_live_set: bool,
    /// Prevents the Organ section from changing the sound.
    pub panel_lock_organ: bool,
    /// Prevents the Filter/EG section from changing the sound.
    pub panel_lock_filter_eg: bool,
    /// Prevents the Drive/Effect section from changing the sound.
    pub panel_lock_drive_effect: bool,
    /// Prevents the Delay/Reverb section from changing the sound.
    pub panel_lock_delay_reverb: bool,
    /// Prevents the Equalizer section from changing the sound.
    pub panel_lock_equalizer: bool,
    /// Power-on page, raw `0..=0x13` (Live Set page 1–20).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 19)))]
    pub power_on_page: u8,
    /// Power-on sound, raw `0..=0x07` (sound 1–8).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 7)))]
    pub power_on_sound: u8,
    /// Hardware type connected to Foot Pedal 1.
    pub foot_pedal_1_type: FootPedalType,
    /// Foot Pedal 1 Live Set increment/decrement assignment.
    pub foot_pedal_1_live_set: FootPedalLiveSet,
    /// Hardware type connected to Foot Pedal 2.
    pub foot_pedal_2_type: FootPedalType,
    /// Foot Pedal 2 Live Set increment/decrement assignment.
    pub foot_pedal_2_live_set: FootPedalLiveSet,
    /// Reset the Filter EG to its default state when the Live Set changes.
    pub filter_eg_reset: bool,
    /// Reset effect on/off states to their default when the Live Set changes.
    pub effect_on_off_reset: bool,
    /// USB audio volume, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub usb_audio_volume: u8,
    /// Bluetooth volume, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub bluetooth_volume: u8,
    /// A/D input volume, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub ad_volume: u8,
    /// Mix incoming USB audio back into the USB audio output.
    pub usb_audio_loopback: bool,
    /// Internal-speaker voicing: Normal or Table.
    pub speaker_mode: SpeakerMode,
    /// Internal-speaker mute behaviour: automatic or manual.
    pub speaker_mute: SpeakerMute,
    /// Reserved/undocumented bytes captured verbatim so writes round-trip exactly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reserved: Vec<RawByte>,
}

impl Default for SystemCommon {
    /// Factory defaults from the Owner's Manual "Default (HEX)" column.
    fn default() -> Self {
        Self {
            master_tune: 0x0400,
            keyboard_octave_shift: 0,
            keyboard_transpose: 0,
            controller_reset: ControllerReset::Reset,
            local_control: true,
            tx_channel: 0,
            rx_channel: 0,
            midi_control: false,
            output_gain: 0x3E,
            touch_curve: TouchCurve::Normal,
            fixed_velocity: 0x40,
            tx_rx_bank_select: true,
            tx_rx_program_change: true,
            midi_in_out: true,
            usb_in_out: true,
            value_indication: true,
            controller_mode: ControllerMode::Jump,
            lcd_switch: true,
            lcd_contrast: 0,
            panel_lock_live_set: true,
            panel_lock_organ: true,
            panel_lock_filter_eg: true,
            panel_lock_drive_effect: true,
            panel_lock_delay_reverb: true,
            panel_lock_equalizer: true,
            power_on_page: 0,
            power_on_sound: 0,
            foot_pedal_1_type: FootPedalType::Fc3aHalfOn,
            foot_pedal_1_live_set: FootPedalLiveSet::Off,
            foot_pedal_2_type: FootPedalType::Fc7,
            foot_pedal_2_live_set: FootPedalLiveSet::Off,
            filter_eg_reset: true,
            effect_on_off_reset: true,
            usb_audio_volume: 0x40,
            bluetooth_volume: 0x40,
            ad_volume: 0x01,
            usb_audio_loopback: false,
            speaker_mode: SpeakerMode::Normal,
            speaker_mute: SpeakerMute::Auto,
            reserved: Vec::new(),
        }
    }
}

impl SystemCommon {
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() < SYSTEM_COMMON_LEN {
            return Err(CodecError::WrongLength {
                expected: SYSTEM_COMMON_LEN,
                actual: b.len(),
            });
        }
        let mut value = Self {
            master_tune: read_u16_nibbles(&b[0x02..0x06]),
            keyboard_octave_shift: signed_center(b[0x06], 0x40, -3, 3, "keyboard_octave_shift")?,
            keyboard_transpose: signed_center(b[0x07], 0x40, -12, 12, "keyboard_transpose")?,
            controller_reset: ControllerReset::from_byte(b[0x08])?,
            local_control: bool_byte(b[0x09], "local_control")?,
            tx_channel: ranged(b[0x0A], 0x00, 0x7F, "tx_channel")?,
            rx_channel: ranged(b[0x0B], 0x00, 0x10, "rx_channel")?,
            midi_control: bool_byte(b[0x0C], "midi_control")?,
            output_gain: ranged(b[0x0E], 0x38, 0x48, "output_gain")?,
            touch_curve: TouchCurve::from_byte(b[0x10])?,
            fixed_velocity: ranged(b[0x11], 0x01, 0x7F, "fixed_velocity")?,
            tx_rx_bank_select: bool_byte(b[0x12], "tx_rx_bank_select")?,
            tx_rx_program_change: bool_byte(b[0x13], "tx_rx_program_change")?,
            midi_in_out: bool_byte(b[0x15], "midi_in_out")?,
            usb_in_out: bool_byte(b[0x16], "usb_in_out")?,
            value_indication: bool_byte(b[0x1B], "value_indication")?,
            controller_mode: ControllerMode::from_byte(b[0x1C])?,
            lcd_switch: bool_byte(b[0x1E], "lcd_switch")?,
            lcd_contrast: signed_center(b[0x1F], 0x40, -10, 10, "lcd_contrast")?,
            panel_lock_live_set: bool_byte(b[0x20], "panel_lock_live_set")?,
            panel_lock_organ: bool_byte(b[0x21], "panel_lock_organ")?,
            panel_lock_filter_eg: bool_byte(b[0x22], "panel_lock_filter_eg")?,
            panel_lock_drive_effect: bool_byte(b[0x23], "panel_lock_drive_effect")?,
            panel_lock_delay_reverb: bool_byte(b[0x24], "panel_lock_delay_reverb")?,
            panel_lock_equalizer: bool_byte(b[0x25], "panel_lock_equalizer")?,
            power_on_page: ranged(b[0x28], 0x00, 0x13, "power_on_page")?,
            power_on_sound: ranged(b[0x29], 0x00, 0x07, "power_on_sound")?,
            foot_pedal_1_type: FootPedalType::from_byte(b[0x2A])?,
            foot_pedal_1_live_set: FootPedalLiveSet::from_byte(b[0x2B])?,
            foot_pedal_2_type: FootPedalType::from_byte(b[0x2C])?,
            foot_pedal_2_live_set: FootPedalLiveSet::from_byte(b[0x2D])?,
            filter_eg_reset: bool_byte(b[0x2E], "filter_eg_reset")?,
            effect_on_off_reset: bool_byte(b[0x2F], "effect_on_off_reset")?,
            usb_audio_volume: ranged(b[0x31], 0x00, 0x7F, "usb_audio_volume")?,
            bluetooth_volume: ranged(b[0x32], 0x00, 0x7F, "bluetooth_volume")?,
            ad_volume: ranged(b[0x33], 0x00, 0x7F, "ad_volume")?,
            usb_audio_loopback: bool_byte(b[0x34], "usb_audio_loopback")?,
            speaker_mode: SpeakerMode::from_byte(b[0x36])?,
            speaker_mute: SpeakerMute::from_byte(b[0x37])?,
            reserved: Vec::new(),
        };
        let typed_only = value.to_bytes()?;
        value.reserved = capture_reserved(b, &typed_only);
        Ok(value)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        let mut b = vec![0u8; SYSTEM_COMMON_LEN];
        b[0x02..0x06].copy_from_slice(&write_u16_nibbles(self.master_tune));
        b[0x06] = to_signed_center(
            self.keyboard_octave_shift,
            0x40,
            -3,
            3,
            "keyboard_octave_shift",
        )?;
        b[0x07] = to_signed_center(self.keyboard_transpose, 0x40, -12, 12, "keyboard_transpose")?;
        b[0x08] = self.controller_reset.to_byte();
        b[0x09] = self.local_control as u8;
        b[0x0A] = ranged(self.tx_channel, 0x00, 0x7F, "tx_channel")?;
        b[0x0B] = ranged(self.rx_channel, 0x00, 0x10, "rx_channel")?;
        b[0x0C] = self.midi_control as u8;
        b[0x0E] = ranged(self.output_gain, 0x38, 0x48, "output_gain")?;
        b[0x10] = self.touch_curve.to_byte();
        b[0x11] = ranged(self.fixed_velocity, 0x01, 0x7F, "fixed_velocity")?;
        b[0x12] = self.tx_rx_bank_select as u8;
        b[0x13] = self.tx_rx_program_change as u8;
        b[0x15] = self.midi_in_out as u8;
        b[0x16] = self.usb_in_out as u8;
        b[0x1B] = self.value_indication as u8;
        b[0x1C] = self.controller_mode.to_byte();
        b[0x1E] = self.lcd_switch as u8;
        b[0x1F] = to_signed_center(self.lcd_contrast, 0x40, -10, 10, "lcd_contrast")?;
        b[0x20] = self.panel_lock_live_set as u8;
        b[0x21] = self.panel_lock_organ as u8;
        b[0x22] = self.panel_lock_filter_eg as u8;
        b[0x23] = self.panel_lock_drive_effect as u8;
        b[0x24] = self.panel_lock_delay_reverb as u8;
        b[0x25] = self.panel_lock_equalizer as u8;
        b[0x28] = ranged(self.power_on_page, 0x00, 0x13, "power_on_page")?;
        b[0x29] = ranged(self.power_on_sound, 0x00, 0x07, "power_on_sound")?;
        b[0x2A] = self.foot_pedal_1_type.to_byte();
        b[0x2B] = self.foot_pedal_1_live_set.to_byte();
        b[0x2C] = self.foot_pedal_2_type.to_byte();
        b[0x2D] = self.foot_pedal_2_live_set.to_byte();
        b[0x2E] = self.filter_eg_reset as u8;
        b[0x2F] = self.effect_on_off_reset as u8;
        b[0x31] = ranged(self.usb_audio_volume, 0x00, 0x7F, "usb_audio_volume")?;
        b[0x32] = ranged(self.bluetooth_volume, 0x00, 0x7F, "bluetooth_volume")?;
        b[0x33] = ranged(self.ad_volume, 0x00, 0x7F, "ad_volume")?;
        b[0x34] = self.usb_audio_loopback as u8;
        b[0x36] = self.speaker_mode.to_byte();
        b[0x37] = self.speaker_mute.to_byte();
        apply_reserved(&mut b, &self.reserved);
        Ok(b)
    }
}

/// The System **Master EQ** block — a 3-band EQ.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// Restores the per-field `"default":` entries in the emitted JSON Schema. Only
// under `schema`: `tsify` reads a container default as "every field optional" and
// would loosen the editor's TypeScript. Partial-document leniency lives at the
// document boundary instead (see `document::from_value_over_default`).
#[cfg_attr(feature = "schema", serde(default))]
pub struct MasterEq {
    /// Low gain, −12..=+12 dB.
    #[cfg_attr(feature = "schema", schemars(range(min = -12, max = 12)))]
    pub low_gain: i8,
    /// Low frequency, raw `0x04..=0x28` (32 Hz – 2.0 kHz).
    #[cfg_attr(feature = "schema", schemars(range(min = 4, max = 40)))]
    pub low_freq: u8,
    /// Mid gain, −12..=+12 dB.
    #[cfg_attr(feature = "schema", schemars(range(min = -12, max = 12)))]
    pub mid_gain: i8,
    /// Mid frequency, raw `0x0E..=0x36` (100 Hz – 10 kHz).
    #[cfg_attr(feature = "schema", schemars(range(min = 14, max = 54)))]
    pub mid_freq: u8,
    /// High gain, −12..=+12 dB.
    #[cfg_attr(feature = "schema", schemars(range(min = -12, max = 12)))]
    pub high_gain: i8,
    /// High frequency, raw `0x1C..=0x3A` (500 Hz – 16 kHz).
    #[cfg_attr(feature = "schema", schemars(range(min = 28, max = 58)))]
    pub high_freq: u8,
    /// Reserved/undocumented bytes captured verbatim so writes round-trip exactly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reserved: Vec<RawByte>,
}

impl Default for MasterEq {
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

impl MasterEq {
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() < MASTER_EQ_LEN {
            return Err(CodecError::WrongLength {
                expected: MASTER_EQ_LEN,
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
        let mut b = vec![0u8; MASTER_EQ_LEN];
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

    /// A defaults-derived System Common payload (reserved bytes = 0).
    fn default_common_bytes() -> Vec<u8> {
        let mut b = vec![0u8; SYSTEM_COMMON_LEN];
        b[0x02..0x06].copy_from_slice(&[0x00, 0x04, 0x00, 0x00]); // master tune
        b[0x06] = 0x40; // octave 0
        b[0x07] = 0x40; // transpose 0
        b[0x08] = 0x01; // controller reset = reset
        b[0x09] = 0x01;
        b[0x0A] = 0x00;
        b[0x0B] = 0x00;
        b[0x0C] = 0x00;
        b[0x0E] = 0x3E;
        b[0x10] = 0x00;
        b[0x11] = 0x40;
        b[0x12] = 0x01;
        b[0x13] = 0x01;
        b[0x15] = 0x01;
        b[0x16] = 0x01;
        b[0x1B] = 0x01;
        b[0x1C] = 0x00;
        b[0x1E] = 0x01;
        b[0x1F] = 0x40;
        b[0x20] = 0x01;
        b[0x21] = 0x01;
        b[0x22] = 0x01;
        b[0x23] = 0x01;
        b[0x24] = 0x01;
        b[0x25] = 0x01;
        b[0x2C] = 0x03; // foot pedal 2 type default
        b[0x2E] = 0x01;
        b[0x2F] = 0x01;
        b[0x31] = 0x40;
        b[0x32] = 0x40;
        b[0x33] = 0x01;
        b
    }

    #[test]
    fn system_common_round_trips() {
        let bytes = default_common_bytes();
        let decoded = SystemCommon::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.keyboard_transpose, 0);
        assert_eq!(decoded.output_gain, 0x3E);
        assert_eq!(decoded.controller_reset, ControllerReset::Reset);
        assert_eq!(decoded.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn system_common_preserves_trailing_firmware_bytes() {
        let mut bytes = default_common_bytes();
        bytes.extend_from_slice(&[0x11, 0x22]); // pretend newer firmware appended params
        let decoded = SystemCommon::from_bytes(&bytes).unwrap();
        // Trailing bytes are captured as reserved entries and round-trip exactly.
        assert!(decoded
            .reserved
            .iter()
            .any(|r| r.offset == 56 && r.value == 0x11));
        assert_eq!(decoded.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn system_common_preserves_nonzero_reserved_offset() {
        let mut bytes = default_common_bytes();
        bytes[0x0D] = 0x05; // a documented-reserved offset the device populates
        let decoded = SystemCommon::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn master_eq_round_trips() {
        let mut b = vec![0u8; MASTER_EQ_LEN];
        b[0x00] = 0x40;
        b[0x01] = 0x0C;
        b[0x08] = 0x40;
        b[0x09] = 0x22;
        b[0x10] = 0x40;
        b[0x11] = 0x30;
        let eq = MasterEq::from_bytes(&b).unwrap();
        assert_eq!(eq.low_gain, 0);
        assert_eq!(eq.mid_freq, 0x22);
        assert_eq!(eq.to_bytes().unwrap(), b);
    }

    #[test]
    fn yaml_renders_named_enums() {
        let s = SystemCommon::from_bytes(&default_common_bytes()).unwrap();
        let yaml = serde_yaml::to_string(&s).unwrap();
        assert!(yaml.contains("controller_reset: reset"));
        assert!(yaml.contains("touch_curve: normal"));
    }
}
