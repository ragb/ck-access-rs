//! Parameter address map for the CK.
//!
//! Group Number `7F 1C`, Model ID `0B`. Addresses are 3 bytes (High, Mid, Low).
//! From the *Owner's Manual* "Parameter Base Address" and "Bulk Dump Block"
//! tables (pp. 56–59):
//!
//! | Block                | High | Mid  | Low | Bytes |
//! |----------------------|------|------|-----|-------|
//! | System Common        | `20` | `00` | `00`| 56    |
//! | System Master EQ     | `20` | `40` | `00`| 20    |
//! | Soundmondo Version   | `00` | `7F` | `00`| 4     |
//! | Bulk Header (slot)   | `0E` | `pp` | `0n`| —     |
//! | Bulk Header (buffer) | `0E` | `7F` | `00`| —     |
//! | Bulk Footer (slot)   | `0F` | `pp` | `0n`| —     |
//! | Bulk Footer (buffer) | `0F` | `7F` | `00`| —     |
//! | Store To Flash       | `0B` | `10` | `00`| 6     |
//! | Live Set Common      | `46` | `00` | `00`| 83    |
//! | Live Set EQ          | `46` | `40` | `00`| 20    |
//! | Audio Trigger Path   | `46` | `10` | `00`| 255   |
//! | Zone (zz = 00..=03)  | `4A` | `zz` | `00`| 16    |
//! | Live Set Part (p)    | `50` | `0p` | `00`| 105   |
//!
//! Bulk Header / Footer addresses come in two variants. The `pp`/`0n` form
//! brackets a bulk dump targeting a stored User Live Set Sound slot
//! (`pp` = page 0..=19, `n` = sound 0..=7); the `7F 00` form brackets a dump
//! targeting the volatile edit buffer (whichever slot the CK is currently
//! playing).
//!
//! **Neither bulk dump persists on its own.** Both land in the CK's working
//! RAM — a slot-addressed dump reads back correctly while the keyboard stays
//! powered, but is lost on a power cycle. Committing to non-volatile flash is a
//! *separate* step: a **Store To Flash** Bulk Dump to `0B 10 00` naming the
//! destination slot (the panel STORE button's equivalent; see
//! [`STORE_TO_FLASH_BASE`] and [`crate::device::Ck::store_to_flash`]). To
//! persist a Live Set the way the Melas editor does: dump every block to the
//! edit buffer (`0E 7F 00` … `0F 7F 00`), then send the store.
//!
//! (The Owner's Manual lists a `0D 00 00` "Store To Flash" address, but a
//! capture of the Melas editor shows the store is actually the `0B 10 00`
//! Bulk Dump below; `0D 00 00` is unused.)

/// Number of Zones in a Live Set (`zz` = 0..=3).
pub const ZONE_COUNT: u8 = 4;

/// Number of Live Set Sound pages on the CK (User 1..=20).
pub const LIVE_SET_PAGE_COUNT: u8 = 20;
/// Live Set Sounds per page (1..=8).
pub const LIVE_SET_SOUNDS_PER_PAGE: u8 = 8;

// Block base addresses (current edit buffer).
pub const SYSTEM_COMMON_BASE: [u8; 3] = [0x20, 0x00, 0x00];
pub const MASTER_EQ_BASE: [u8; 3] = [0x20, 0x40, 0x00];
pub const SOUNDMONDO_VERSION_BASE: [u8; 3] = [0x00, 0x7F, 0x00];
/// Address of the Store To Flash Bulk Dump — sent after dumping a Live Set to
/// the edit buffer to commit it to a non-volatile slot (the panel STORE
/// button). A bulk dump alone only writes working RAM and is lost on a power
/// cycle. Its 6-byte payload names the destination slot; see
/// [`crate::device::Ck::store_to_flash`]. (Device-verified by capturing the
/// Melas CK editor — the manual's `0D 00 00` entry is unused.)
pub const STORE_TO_FLASH_BASE: [u8; 3] = [0x0B, 0x10, 0x00];

