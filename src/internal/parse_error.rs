// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private parser outcome used by streaming codecs.

use core::num::NonZeroUsize;

use qubit_codec::DecodeFailure;

use crate::MiscCodecError;

/// Distinguishes retryable stream tails from malformed codec input.
#[must_use]
#[derive(Debug)]
pub(crate) enum ParseError {
    /// More input is required before a value can be decided.
    Incomplete {
        /// Total units required from the current value start.
        required: NonZeroUsize,
    },
    /// The input is malformed.
    Invalid(MiscCodecError),
}

impl From<MiscCodecError> for ParseError {
    /// Wraps a codec error as malformed parser input.
    ///
    /// # Parameters
    ///
    /// - `error`: Codec-specific error to retain as invalid input.
    ///
    /// # Returns
    ///
    /// Returns the parser outcome carrying `error` as malformed input.
    #[inline]
    fn from(error: MiscCodecError) -> Self {
        Self::Invalid(error)
    }
}

impl ParseError {
    /// Converts a parser outcome while retaining a known invalid width.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Non-zero width to report if the input is invalid.
    ///
    /// # Returns
    ///
    /// Returns an incomplete failure with its required width, or an invalid
    /// failure carrying `consumed` and the original codec error.
    #[inline]
    pub(crate) fn into_decode_failure_with_consumed(self, consumed: NonZeroUsize) -> DecodeFailure<MiscCodecError> {
        match self {
            Self::Incomplete { required, .. } => DecodeFailure::incomplete(required),
            Self::Invalid(error) => DecodeFailure::invalid(error, consumed),
        }
    }
}
