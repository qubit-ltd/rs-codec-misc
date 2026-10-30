// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Percent codec type and trait implementations.

use core::num::NonZeroUsize;

use qubit_codec::Codec;
use qubit_codec::DecodeFailure;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_utils::nonzero;

use super::internal::percent_decode_byte_eof;
use super::percent_decode_byte;
use super::percent_decode_bytes;
use super::percent_encode_byte;
use super::percent_encode_bytes;
use super::percent_encode_len;
use super::percent_invalid_consumed;
use crate::MiscCodecError;
use crate::MiscCodecResult;
use crate::internal::ParseError;
use crate::internal::PercentEncodingMode;
use crate::misc_error::map_misc_decode_failure_with_consumed;

/// Encodes and decodes percent-encoded UTF-8 text.
///
/// Its low-level [`Codec<Value = u8, Unit = u8>`] implementation converts one
/// byte to either one unreserved ASCII unit or a `%XX` escape. UTF-8 validation
/// remains part of the owned [`decode`](Self::decode) helper.
/// Streaming skip/replace hooks may consume the malformed span visible in the
/// current input chunk, so recovery output for malformed input can depend on
/// chunk boundaries. This does not affect valid input or strict decoding.
///
/// # Examples
///
/// ```
/// use qubit_codec_misc::PercentCodec;
///
/// let codec = PercentCodec::new();
/// let encoded = codec.encode("hello world");
/// assert_eq!(encoded, "hello%20world");
/// assert_eq!(codec.decode(&encoded).expect("valid encoded text"), "hello world");
/// ```
#[must_use]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PercentCodec;

impl PercentCodec {
    /// Creates a percent codec.
    ///
    /// # Returns
    /// Percent codec.
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Encodes text using percent encoding.
    ///
    /// # Parameters
    /// - `text`: UTF-8 text to encode.
    ///
    /// # Returns
    /// Percent-encoded text.
    #[inline]
    #[must_use]
    pub fn encode(&self, text: &str) -> String {
        percent_encode_bytes(text.as_bytes(), PercentEncodingMode::Rfc3986)
    }

    /// Decodes percent-encoded UTF-8 text.
    ///
    /// # Parameters
    /// - `text`: Percent-encoded text.
    ///
    /// # Returns
    /// Decoded UTF-8 text.
    ///
    /// # Errors
    /// Returns [`MiscCodecError`] when a percent escape is malformed or decoded
    /// bytes are not valid UTF-8.
    #[inline]
    pub fn decode(&self, text: &str) -> MiscCodecResult<String> {
        String::from_utf8(percent_decode_bytes(text, false)?).map_err(MiscCodecError::from)
    }
}

impl ValueEncoder<str> for PercentCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Encodes one text input using RFC 3986 percent encoding.
    ///
    /// # Parameters
    /// - `input`: UTF-8 text to encode.
    ///
    /// # Returns
    /// The percent-encoded text.
    ///
    /// # Errors
    /// This implementation does not reject text and therefore always succeeds.
    #[inline]
    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(PercentCodec::encode(self, input))
    }
}

impl ValueDecoder<str> for PercentCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Decodes one percent-encoded UTF-8 text input.
    ///
    /// # Parameters
    /// - `input`: Percent-encoded text to decode.
    ///
    /// # Returns
    /// The decoded UTF-8 text.
    ///
    /// # Errors
    /// Returns an error for malformed escapes or decoded bytes that are not
    /// valid UTF-8.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        PercentCodec::decode(self, input)
    }
}

impl Codec for PercentCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = MiscCodecError;
    type EncodeError = MiscCodecError;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 3;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 3;

    /// Returns the exact RFC 3986 encoded width for one byte.
    ///
    /// # Parameters
    /// - `value`: Byte whose encoded width is measured.
    ///
    /// # Returns
    /// One for an unreserved byte, or three for an escaped byte.
    #[inline]
    fn encode_len(&self, value: &u8) -> usize {
        percent_encode_len(*value, PercentEncodingMode::Rfc3986)
    }

    /// Decodes one raw byte or `%XX` escape.
    ///
    /// # Parameters
    /// - `input`: Encoded byte units.
    /// - `input_index`: Index of the byte or escape to decode.
    ///
    /// # Returns
    /// The decoded byte and non-zero number of consumed units.
    ///
    /// # Errors
    /// Returns a decode failure when the visible escape is malformed or
    /// incomplete.
    ///
    /// # Safety
    /// The caller must provide an input index with at least one readable unit
    /// and a slice satisfying the codec trait's decode preconditions.
    #[inline]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());

        let (value, consumed) = percent_decode_byte(input, input_index, false).map_err(|error| {
            ParseError::into_decode_failure_with_consumed(error, percent_invalid_consumed(input, input_index))
        })?;
        debug_assert!(consumed > 0);
        // SAFETY: `percent_decode_byte` returns a non-zero width for every
        // successful raw byte or escape.
        let consumed = nonzero(consumed);
        Ok((value, consumed))
    }

    /// Encodes one byte using percent encoding.
    ///
    /// # Parameters
    /// - `value`: Byte to encode.
    /// - `output`: Destination unit buffer.
    /// - `output_index`: Start index in `output`.
    ///
    /// # Returns
    /// Number of units written.
    ///
    /// # Errors
    /// This implementation accepts every byte and therefore always succeeds.
    ///
    /// # Safety
    /// The caller must provide enough writable output units for
    /// [`Self::encode_len`].
    #[inline]
    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        let required = percent_encode_len(*value, PercentEncodingMode::Rfc3986);
        debug_assert!(output_index + required <= output.len());

        let written = percent_encode_byte(*value, output, output_index, PercentEncodingMode::Rfc3986);
        debug_assert_eq!(written, required);
        Ok(required)
    }

    /// Decodes one value after end of input has been confirmed.
    ///
    /// # Parameters
    /// - `input`: Complete encoded byte units.
    /// - `input_index`: Index of the byte or escape to decode.
    ///
    /// # Returns
    /// The decoded byte and non-zero number of consumed units.
    ///
    /// # Errors
    /// Returns a decode failure when the escape is malformed or incomplete.
    ///
    /// # Safety
    /// The caller must provide an input index with at least one readable unit
    /// and a slice satisfying the codec trait's EOF decode preconditions.
    #[inline]
    unsafe fn decode_eof(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());

        let (value, consumed) = percent_decode_byte_eof(input, input_index, false).map_err(|error| {
            map_misc_decode_failure_with_consumed(error, percent_invalid_consumed(input, input_index))
        })?;
        debug_assert!(consumed > 0);
        let consumed = nonzero(consumed);
        Ok((value, consumed))
    }
}
