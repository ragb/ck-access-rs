//! Live Set selection over MIDI channel-voice (Bank Select + Program Change).
//!
//! The CK organises its 160 user Live Set Sounds as **20 pages × 8 sounds per
//! page**. Switching the active sound from a host is the standard MIDI dance
//! of Bank Select MSB / LSB followed by a Program Change. The mapping is
//! pinned in the *Owner's Manual* MIDI Implementation Chart:
//!
//! ```text
//!                   CATEGORY            MSB  LSB   PROGRAM No.
//! Live Set Page 1                       63   0     0..=7
//!     :
//! Live Set Page 20                      63   19    0..=7
//! ```
//!
//! Both the System "Tx/Rx Bank" and "Tx/Rx Pgm" switches must be on for the
//! CK to act on these (defaults are on). The channel byte uses the device's
//! receive channel.

/// Bank MSB value the CK uses to select a Live Set Sound bank.
pub const LIVE_SET_BANK_MSB: u8 = 63;

/// Number of Live Set pages (Bank LSB range is `0..=PAGES-1`).
pub const PAGES: u8 = 20;

/// Number of Live Set Sounds per page (Program Change range is `0..=SOUNDS_PER_PAGE-1`).
pub const SOUNDS_PER_PAGE: u8 = 8;

/// Total user Live Set Sounds on the CK.
pub const TOTAL_LIVE_SETS: u16 = PAGES as u16 * SOUNDS_PER_PAGE as u16;

/// Errors from [`select_live_set_messages`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SelectError {
    #[error("channel {0} out of MIDI range (1..=16)")]
    Channel(u8),
    #[error("page {0} out of range (1..={PAGES})")]
    Page(u8),
    #[error("sound {0} out of range (1..={SOUNDS_PER_PAGE})")]
    Sound(u8),
}

/// Build the three short MIDI frames that switch the CK to Live Set Sound
/// `sound` on `page`, on MIDI channel `channel`.
///
/// `channel` is 1-based (1..=16); `page` is 1-based (1..=20); `sound` is
/// 1-based (1..=8). The returned frames must be sent **in order** — the CK,
/// like all Yamaha gear, only acts on the latched Bank Select once the
/// Program Change arrives.
pub fn select_live_set_messages(
    channel: u8,
    page: u8,
    sound: u8,
) -> Result<[Vec<u8>; 3], SelectError> {
    if !(1..=16).contains(&channel) {
        return Err(SelectError::Channel(channel));
    }
    if !(1..=PAGES).contains(&page) {
        return Err(SelectError::Page(page));
    }
    if !(1..=SOUNDS_PER_PAGE).contains(&sound) {
        return Err(SelectError::Sound(sound));
    }
    let ch = channel - 1;
    Ok([
        // Bank Select MSB: CC 0 = 63
        vec![0xB0 | ch, 0x00, LIVE_SET_BANK_MSB],
        // Bank Select LSB: CC 32 = page (0..=19)
        vec![0xB0 | ch, 0x20, page - 1],
        // Program Change: sound (0..=7)
        vec![0xC0 | ch, sound - 1],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_1_sound_1_on_ch1() {
        let m = select_live_set_messages(1, 1, 1).unwrap();
        assert_eq!(m[0], [0xB0, 0x00, 63]);
        assert_eq!(m[1], [0xB0, 0x20, 0]);
        assert_eq!(m[2], [0xC0, 0]);
    }

    #[test]
    fn page_20_sound_8_on_ch16() {
        let m = select_live_set_messages(16, 20, 8).unwrap();
        assert_eq!(m[0], [0xBF, 0x00, 63]);
        assert_eq!(m[1], [0xBF, 0x20, 19]);
        assert_eq!(m[2], [0xCF, 7]);
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(matches!(
            select_live_set_messages(0, 1, 1),
            Err(SelectError::Channel(_))
        ));
        assert!(matches!(
            select_live_set_messages(1, 0, 1),
            Err(SelectError::Page(_))
        ));
        assert!(matches!(
            select_live_set_messages(1, 21, 1),
            Err(SelectError::Page(_))
        ));
        assert!(matches!(
            select_live_set_messages(1, 1, 9),
            Err(SelectError::Sound(_))
        ));
    }

    #[test]
    fn total_matches_grid() {
        assert_eq!(TOTAL_LIVE_SETS, 160);
    }
}
