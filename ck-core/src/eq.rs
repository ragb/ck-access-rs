//! EQ frequency index catalog.
//!
//! Every EQ frequency field on the CK — System [`crate::system::MasterEq`],
//! [`crate::live_set::LiveSetEq`], and the A/D-input EQ inside
//! [`crate::live_set::LiveSetCommon`] — stores its centre frequency as a raw
//! byte index into one shared frequency table, with each band restricted to a
//! sub-range:
//!
//! | Band | Index range   | Frequency span |
//! |------|---------------|----------------|
//! | Low  | `0x04..=0x28` | 32 Hz – 2.0 kHz |
//! | Mid  | `0x0E..=0x36` | 100 Hz – 10 kHz |
//! | High | `0x1C..=0x3A` | 500 Hz – 16 kHz |
//!
//! The manual documents only those per-band endpoints, not the per-index table.
//! The six endpoints fall exactly on a **1/6-octave** series (index 4 = 32 Hz,
//! each step ×2^(1/6)), so the table below is that standard Yamaha/R20 series
//! anchored to the documented endpoints. The six endpoint labels are
//! manual-confirmed; the intermediate labels follow the standard series and are
//! not individually device-verified.

/// One EQ frequency step: its table index and the centre frequency it selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EqFreq {
    pub index: u8,
    /// Centre frequency in Hz.
    pub hz: u32,
    /// Panel-style label, e.g. `"32 Hz"`, `"2.0 kHz"`.
    pub label: &'static str,
}

/// Which EQ band a frequency field belongs to (constrains the usable index range).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EqBand {
    Low,
    Mid,
    High,
}

impl EqBand {
    /// Inclusive `(lo, hi)` index range this band's frequency field accepts.
    pub fn index_range(self) -> (u8, u8) {
        match self {
            EqBand::Low => (0x04, 0x28),
            EqBand::Mid => (0x0E, 0x36),
            EqBand::High => (0x1C, 0x3A),
        }
    }

    pub fn from_index(i: u8) -> Option<Self> {
        match i {
            0 => Some(EqBand::Low),
            1 => Some(EqBand::Mid),
            2 => Some(EqBand::High),
            _ => None,
        }
    }
}

const fn ef(index: u8, hz: u32, label: &'static str) -> EqFreq {
    EqFreq { index, hz, label }
}

/// The shared EQ frequency table, indexed by the stored byte (`0x00..=0x3A`).
pub static EQ_FREQUENCIES: &[EqFreq] = &[
    ef(0x00, 20, "20 Hz"),
    ef(0x01, 22, "22 Hz"),
    ef(0x02, 25, "25 Hz"),
    ef(0x03, 28, "28 Hz"),
    ef(0x04, 32, "32 Hz"),
    ef(0x05, 36, "36 Hz"),
    ef(0x06, 40, "40 Hz"),
    ef(0x07, 45, "45 Hz"),
    ef(0x08, 50, "50 Hz"),
    ef(0x09, 56, "56 Hz"),
    ef(0x0A, 63, "63 Hz"),
    ef(0x0B, 71, "71 Hz"),
    ef(0x0C, 80, "80 Hz"),
    ef(0x0D, 90, "90 Hz"),
    ef(0x0E, 100, "100 Hz"),
    ef(0x0F, 112, "112 Hz"),
    ef(0x10, 125, "125 Hz"),
    ef(0x11, 140, "140 Hz"),
    ef(0x12, 160, "160 Hz"),
    ef(0x13, 180, "180 Hz"),
    ef(0x14, 200, "200 Hz"),
    ef(0x15, 224, "224 Hz"),
    ef(0x16, 250, "250 Hz"),
    ef(0x17, 280, "280 Hz"),
    ef(0x18, 315, "315 Hz"),
    ef(0x19, 355, "355 Hz"),
    ef(0x1A, 400, "400 Hz"),
    ef(0x1B, 450, "450 Hz"),
    ef(0x1C, 500, "500 Hz"),
    ef(0x1D, 560, "560 Hz"),
    ef(0x1E, 630, "630 Hz"),
    ef(0x1F, 710, "710 Hz"),
    ef(0x20, 800, "800 Hz"),
    ef(0x21, 900, "900 Hz"),
    ef(0x22, 1000, "1.0 kHz"),
    ef(0x23, 1120, "1.12 kHz"),
    ef(0x24, 1250, "1.25 kHz"),
    ef(0x25, 1400, "1.4 kHz"),
    ef(0x26, 1600, "1.6 kHz"),
    ef(0x27, 1800, "1.8 kHz"),
    ef(0x28, 2000, "2.0 kHz"),
    ef(0x29, 2240, "2.24 kHz"),
    ef(0x2A, 2500, "2.5 kHz"),
    ef(0x2B, 2800, "2.8 kHz"),
    ef(0x2C, 3150, "3.15 kHz"),
    ef(0x2D, 3550, "3.55 kHz"),
    ef(0x2E, 4000, "4.0 kHz"),
    ef(0x2F, 4500, "4.5 kHz"),
    ef(0x30, 5000, "5.0 kHz"),
    ef(0x31, 5600, "5.6 kHz"),
    ef(0x32, 6300, "6.3 kHz"),
    ef(0x33, 7100, "7.1 kHz"),
    ef(0x34, 8000, "8.0 kHz"),
    ef(0x35, 9000, "9.0 kHz"),
    ef(0x36, 10000, "10.0 kHz"),
    ef(0x37, 11200, "11.2 kHz"),
    ef(0x38, 12500, "12.5 kHz"),
    ef(0x39, 14000, "14.0 kHz"),
    ef(0x3A, 16000, "16.0 kHz"),
];

