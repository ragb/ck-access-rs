//! Parameter address map for the CK.
//!
//! Group Number `7F 1C`, Model ID `0B`. Addresses are 3 bytes (High, Mid, Low).
//! From the *Owner's Manual* "Parameter Base Address" and "Bulk Dump Block"
//! tables:
//!
//! | Block                | High | Mid  | Low | Bytes |
//! |----------------------|------|------|-----|-------|
//! | System Common        | `20` | `00` | `00`| 56    |
//! | System Master EQ     | `20` | `40` | `00`| 20    |
//! | Soundmondo Version   | `00` | `7F` | `00`| 4     |
//! | Bulk Header          | `0E` | `pp` | `0n`| —     |
//! | Bulk Footer          | `0F` | `pp` | `0n`| —     |
//! | Store To Flash       | `0D` | `00` | `00`| —     |
//! | Live Set Common      | `46` | `00` | `00`| 83    |
//! | Live Set EQ          | `46` | `40` | `00`| 20    |
//! | Audio Trigger Path   | `46` | `10` | `00`| 255   |
//! | Zone (zz = 00..=03)  | `4A` | `zz` | `00`| 16    |
//! | Live Set Part (p)    | `50` | `0p` | `00`| 105   |
//!
//! For Bulk Header/Footer, `pp` = Live Set Sound user page (0..=19) and
//! `0n` = part/section selector; both are 0 when addressing the *current* edit
//! buffer.

/// Number of Zones in a Live Set (`zz` = 0..=3).
pub const ZONE_COUNT: u8 = 4;

// Block base addresses (current edit buffer).
pub const SYSTEM_COMMON_BASE: [u8; 3] = [0x20, 0x00, 0x00];
pub const MASTER_EQ_BASE: [u8; 3] = [0x20, 0x40, 0x00];
pub const SOUNDMONDO_VERSION_BASE: [u8; 3] = [0x00, 0x7F, 0x00];
pub const STORE_TO_FLASH_BASE: [u8; 3] = [0x0D, 0x00, 0x00];
pub const BULK_HEADER_BASE: [u8; 3] = [0x0E, 0x00, 0x00];
pub const BULK_FOOTER_BASE: [u8; 3] = [0x0F, 0x00, 0x00];
pub const LIVE_SET_COMMON_BASE: [u8; 3] = [0x46, 0x00, 0x00];
pub const LIVE_SET_EQ_BASE: [u8; 3] = [0x46, 0x40, 0x00];
pub const AUDIO_TRIGGER_PATH_BASE: [u8; 3] = [0x46, 0x10, 0x00];
/// Rotary Speaker block, added in firmware v1.10.
pub const ROTARY_BASE: [u8; 3] = [0x46, 0x20, 0x00];

// Documented block byte counts (firmware v1.0). Newer firmware may append
// parameters; decoders accept longer payloads (see each area's `from_bytes`).
pub const SYSTEM_COMMON_LEN: usize = 56;
pub const MASTER_EQ_LEN: usize = 20;
pub const LIVE_SET_COMMON_LEN: usize = 83;
pub const LIVE_SET_EQ_LEN: usize = 20;
pub const ZONE_LEN: usize = 16;
pub const PART_LEN: usize = 105;
pub const ROTARY_LEN: usize = 24;
pub const AUDIO_TRIGGER_PATH_LEN: usize = 255;

/// The three Live Set parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    A,
    B,
    C,
}

impl Part {
    /// Mid-address selector `0p` (A=0, B=1, C=2).
    pub fn index(self) -> u8 {
        match self {
            Part::A => 0,
            Part::B => 1,
            Part::C => 2,
        }
    }

    /// Base address of this part's 105-byte block: `50 0p 00`.
    pub fn base_address(self) -> [u8; 3] {
        [0x50, self.index(), 0x00]
    }

    pub fn from_index(i: u8) -> Option<Self> {
        match i {
            0 => Some(Part::A),
            1 => Some(Part::B),
            2 => Some(Part::C),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Part::A => "A",
            Part::B => "B",
            Part::C => "C",
        }
    }
}

/// Base address of Zone `zz` (0..=3): `4A zz 00`.
pub fn zone_base_address(zz: u8) -> Option<[u8; 3]> {
    (zz < ZONE_COUNT).then_some([0x4A, zz, 0x00])
}

/// Top-level partition of the CK address space, keyed off the address-High byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressSpace {
    /// `00 7F ..` — Soundmondo format version.
    SoundmondoVersion,
    /// `0D/0E/0F ..` — bulk control (store-to-flash, header, footer).
    BulkControl,
    /// `20 ..` — System (Common + Master EQ).
    System,
    /// `46 ..` — Live Set Common (Common, EQ, Audio Trigger).
    LiveSetCommon,
    /// `4A ..` — a Zone.
    Zone,
    /// `50 ..` — a Live Set Part.
    Part,
    /// Anything we haven't classified.
    Unknown,
}

impl AddressSpace {
    pub fn classify(address: [u8; 3]) -> Self {
        match address[0] {
            0x00 if address[1] == 0x7F => Self::SoundmondoVersion,
            0x0D..=0x0F => Self::BulkControl,
            0x20 => Self::System,
            0x46 => Self::LiveSetCommon,
            0x4A => Self::Zone,
            0x50 => Self::Part,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_addresses_match_spec() {
        assert_eq!(Part::A.base_address(), [0x50, 0x00, 0x00]);
        assert_eq!(Part::B.base_address(), [0x50, 0x01, 0x00]);
        assert_eq!(Part::C.base_address(), [0x50, 0x02, 0x00]);
    }

    #[test]
    fn zone_addresses_match_spec() {
        assert_eq!(zone_base_address(0), Some([0x4A, 0x00, 0x00]));
        assert_eq!(zone_base_address(3), Some([0x4A, 0x03, 0x00]));
        assert_eq!(zone_base_address(4), None);
    }

    #[test]
    fn classify_known_addresses() {
        assert_eq!(
            AddressSpace::classify(SYSTEM_COMMON_BASE),
            AddressSpace::System
        );
        assert_eq!(AddressSpace::classify(MASTER_EQ_BASE), AddressSpace::System);
        assert_eq!(
            AddressSpace::classify(LIVE_SET_COMMON_BASE),
            AddressSpace::LiveSetCommon
        );
        assert_eq!(
            AddressSpace::classify([0x4A, 0x00, 0x00]),
            AddressSpace::Zone
        );
        assert_eq!(
            AddressSpace::classify([0x50, 0x00, 0x00]),
            AddressSpace::Part
        );
        assert_eq!(
            AddressSpace::classify([0x77, 0x00, 0x00]),
            AddressSpace::Unknown
        );
    }
}