/// Bulk Header for the volatile edit buffer (whichever Live Set Sound the
/// CK is currently playing). Use this to bracket `sendLiveSet`-style pushes
/// that update the audible state without persisting to storage.
pub const BULK_HEADER_CURRENT_BUFFER: [u8; 3] = [0x0E, 0x7F, 0x00];
/// Bulk Footer for the volatile edit buffer. Pairs with
/// [`BULK_HEADER_CURRENT_BUFFER`].
pub const BULK_FOOTER_CURRENT_BUFFER: [u8; 3] = [0x0F, 0x7F, 0x00];

pub const LIVE_SET_COMMON_BASE: [u8; 3] = [0x46, 0x00, 0x00];
pub const LIVE_SET_EQ_BASE: [u8; 3] = [0x46, 0x40, 0x00];
pub const AUDIO_TRIGGER_PATH_BASE: [u8; 3] = [0x46, 0x10, 0x00];
/// Rotary Speaker block, added in firmware v1.10.
pub const ROTARY_BASE: [u8; 3] = [0x46, 0x20, 0x00];

/// Bulk Header that targets a stored User Live Set Sound slot. The dump
/// bracketed by this header (and the matching [`bulk_footer_for_slot`])
/// writes into that slot's working RAM — it reads back while the CK stays
/// powered, but does **not** survive a power cycle. Follow it with a Store To
/// Flash Bulk Dump ([`STORE_TO_FLASH_BASE`]) to persist, the way the panel
/// STORE button (and the Melas editor's "Store to…") does.
///
/// `page` and `sound` are 1-based to match the CK's display
/// (1..=`LIVE_SET_PAGE_COUNT`, 1..=`LIVE_SET_SOUNDS_PER_PAGE`).
/// Returns `None` if either is out of range.
pub fn bulk_header_for_slot(page: u8, sound: u8) -> Option<[u8; 3]> {
    if page == 0 || page > LIVE_SET_PAGE_COUNT {
        return None;
    }
    if sound == 0 || sound > LIVE_SET_SOUNDS_PER_PAGE {
        return None;
    }
    Some([0x0E, page - 1, sound - 1])
}

/// Bulk Footer that closes a slot-targeted bulk dump. Pairs with
/// [`bulk_header_for_slot`]; same range rules.
pub fn bulk_footer_for_slot(page: u8, sound: u8) -> Option<[u8; 3]> {
    if page == 0 || page > LIVE_SET_PAGE_COUNT {
        return None;
    }
    if sound == 0 || sound > LIVE_SET_SOUNDS_PER_PAGE {
        return None;
    }
    Some([0x0F, page - 1, sound - 1])
}

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

    #[test]
    fn edit_buffer_brackets_match_spec() {
        // Manual p.59 documents the current-buffer addresses as `0E 7F 00`
        // and `0F 7F 00`. Reading or writing the edit buffer must use these,
        // NOT `0E 00 00` / `0F 00 00` (which would target User Live Set 1-1).
        assert_eq!(BULK_HEADER_CURRENT_BUFFER, [0x0E, 0x7F, 0x00]);
        assert_eq!(BULK_FOOTER_CURRENT_BUFFER, [0x0F, 0x7F, 0x00]);
    }

    #[test]
    fn slot_brackets_match_spec() {
        // Page 1, Sound 1 → `0E 00 00` / `0F 00 00` (the first User slot).
        assert_eq!(bulk_header_for_slot(1, 1), Some([0x0E, 0x00, 0x00]));
        assert_eq!(bulk_footer_for_slot(1, 1), Some([0x0F, 0x00, 0x00]));
        // Page 20, Sound 8 → `0E 13 07` / `0F 13 07` (the last User slot).
        assert_eq!(bulk_header_for_slot(20, 8), Some([0x0E, 0x13, 0x07]));
        assert_eq!(bulk_footer_for_slot(20, 8), Some([0x0F, 0x13, 0x07]));
        // Out of range — 0 and over-max both rejected.
        assert_eq!(bulk_header_for_slot(0, 1), None);
        assert_eq!(bulk_header_for_slot(1, 0), None);
        assert_eq!(bulk_header_for_slot(21, 1), None);
        assert_eq!(bulk_header_for_slot(1, 9), None);
        assert_eq!(bulk_footer_for_slot(21, 1), None);
    }
}
