//! Shared codec error type, the `byte_enum!` macro, and value-format helpers
//! used by every typed area module.
//!
//! The CK stores parameters in a handful of recurring shapes, all documented in
//! the *CK88/CK61 Owner's Manual* "MIDI Data Table" (Group Number 7F 1C,
//! Model ID 0B):
//!
//! - **single byte** — a plain `0..=n` value or `0/1` boolean.
//! - **centered signed** — e.g. Transpose `−12..=+12` stored as `byte − 0x40`
//!   (so `0x40` is the centre). The default column shows `40` for these.
//! - **14-bit split** (`Size 2`) — `1st byte = bits13..7`, `2nd byte = bits6..0`.
//!   Used for voice numbers and Tempo.
//! - **16-bit nibble-packed** (`Size 4`) — Master Tune: four bytes carry one
//!   nibble each, MSN first.
//! - **ASCII** — names / file paths, one character per byte.

use thiserror::Error;

/// Errors from decoding/encoding the typed area models over their byte buffers.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CodecError {
    #[error("expected at least {expected} bytes, got {actual}")]
    WrongLength { expected: usize, actual: usize },

    #[error("invalid {field} value {value:#04x} (valid: {valid})")]
    InvalidValue {
        field: &'static str,
        value: u8,
        valid: &'static str,
    },

    #[error("{field} out of range: {value} (valid: {valid})")]
    OutOfRange {
        field: &'static str,
        value: i32,
        valid: &'static str,
    },

    #[error("{field}: invalid ASCII string ({reason})")]
    BadString {
        field: &'static str,
        reason: &'static str,
    },

    #[error("YAML: {0}")]
    Yaml(String),

    #[error("missing required Live Set block: {0}")]
    MissingBlock(&'static str),
}

/// Define a C-like enum that maps to/from a single wire byte, deriving the same
/// `serde` / `tsify` / `schemars` attributes every CK type uses.
///
/// Mirrors the macro the sibling `minilogue-core` crate uses.
macro_rules! byte_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident = $value:expr),+ $(,)? } valid = $valid:expr) => {
        $(#[$meta])*
        #[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
        #[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            const FIELD: &'static str = stringify!($name);

            /// Decode the wire byte.
            pub fn from_byte(b: u8) -> Result<Self, $crate::codec::CodecError> {
                match b {
                    $($value => Ok(Self::$variant),)+
                    _ => Err($crate::codec::CodecError::InvalidValue {
                        field: Self::FIELD, value: b, valid: $valid,
                    }),
                }
            }

            /// Encode to the wire byte.
            pub fn to_byte(self) -> u8 {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }
    };
}
pub(crate) use byte_enum;

/// `0 => false`, `1 => true`, anything else is an error.
pub fn bool_byte(b: u8, field: &'static str) -> Result<bool, CodecError> {
    match b {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(CodecError::InvalidValue {
            field,
            value: b,
            valid: "0=off, 1=on",
        }),
    }
}

/// Validate that `b` lies in `lo..=hi` and return it unchanged.
pub fn ranged(b: u8, lo: u8, hi: u8, field: &'static str) -> Result<u8, CodecError> {
    if (lo..=hi).contains(&b) {
        Ok(b)
    } else {
        Err(CodecError::OutOfRange {
            field,
            value: b as i32,
            valid: leak_range(lo as i32, hi as i32),
        })
    }
}

/// Decode a centred-signed byte: actual value = `b − center`, validated to
/// `lo..=hi`. e.g. `center = 0x40` for `−12..=+12` semitone fields.
pub fn signed_center(
    b: u8,
    center: u8,
    lo: i32,
    hi: i32,
    field: &'static str,
) -> Result<i8, CodecError> {
    let v = b as i32 - center as i32;
    if (lo..=hi).contains(&v) {
        Ok(v as i8)
    } else {
        Err(CodecError::OutOfRange {
            field,
            value: v,
            valid: leak_range(lo, hi),
        })
    }
}

/// Encode a centred-signed value back to its wire byte (`v + center`), validated.
pub fn to_signed_center(
    v: i8,
    center: u8,
    lo: i32,
    hi: i32,
    field: &'static str,
) -> Result<u8, CodecError> {
    let vi = v as i32;
    if (lo..=hi).contains(&vi) {
        Ok((vi + center as i32) as u8)
    } else {
        Err(CodecError::OutOfRange {
            field,
            value: vi,
            valid: leak_range(lo, hi),
        })
    }
}

/// Decode a `Size 2` 14-bit value: `hi` carries bits 13..7, `lo` carries bits 6..0.
pub fn read_u14(hi: u8, lo: u8) -> u16 {
    ((hi as u16 & 0x7F) << 7) | (lo as u16 & 0x7F)
}

/// Encode a 14-bit value into its `(hi, lo)` byte pair.
pub fn write_u14(v: u16) -> (u8, u8) {
    (((v >> 7) & 0x7F) as u8, (v & 0x7F) as u8)
}

