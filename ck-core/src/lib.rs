#![forbid(unsafe_code)]

//! Yamaha **CK61 / CK88** SysEx codec.
//!
//! Pure: no MIDI, no file I/O — compiles for `wasm32-unknown-unknown`.
//!
//! The CK speaks Yamaha's address-based parameter protocol (Group `7F 1C`,
//! Model ID `0B`): a 3-byte address selects a parameter or block, and four
//! message kinds ([`sysex::Message`]) read/write it. [`address`] maps the
//! address space onto the documented blocks; the typed area models
//! ([`system`], [`live_set`], [`zone`], [`part`]) decode each block to named
//! fields and re-encode byte-exact. [`document`] bundles them into the
//! editable [`System`] and [`LiveSet`] aggregates.
//!
//! Layout, ranges, and defaults come from the *CK88/CK61 Owner's Manual*
//! "MIDI Data Format" / "MIDI Data Table". See `docs/sysex-notes.md` for the
//! protocol notes and the (still-pending) hardware verification status.

pub mod address;
pub mod codec;
pub mod document;
pub mod inbound;
pub mod live_set;
pub mod live_set_select;
pub mod part;
pub mod rotary;
#[cfg(feature = "schema")]
pub mod schema;
pub mod sysex;
pub mod system;
pub mod voices;
pub mod yaml;
pub mod zone;

pub use address::{AddressSpace, Part as PartSlot};
pub use codec::CodecError;
pub use document::{LiveSet, RawBlock, System};
pub use inbound::{classify_inbound, InboundMessage};
pub use live_set::{LiveSetCommon, LiveSetEq};
pub use live_set_select::{
    select_live_set_messages, SelectError, LIVE_SET_BANK_MSB, PAGES, SOUNDS_PER_PAGE,
    TOTAL_LIVE_SETS,
};
pub use part::Part;
pub use rotary::{RotaryParamSpec, RotarySpeaker, StereoMono, ROTARY_SPECS};
pub use sysex::{
    checksum, identify_reply, identity_request, CkModel, Message, SysExError, MODEL_ID, YAMAHA_ID,
};
pub use system::{MasterEq, SystemCommon};
pub use voices::{category_of, voice_name, Category};
pub use zone::{Zone, ZoneTransmit, ZoneTransmitControllers};
