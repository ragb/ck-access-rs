//! Live Set **Zone** block (`4A zz 00`, zz = 0..=3, 16 bytes).
//!
//! A Zone is the CK's master-keyboard MIDI transmit configuration: channel,
//! transpose, note range, and which controllers it relays. Offsets/ranges from
//! the *Owner's Manual* "MIDI Data Table → ZONE".

use serde::{Deserialize, Serialize};

use crate::address::ZONE_LEN;
use crate::codec::{
    apply_reserved, bool_byte, capture_reserved, ranged, signed_center, to_signed_center,
    CodecError, RawByte,
};

/// One Zone.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    pub zone_switch: bool,
    /// Transmit channel, raw `0..=0x0F` (= MIDI ch 1–16).
    pub transmit_channel: u8,
    /// Transpose (octave), −3..=+3.
    pub transpose_octave: i8,
    /// Transpose (semitone), −11..=+11.
    pub transpose_semitone: i8,
    /// Note limit low, 0..=127 (C-2 .. G8).
    pub note_limit_low: u8,
    /// Note limit high, 0..=127 (C-2 .. G8).
    pub note_limit_high: u8,
    /// MIDI volume, 0..=127.
    pub midi_volume: u8,
    /// MIDI pan, −64..=+63 (L64 .. C .. R63).
    pub midi_pan: i8,
    /// MIDI bank select MSB, 0..=127.
    pub midi_bank_msb: u8,
    /// MIDI bank select LSB, 0..=127.
    pub midi_bank_lsb: u8,
    /// MIDI program number, raw 0..=127 (program 1–128).
    pub midi_program: u8,
    /// Transmit-enable bitfield, raw `0x00..=0x1F`:
    /// bit0 Bank Select, bit1 Program Change, bit2 Volume, bit3 Pan.
    pub transmit_flags: u8,
    /// Controller transmit bitfield, raw `0x00..=0x3F`:
    /// bit0 Pitch Bend, bit1 Modulation, bit2 Foot Pedal 1, bit3 Foot Pedal 2.
    pub transmit_controller_flags: u8,
    /// Reserved/undocumented bytes captured verbatim so writes round-trip exactly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reserved: Vec<RawByte>,
}

impl Zone {
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() < ZONE_LEN {
            return Err(CodecError::WrongLength {
                expected: ZONE_LEN,
                actual: b.len(),
            });
        }
        let mut value = Self {
            zone_switch: bool_byte(b[0x00], "zone_switch")?,
            transmit_channel: ranged(b[0x01], 0x00, 0x0F, "transmit_channel")?,
            transpose_octave: signed_center(b[0x02], 0x40, -3, 3, "transpose_octave")?,
            transpose_semitone: signed_center(b[0x03], 0x40, -11, 11, "transpose_semitone")?,
            note_limit_low: ranged(b[0x04], 0x00, 0x7F, "note_limit_low")?,
            note_limit_high: ranged(b[0x05], 0x00, 0x7F, "note_limit_high")?,
            midi_volume: ranged(b[0x07], 0x00, 0x7F, "midi_volume")?,
            midi_pan: signed_center(b[0x08], 0x40, -64, 63, "midi_pan")?,
            midi_bank_msb: ranged(b[0x09], 0x00, 0x7F, "midi_bank_msb")?,
            midi_bank_lsb: ranged(b[0x0A], 0x00, 0x7F, "midi_bank_lsb")?,
            midi_program: ranged(b[0x0B], 0x00, 0x7F, "midi_program")?,
            transmit_flags: ranged(b[0x0C], 0x00, 0x1F, "transmit_flags")?,
            transmit_controller_flags: ranged(b[0x0D], 0x00, 0x3F, "transmit_controller_flags")?,
            reserved: Vec::new(),
        };
        let typed_only = value.to_bytes()?;
        value.reserved = capture_reserved(b, &typed_only);
        Ok(value)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        let mut b = vec![0u8; ZONE_LEN];
        b[0x00] = self.zone_switch as u8;
        b[0x01] = ranged(self.transmit_channel, 0x00, 0x0F, "transmit_channel")?;
        b[0x02] = to_signed_center(self.transpose_octave, 0x40, -3, 3, "transpose_octave")?;
        b[0x03] = to_signed_center(self.transpose_semitone, 0x40, -11, 11, "transpose_semitone")?;
        b[0x04] = ranged(self.note_limit_low, 0x00, 0x7F, "note_limit_low")?;
        b[0x05] = ranged(self.note_limit_high, 0x00, 0x7F, "note_limit_high")?;
        b[0x07] = ranged(self.midi_volume, 0x00, 0x7F, "midi_volume")?;
        b[0x08] = to_signed_center(self.midi_pan, 0x40, -64, 63, "midi_pan")?;
        b[0x09] = ranged(self.midi_bank_msb, 0x00, 0x7F, "midi_bank_msb")?;
        b[0x0A] = ranged(self.midi_bank_lsb, 0x00, 0x7F, "midi_bank_lsb")?;
        b[0x0B] = ranged(self.midi_program, 0x00, 0x7F, "midi_program")?;
        b[0x0C] = ranged(self.transmit_flags, 0x00, 0x1F, "transmit_flags")?;
        b[0x0D] = ranged(
            self.transmit_controller_flags,
            0x00,
            0x3F,
            "transmit_controller_flags",
        )?;
        apply_reserved(&mut b, &self.reserved);
        Ok(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_zone_bytes() -> Vec<u8> {
        let mut b = vec![0u8; ZONE_LEN];
        b[0x00] = 0x01; // on
        b[0x01] = 0x00; // ch 1
        b[0x02] = 0x40; // octave 0
        b[0x03] = 0x40; // semitone 0
        b[0x04] = 0x00;
        b[0x05] = 0x7F;
        b[0x07] = 0x7F;
        b[0x08] = 0x40; // pan center
        b[0x0C] = 0x1F;
        b[0x0D] = 0x0F;
        b
    }

    #[test]
    fn zone_round_trips() {
        let bytes = default_zone_bytes();
        let z = Zone::from_bytes(&bytes).unwrap();
        assert!(z.zone_switch);
        assert_eq!(z.midi_pan, 0);
        assert_eq!(z.transpose_octave, 0);
        assert_eq!(z.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn zone_pan_extremes() {
        let mut bytes = default_zone_bytes();
        bytes[0x08] = 0x00;
        assert_eq!(Zone::from_bytes(&bytes).unwrap().midi_pan, -64);
        bytes[0x08] = 0x7F;
        assert_eq!(Zone::from_bytes(&bytes).unwrap().midi_pan, 63);
    }
}
