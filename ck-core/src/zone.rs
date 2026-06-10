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

fn u8_is_zero(v: &u8) -> bool {
    *v == 0
}

/// Which "snapshot" MIDI messages the Zone sends when its Live Set is loaded.
///
/// The byte at Zone offset `0x0C` is a bitmask. Bits 0–3 are named in the
/// CK Owner's Manual; bit 4 sits inside the manual's documented `0x00..=0x1F`
/// range without a published meaning. Any higher bits a device emits
/// (defensive) are kept verbatim in `reserved_bits` so the block round-trips
/// byte-exact.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneTransmit {
    /// Bit 0: transmit Bank Select MSB+LSB.
    pub bank_select: bool,
    /// Bit 1: transmit Program Change.
    pub program_change: bool,
    /// Bit 2: transmit Channel Volume (CC 7).
    pub volume: bool,
    /// Bit 3: transmit Pan (CC 10).
    pub pan: bool,
    /// Bits 4..=7 captured verbatim — bit 4 falls inside the manual's
    /// documented `0x1F` range but isn't named; higher bits aren't expected
    /// but are preserved defensively.
    #[serde(default, skip_serializing_if = "u8_is_zero")]
    pub reserved_bits: u8,
}

impl ZoneTransmit {
    pub fn from_byte(b: u8) -> Self {
        Self {
            bank_select: b & 0x01 != 0,
            program_change: b & 0x02 != 0,
            volume: b & 0x04 != 0,
            pan: b & 0x08 != 0,
            reserved_bits: b & 0xF0,
        }
    }

    pub fn to_byte(self) -> u8 {
        (self.bank_select as u8)
            | (self.program_change as u8) << 1
            | (self.volume as u8) << 2
            | (self.pan as u8) << 3
            | (self.reserved_bits & 0xF0)
    }
}

/// Which live controllers the Zone forwards to its transmit channel.
///
/// Byte at Zone offset `0x0D`. Bits 0–3 are documented; bits 4–5 fall inside
/// the manual's `0x00..=0x3F` documented range without published meaning;
/// higher bits are unexpected but preserved.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneTransmitControllers {
    /// Bit 0: transmit Pitch Bend.
    pub pitch_bend: bool,
    /// Bit 1: transmit Modulation Wheel (CC 1).
    pub modulation: bool,
    /// Bit 2: transmit Foot Pedal 1.
    pub foot_pedal_1: bool,
    /// Bit 3: transmit Foot Pedal 2.
    pub foot_pedal_2: bool,
    /// Bits 4..=7 captured verbatim — bits 4 and 5 sit inside the manual's
    /// `0x3F` documented range without a published meaning.
    #[serde(default, skip_serializing_if = "u8_is_zero")]
    pub reserved_bits: u8,
}

impl ZoneTransmitControllers {
    pub fn from_byte(b: u8) -> Self {
        Self {
            pitch_bend: b & 0x01 != 0,
            modulation: b & 0x02 != 0,
            foot_pedal_1: b & 0x04 != 0,
            foot_pedal_2: b & 0x08 != 0,
            reserved_bits: b & 0xF0,
        }
    }

    pub fn to_byte(self) -> u8 {
        (self.pitch_bend as u8)
            | (self.modulation as u8) << 1
            | (self.foot_pedal_1 as u8) << 2
            | (self.foot_pedal_2 as u8) << 3
            | (self.reserved_bits & 0xF0)
    }
}

/// One Zone.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    /// Enables this Zone's MIDI transmission.
    pub zone_switch: bool,
    /// Transmit channel, raw `0..=0x0F` (= MIDI ch 1–16).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 15)))]
    pub transmit_channel: u8,
    /// Transpose (octave), −3..=+3.
    #[cfg_attr(feature = "schema", schemars(range(min = -3, max = 3)))]
    pub transpose_octave: i8,
    /// Transpose (semitone), −11..=+11.
    #[cfg_attr(feature = "schema", schemars(range(min = -11, max = 11)))]
    pub transpose_semitone: i8,
    /// Note limit low, 0..=127 (C-2 .. G8).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub note_limit_low: u8,
    /// Note limit high, 0..=127 (C-2 .. G8).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub note_limit_high: u8,
    /// MIDI volume, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub midi_volume: u8,
    /// MIDI pan, −64..=+63 (L64 .. C .. R63).
    #[cfg_attr(feature = "schema", schemars(range(min = -64, max = 63)))]
    pub midi_pan: i8,
    /// MIDI bank select MSB, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub midi_bank_msb: u8,
    /// MIDI bank select LSB, 0..=127.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub midi_bank_lsb: u8,
    /// MIDI program number, raw 0..=127 (program 1–128).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 127)))]
    pub midi_program: u8,
    /// Which snapshot messages (Bank/PC/Volume/Pan) this Zone sends.
    pub transmit: ZoneTransmit,
    /// Which live controllers this Zone forwards (Pitch Bend / Mod / FP1 / FP2).
    pub transmit_controllers: ZoneTransmitControllers,
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
            transmit: ZoneTransmit::from_byte(b[0x0C]),
            transmit_controllers: ZoneTransmitControllers::from_byte(b[0x0D]),
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
        b[0x0C] = self.transmit.to_byte();
        b[0x0D] = self.transmit_controllers.to_byte();
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
        // 0x1F = all four named bits + the in-range reserved bit 4.
        assert!(z.transmit.bank_select);
        assert!(z.transmit.program_change);
        assert!(z.transmit.volume);
        assert!(z.transmit.pan);
        assert_eq!(z.transmit.reserved_bits, 0x10);
        // 0x0F = all four named controllers, no reserved bits set.
        assert!(z.transmit_controllers.pitch_bend);
        assert!(z.transmit_controllers.modulation);
        assert!(z.transmit_controllers.foot_pedal_1);
        assert!(z.transmit_controllers.foot_pedal_2);
        assert_eq!(z.transmit_controllers.reserved_bits, 0x00);
        assert_eq!(z.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn transmit_flags_round_trip_each_bit() {
        for raw in 0u8..=0xFFu8 {
            let parsed = ZoneTransmit::from_byte(raw);
            assert_eq!(parsed.to_byte(), raw);
        }
        for raw in 0u8..=0xFFu8 {
            let parsed = ZoneTransmitControllers::from_byte(raw);
            assert_eq!(parsed.to_byte(), raw);
        }
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
