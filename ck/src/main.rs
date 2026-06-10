//! CK CLI entry point.
//!
//! The whole command surface — dump / sync / show / lint / diff / schema /
//! catalog / resolve / identity / ports — is the generic engine in
//! `midi-access-cli`, dispatched through [`ck_core::Ck`]'s [`Device`] impl.
//!
//! [`Device`]: midi_access_core::Device

use std::process::ExitCode;

fn main() -> ExitCode {
    midi_access_cli::run::<ck_core::Ck>()
}
