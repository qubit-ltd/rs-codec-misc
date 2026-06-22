// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared codec error type.

use std::string::FromUtf8Error;

use qubit_codec::CodecDecodeFailure;
use thiserror::Error;

/// Result alias returned by codec operations.
pub type MiscCodecResult<T> = Result<T, MiscCodecError>;

/// Error returned by codec operations.
#[derive(Debug, Error)]
pub enum MiscCodecError {
    /// Input ended before a complete codec value was available.
    #[error("incomplete input: required {required} units, available {available}")]
    Incomplete {
        /// Total units required from the current decode start.
        required: usize,
        /// Units currently available from the current decode start.
        available: usize,
    },

    /// A configured prefix was required but missing.
    #[error("missing required prefix '{prefix}'")]
    MissingPrefix {
        /// Required prefix.
        prefix: String,
    },

    /// Input contained a digit that is invalid for the requested radix.
    #[error("invalid radix-{radix} digit '{character}' at index {index}")]
    InvalidDigit {
        /// Numeric radix expected by the codec.
        radix: u32,
        /// Character byte index in the original input.
        index: usize,
        /// Invalid character.
        character: char,
    },

    /// Input length does not satisfy a codec requirement.
    #[error("invalid length for {context}: expected {expected}, got {actual}")]
    InvalidLength {
        /// Human-readable input part whose length was invalid.
        context: &'static str,
        /// Human-readable length requirement.
        expected: String,
        /// Actual length observed by the codec.
        actual: usize,
    },

    /// Input contained a malformed or unsupported escape sequence.
    #[error("invalid escape {escape:?} at index {index}: {reason}")]
    InvalidEscape {
        /// Byte index of the escape marker in the original input.
        index: usize,
        /// Escape sequence fragment that caused the error.
        escape: String,
        /// Human-readable reason the escape was rejected.
        reason: String,
    },

    /// Input contained a character that cannot appear in that context.
    #[error("invalid character '{character}' at index {index}: {reason}")]
    InvalidCharacter {
        /// Character byte index in the original input.
        index: usize,
        /// Invalid character.
        character: char,
        /// Human-readable reason the character was rejected.
        reason: String,
    },

    /// Input was rejected by a codec-specific validator.
    #[error("invalid {codec} input: {reason}")]
    InvalidInput {
        /// Stable codec name, such as `base64`.
        codec: &'static str,
        /// Human-readable reason reported by the codec.
        reason: String,
    },

    /// Decoded bytes were not valid UTF-8.
    #[error("decoded bytes are not valid UTF-8: {source}")]
    InvalidUtf8 {
        /// Underlying UTF-8 conversion error.
        #[from]
        source: FromUtf8Error,
    },
}

#[inline]
pub(crate) fn map_misc_decode_failure(error: MiscCodecError) -> CodecDecodeFailure<MiscCodecError> {
    match error {
        MiscCodecError::Incomplete { required, .. } => CodecDecodeFailure::incomplete(required),
        error => CodecDecodeFailure::invalid_without_consumed(error),
    }
}
