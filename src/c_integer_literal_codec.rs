/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! C integer literal decoder.

use crate::{
    CodecError,
    CodecResult,
    Decoder,
};

/// Decodes non-negative C integer literal fragments.
///
/// This codec accepts decimal literals such as `123`, octal literals such as
/// `0123`, and hexadecimal literals such as `0x123` or `0X123`. It trims
/// surrounding whitespace and returns a `u64`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CIntegerLiteralCodec;

impl CIntegerLiteralCodec {
    /// Creates a C integer literal codec.
    ///
    /// # Returns
    /// A stateless C integer literal codec.
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
    /// Returns [`CodecError::InvalidInput`] when the input is empty, lacks digits,
    /// or overflows `u64`; returns [`CodecError::InvalidDigit`] when a character
    /// is not valid for the detected radix.
    pub fn decode(&self, text: &str) -> CodecResult<u64> {
        let (trimmed, trim_offset) = trim_with_offset(text);
        if trimmed.is_empty() {
            return Err(invalid_c_integer_input("expected at least one digit"));
        }
        let components = LiteralComponents::parse(trimmed, trim_offset)?;
        validate_digits(components)?;
        u64::from_str_radix(components.digits, components.radix)
            .map_err(|error| invalid_c_integer_input(&format!("integer literal is out of range: {error}")))
    }
}

impl Decoder<str> for CIntegerLiteralCodec {
    type Error = CodecError;
    type Output = u64;

    /// Decodes a C integer literal into a `u64`.
    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        CIntegerLiteralCodec::decode(self, input)
    }
}

/// Parsed C integer literal components.
#[derive(Debug, Clone, Copy)]
struct LiteralComponents<'a> {
    radix: u32,
    digits: &'a str,
    digits_offset: usize,
}

impl<'a> LiteralComponents<'a> {
    /// Parses radix and digit slice from trimmed input.
    ///
    /// # Parameters
    /// - `trimmed`: Input after surrounding whitespace has been removed.
    /// - `trim_offset`: Byte offset of `trimmed` in the original input.
    ///
    /// # Returns
    /// Literal components used by validation and numeric parsing.
    ///
    /// # Errors
    /// Returns [`CodecError::InvalidInput`] when a radix prefix is present without
    /// any digits after it.
    fn parse(trimmed: &'a str, trim_offset: usize) -> CodecResult<Self> {
        if let Some(digits) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
            if digits.is_empty() {
                return Err(invalid_c_integer_input(
                    "hexadecimal literal requires at least one digit",
                ));
            }
            return Ok(Self {
                radix: 16,
                digits,
                digits_offset: trim_offset + 2,
            });
        }
        if trimmed.len() > 1
            && let Some(digits) = trimmed.strip_prefix('0')
        {
            return Ok(Self {
                radix: 8,
                digits,
                digits_offset: trim_offset + 1,
            });
        }
        Ok(Self {
            radix: 10,
            digits: trimmed,
            digits_offset: trim_offset,
        })
    }
}

/// Trims surrounding whitespace while preserving the start byte offset.
///
/// # Parameters
/// - `text`: Input text.
///
/// # Returns
/// Trimmed text and the byte offset where it starts in `text`.
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
/// Returns [`CodecError::InvalidDigit`] with the original input byte index of
/// the invalid character.
fn validate_digits(components: LiteralComponents<'_>) -> CodecResult<()> {
    for (index, character) in components.digits.char_indices() {
        if character.is_digit(components.radix) {
            continue;
        }
        return Err(CodecError::InvalidDigit {
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
fn invalid_c_integer_input(reason: &str) -> CodecError {
    CodecError::InvalidInput {
        codec: "c-integer-literal",
        reason: reason.to_owned(),
    }
}