/// Centre frequency in Hz for a table index, or `None` if out of range.
pub fn eq_freq_hz(index: u8) -> Option<u32> {
    EQ_FREQUENCIES.get(index as usize).map(|f| f.hz)
}

/// Panel label for a table index, or `None` if out of range.
pub fn eq_freq_label(index: u8) -> Option<&'static str> {
    EQ_FREQUENCIES.get(index as usize).map(|f| f.label)
}

/// The frequency steps selectable for a given band (its dropdown options).
pub fn eq_band_frequencies(band: EqBand) -> &'static [EqFreq] {
    let (lo, hi) = band.index_range();
    &EQ_FREQUENCIES[lo as usize..=hi as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_documented_endpoints() {
        // The six band endpoints the Owner's Manual states explicitly.
        assert_eq!(eq_freq_label(0x04), Some("32 Hz")); // Low min
        assert_eq!(eq_freq_label(0x28), Some("2.0 kHz")); // Low max
        assert_eq!(eq_freq_label(0x0E), Some("100 Hz")); // Mid min
        assert_eq!(eq_freq_label(0x36), Some("10.0 kHz")); // Mid max
        assert_eq!(eq_freq_label(0x1C), Some("500 Hz")); // High min
        assert_eq!(eq_freq_label(0x3A), Some("16.0 kHz")); // High max
        assert_eq!(eq_freq_hz(0x04), Some(32));
        assert_eq!(eq_freq_hz(0x3A), Some(16000));
        assert_eq!(eq_freq_label(0x3B), None);
    }

    #[test]
    fn table_is_dense_ordered_and_increasing() {
        for (i, f) in EQ_FREQUENCIES.iter().enumerate() {
            assert_eq!(f.index as usize, i);
        }
        for w in EQ_FREQUENCIES.windows(2) {
            assert!(w[0].hz < w[1].hz, "frequencies must strictly increase");
        }
        assert_eq!(EQ_FREQUENCIES.len(), 0x3B); // 0x00..=0x3A
    }

    #[test]
    fn band_options_match_ranges() {
        let low = eq_band_frequencies(EqBand::Low);
        assert_eq!(low.first().unwrap().label, "32 Hz");
        assert_eq!(low.last().unwrap().label, "2.0 kHz");
        let high = eq_band_frequencies(EqBand::High);
        assert_eq!(high.first().unwrap().label, "500 Hz");
        assert_eq!(high.last().unwrap().label, "16.0 kHz");
        assert_eq!(EqBand::Mid.index_range(), (0x0E, 0x36));
    }
}
