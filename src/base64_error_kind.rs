// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Structured Base64 dependency error categories.

use ::base64::DecodeError;

/// Structured category for an error reported by the Base64 dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base64ErrorKind {
    /// A byte outside the configured alphabet was encountered.
    InvalidByte,
    /// The number of Base64 symbols is invalid.
    InvalidLength,
    /// The final symbol contains bits that would be discarded.
    InvalidLastSymbol,
    /// Padding is absent, unexpected, or malformed for the configured engine.
    InvalidPadding,
}

impl Base64ErrorKind {
    /// Classifies a Base64 dependency error without discarding its source.
    #[inline]
    pub(crate) const fn from_decode_error(error: &DecodeError) -> Self {
        match error {
            DecodeError::InvalidByte(_, _) => Self::InvalidByte,
            DecodeError::InvalidLength(_) => Self::InvalidLength,
            DecodeError::InvalidLastSymbol { .. } => Self::InvalidLastSymbol,
            DecodeError::InvalidPadding => Self::InvalidPadding,
        }
    }

    /// Returns the input offset carried by this Base64 error, when available.
    #[inline]
    pub(crate) const fn input_index(error: &DecodeError) -> Option<usize> {
        match error {
            DecodeError::InvalidByte(index, _)
            | DecodeError::InvalidLastSymbol { offset: index, .. } => Some(*index),
            DecodeError::InvalidLength(_) | DecodeError::InvalidPadding => None,
        }
    }

    /// Returns the invalid symbol carried by this Base64 error, when present.
    #[inline]
    pub(crate) const fn symbol(error: &DecodeError) -> Option<u8> {
        match error {
            DecodeError::InvalidByte(_, symbol) | DecodeError::InvalidLastSymbol { symbol, .. } => {
                Some(*symbol)
            }
            DecodeError::InvalidLength(_) | DecodeError::InvalidPadding => None,
        }
    }

    /// Returns the invalid input length carried by this Base64 error, when
    /// present.
    #[inline]
    pub(crate) const fn input_length(error: &DecodeError) -> Option<usize> {
        match error {
            DecodeError::InvalidLength(length) => Some(*length),
            DecodeError::InvalidByte(_, _)
            | DecodeError::InvalidLastSymbol { .. }
            | DecodeError::InvalidPadding => None,
        }
    }
}
