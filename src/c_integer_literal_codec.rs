// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! C integer literal decoder.

use crate::{
    MiscCodecError,
    MiscCodecResult,
    internal::LiteralComponents,
};
use qubit_codec::ValueDecoder;

/// Decodes restricted non-negative C integer literal fragments.
///
/// This codec accepts decimal literals such as `123`, octal literals such as
/// `0123`, and hexadecimal literals such as `0x123` or `0X123`. It trims
/// surrounding whitespace and returns a `u64`. It intentionally accepts only
/// a complete unsigned token: signs, integer suffixes, digit separators, and
/// token-stream boundaries are outside this codec's contract.
#[must_use]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CIntegerLiteralCodec;

impl CIntegerLiteralCodec {
    /// Creates a C integer literal codec.
    ///
    /// # Returns
    /// A stateless C integer literal codec.
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Decodes a C integer literal into a `u64`.
    ///
    /// # Parameters
    /// - `text`: C integer literal text.
    ///
    /// # Returns
    /// Parsed integer value.
    ///
    /// # Errors
    /// Returns [`MiscCodecError::InvalidInput`] when the input is empty, lacks
    /// digits, or overflows `u64`; returns [`MiscCodecError::InvalidDigit`]
    /// when a character is not valid for the detected radix.
    #[inline]
    pub fn decode(&self, text: &str) -> MiscCodecResult<u64> {
        let (trimmed, trim_offset) = trim_with_offset(text);
        if trimmed.is_empty() {
            return Err(invalid_c_integer_input("expected at least one digit"));
        }
        let components = LiteralComponents::parse(trimmed, trim_offset)?;
        validate_digits(components)?;
        u64::from_str_radix(components.digits, components.radix).map_err(
            |error| {
                invalid_c_integer_input(&format!(
                    "integer literal is out of range: {error}"
                ))
            },
        )
    }
}

impl ValueDecoder<str> for CIntegerLiteralCodec {
    type Error = MiscCodecError;
    type Output = u64;

    /// Decodes a C integer literal into a `u64`.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        CIntegerLiteralCodec::decode(self, input)
    }
}

/// Trims surrounding whitespace while preserving the start byte offset.
///
/// # Parameters
/// - `text`: Input text.
///
/// # Returns
/// Trimmed text and the byte offset where it starts in `text`.
#[inline]
fn trim_with_offset(text: &str) -> (&str, usize) {
    let trimmed_start = text.trim_start();
    let start = text.len() - trimmed_start.len();
    (trimmed_start.trim_end(), start)
}

/// Validates that every character is valid for the detected radix.
///
/// # Parameters
/// - `components`: Parsed literal components.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidDigit`] with the original input byte index
/// of the invalid character.
fn validate_digits(components: LiteralComponents<'_>) -> MiscCodecResult<()> {
    for (index, character) in components.digits.char_indices() {
        if character.is_digit(components.radix) {
            continue;
        }
        return Err(MiscCodecError::InvalidDigit {
            radix: components.radix,
            index: components.digits_offset + index,
            character,
        });
    }
    Ok(())
}

/// Builds an invalid C integer literal input error.
///
/// # Parameters
/// - `reason`: Human-readable reason the input was rejected.
///
/// # Returns
/// An invalid input error for the C integer literal codec.
fn invalid_c_integer_input(reason: &str) -> MiscCodecError {
    MiscCodecError::InvalidInput {
        codec: "c-integer-literal",
        reason: reason.to_owned(),
    }
}
