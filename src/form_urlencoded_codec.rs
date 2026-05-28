/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! `application/x-www-form-urlencoded` text codec.

use crate::percent_codec::{
    percent_decode_byte,
    percent_decode_bytes,
    percent_encode_byte,
    percent_encode_bytes,
};
use crate::{
    Codec,
    MiscCodecError,
    MiscCodecResult,
    ValueDecoder,
    ValueEncoder,
};

/// Encodes and decodes `application/x-www-form-urlencoded` text fragments.
///
/// Its low-level [`Codec<u8, u8>`] implementation converts one byte at a time,
/// including the form-specific space and `+` mapping. UTF-8 validation remains
/// part of the owned [`decode`](Self::decode) helper.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FormUrlencodedCodec;

impl FormUrlencodedCodec {
    /// Creates a form-url-encoded codec.
    ///
    /// # Returns
    /// Form URL encoded codec.
    pub fn new() -> Self {
        Self
    }

    /// Encodes text, using `+` for spaces.
    ///
    /// # Parameters
    /// - `text`: Text to encode.
    ///
    /// # Returns
    /// Form-url-encoded text.
    pub fn encode(&self, text: &str) -> String {
        percent_encode_bytes(text.as_bytes(), true)
    }

    /// Decodes text, treating `+` as space.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    ///
    /// # Returns
    /// Decoded UTF-8 text.
    ///
    /// # Errors
    /// Returns [`MiscCodecError`] when an escape is malformed or decoded bytes are
    /// not valid UTF-8.
    pub fn decode(&self, text: &str) -> MiscCodecResult<String> {
        String::from_utf8(percent_decode_bytes(text, true)?).map_err(MiscCodecError::from)
    }
}

impl ValueEncoder<str> for FormUrlencodedCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Encodes text, using `+` for spaces.
    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(FormUrlencodedCodec::encode(self, input))
    }
}

impl ValueDecoder<str> for FormUrlencodedCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Decodes form-url-encoded text.
    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        FormUrlencodedCodec::decode(self, input)
    }
}

unsafe impl Codec<u8, u8> for FormUrlencodedCodec {
    type DecodeError = MiscCodecError;
    type EncodeError = MiscCodecError;

    /// Returns the shortest representation length for one byte.
    fn min_units_per_value(&self) -> usize {
        1
    }

    /// Returns the longest `%XX` representation length for one byte.
    fn max_units_per_value(&self) -> usize {
        3
    }

    /// Decodes one raw byte, `+`, or `%XX` escape.
    unsafe fn decode_unchecked(&self, input: &[u8], index: usize) -> Result<(u8, usize), Self::DecodeError> {
        debug_assert!(index < input.len());

        percent_decode_byte(input, index, true)
    }

    /// Encodes one byte using form URL encoding.
    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(
            index
                + if *value == b' ' || value.is_ascii_alphanumeric() || matches!(*value, b'-' | b'.' | b'_' | b'~') {
                    1
                } else {
                    3
                }
                <= output.len()
        );

        Ok(percent_encode_byte(*value, output, index, true))
    }
}
