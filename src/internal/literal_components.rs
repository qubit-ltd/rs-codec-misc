// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Parsed components of a C integer literal.

use crate::MiscCodecError;
use crate::MiscCodecResult;

/// Radix and digit slice extracted from a C integer literal.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LiteralComponents<'a> {
    /// Radix selected by the literal prefix.
    pub(crate) radix: u32,
    /// Digit slice without the radix prefix.
    pub(crate) digits: &'a str,
    /// Original byte offset of the first digit.
    pub(crate) digits_offset: usize,
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
    /// Returns [`MiscCodecError::InvalidInput`] when a radix prefix is present
    /// without any digits after it.
    #[inline]
    pub(crate) fn parse(
        trimmed: &'a str,
        trim_offset: usize,
    ) -> MiscCodecResult<Self> {
        if let Some(digits) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            if digits.is_empty() {
                return Err(MiscCodecError::InvalidInput {
                    codec: "c-integer-literal",
                    reason: "hexadecimal literal requires at least one digit"
                        .to_owned(),
                });
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