/// Decode a `Size 4` nibble-packed 16-bit value (Master Tune): four bytes carry
/// one nibble each, most-significant nibble first.
pub fn read_u16_nibbles(b: &[u8]) -> u16 {
    ((b[0] as u16 & 0x0F) << 12)
        | ((b[1] as u16 & 0x0F) << 8)
        | ((b[2] as u16 & 0x0F) << 4)
        | (b[3] as u16 & 0x0F)
}

/// Encode a 16-bit value into four nibble-carrying bytes, MSN first.
pub fn write_u16_nibbles(v: u16) -> [u8; 4] {
    [
        ((v >> 12) & 0x0F) as u8,
        ((v >> 8) & 0x0F) as u8,
        ((v >> 4) & 0x0F) as u8,
        (v & 0x0F) as u8,
    ]
}

/// Decode an ASCII name field, trimming trailing spaces and NULs. The CK pads
/// names with `0x20` (space).
pub fn read_ascii(bytes: &[u8], field: &'static str) -> Result<String, CodecError> {
    if bytes.iter().any(|&b| b >= 0x80) {
        return Err(CodecError::BadString {
            field,
            reason: "byte >= 0x80",
        });
    }
    let s: String = bytes.iter().map(|&b| b as char).collect();
    Ok(s.trim_end_matches([' ', '\0']).to_string())
}

/// Encode a name into a fixed-width ASCII field, space-padded and truncated.
pub fn write_ascii(s: &str, width: usize, field: &'static str) -> Result<Vec<u8>, CodecError> {
    if !s.is_ascii() {
        return Err(CodecError::BadString {
            field,
            reason: "non-ASCII character",
        });
    }
    let mut out = vec![0x20u8; width];
    for (slot, b) in out.iter_mut().zip(s.bytes()) {
        *slot = b;
    }
    Ok(out)
}

/// A single raw byte the typed model doesn't interpret — a documented-reserved
/// offset that the device nonetheless populates, or a trailing byte added by
/// newer firmware. Captured so every area round-trips byte-exact even where the
/// manual is silent.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RawByte {
    pub offset: u16,
    pub value: u8,
}

/// Collect the offsets where the device payload differs from what the typed
/// fields alone produce — i.e. reserved/undocumented bytes and any trailing
/// firmware additions. The result, applied via [`apply_reserved`], makes
/// `from_bytes`→`to_bytes` byte-exact.
pub fn capture_reserved(input: &[u8], typed_only: &[u8]) -> Vec<RawByte> {
    let n = input.len().max(typed_only.len());
    (0..n)
        .filter_map(|i| {
            let inp = input.get(i).copied().unwrap_or(0);
            let typ = typed_only.get(i).copied().unwrap_or(0);
            (inp != typ).then_some(RawByte {
                offset: i as u16,
                value: inp,
            })
        })
        .collect()
}

/// Overlay captured reserved bytes onto an encode buffer, growing it if a
/// trailing offset lies past the current end.
pub fn apply_reserved(buf: &mut Vec<u8>, reserved: &[RawByte]) {
    for r in reserved {
        let i = r.offset as usize;
        if i >= buf.len() {
            buf.resize(i + 1, 0);
        }
        buf[i] = r.value;
    }
}

/// Format a `lo..=hi` range into a `'static` string for error messages.
///
/// Error values are rare and human-facing; leaking a tiny string keeps the
/// `CodecError` variants `'static` without threading lifetimes everywhere.
fn leak_range(lo: i32, hi: i32) -> &'static str {
    Box::leak(format!("{lo}..={hi}").into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u14_round_trips() {
        for v in [0u16, 1, 127, 128, 240, 16383] {
            let (hi, lo) = write_u14(v);
            assert_eq!(read_u14(hi, lo), v);
        }
    }

    #[test]
    fn nibbles_round_trip() {
        for v in [0u16, 0x0400, 0x1234, 0xFFFF] {
            assert_eq!(read_u16_nibbles(&write_u16_nibbles(v)), v);
        }
    }

    #[test]
    fn centered_signed_round_trips() {
        // Transpose: −12..=+12, centre 0x40.
        assert_eq!(signed_center(0x40, 0x40, -12, 12, "t").unwrap(), 0);
        assert_eq!(signed_center(0x34, 0x40, -12, 12, "t").unwrap(), -12);
        assert_eq!(signed_center(0x4C, 0x40, -12, 12, "t").unwrap(), 12);
        assert_eq!(to_signed_center(0, 0x40, -12, 12, "t").unwrap(), 0x40);
        assert_eq!(to_signed_center(-12, 0x40, -12, 12, "t").unwrap(), 0x34);
        assert!(signed_center(0x60, 0x40, -12, 12, "t").is_err());
    }

    #[test]
    fn ascii_trims_padding() {
        assert_eq!(read_ascii(b"Init      ", "n").unwrap(), "Init");
        assert_eq!(write_ascii("Init", 10, "n").unwrap(), b"Init      ");
    }
}
