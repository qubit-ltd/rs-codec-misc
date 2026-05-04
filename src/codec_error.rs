/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Shared codec error type.

use std::string::FromUtf8Error;

use thiserror::Error;

/// Result alias returned by codec operations.
pub type CodecResult<T> = Result<T, CodecError>;

/// Error returned by codec operations.
#[derive(Debug, Error)]
pub enum CodecError {
    /// A configured prefix was required but missing.
    #[error("missing required prefix '{prefix}'")]
    MissingPrefix {
        /// Required prefix.
        prefix: String,
    },

    /// Hex input contained an odd number of digits.
    #[error("hex input contains an odd number of digits: {digits}")]
    OddHexLength {
        /// Number of hex digits seen after normalization.
        digits: usize,
    },

    /// Hex input contained a non-hexadecimal digit.
    #[error("invalid hex digit '{character}' at index {index}")]
    InvalidHexDigit {
        /// Character index in the original input.
        index: usize,
        /// Invalid character.
        character: char,
    },

    /// Base64 input was malformed.
    #[error("invalid base64 input: {source}")]
    InvalidBase64 {
        /// Underlying Base64 decoder error.
        #[from]
        source: base64::DecodeError,
    },

    /// Percent input contained a malformed `%XX` escape.
    #[error("invalid percent escape at index {index}")]
    InvalidPercentEscape {
        /// Byte index of the `%` marker in the input.
        index: usize,
    },

    /// Decoded bytes were not valid UTF-8.
    #[error("decoded bytes are not valid UTF-8: {source}")]
    InvalidUtf8 {
        /// Underlying UTF-8 conversion error.
        #[from]
        source: FromUtf8Error,
    },
}
