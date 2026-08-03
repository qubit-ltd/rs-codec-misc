// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! `application/x-www-form-urlencoded` text codec.

use crate::percent_codec::{
    percent_decode_byte,
    percent_decode_bytes,
    percent_encode_byte,
    percent_encode_bytes,
    percent_encode_len,
};
use crate::{
    MiscCodecError,
    MiscCodecResult,
    internal::{
        ParseError,
        PercentEncodingMode,
    },
    misc_codec_error::map_misc_decode_failure_with_consumed,
};
use qubit_codec::{
    Codec,
    ValueDecoder,
    ValueEncoder,
};

/// Encodes and decodes `application/x-www-form-urlencoded` text fragments.
///
/// Its low-level [`Codec<Value = u8, Unit = u8>`] implementation converts one
/// byte at a time, including the form-specific space and `+` mapping. UTF-8
/// validation remains part of the owned [`decode`](Self::decode) helper.
#[must_use]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FormUrlencodedCodec;

impl FormUrlencodedCodec {
    /// Creates a form-url-encoded codec.
    ///
    /// # Returns
    /// Form URL encoded codec.
    #[inline]
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
    #[inline]
    #[must_use]
    pub fn encode(&self, text: &str) -> String {
        percent_encode_bytes(
            text.as_bytes(),
            PercentEncodingMode::FormUrlencoded,
        )
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
    /// Returns [`MiscCodecError`] when an escape is malformed or decoded bytes
    /// are not valid UTF-8.
    #[inline]
    pub fn decode(&self, text: &str) -> MiscCodecResult<String> {
        String::from_utf8(percent_decode_bytes(text, true)?)
            .map_err(MiscCodecError::from)
    }
}

impl ValueEncoder<str> for FormUrlencodedCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Encodes text, using `+` for spaces.
    #[inline]
    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(FormUrlencodedCodec::encode(self, input))
    }
}

impl ValueDecoder<str> for FormUrlencodedCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Decodes form-url-encoded text.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        FormUrlencodedCodec::decode(self, input)
    }
}

impl Codec for FormUrlencodedCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = MiscCodecError;
    type EncodeError = MiscCodecError;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 3;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 3;

    /// Returns the exact form-url-encoded width for one byte.
    #[inline(always)]
    fn encode_len(&self, value: &u8) -> usize {
        percent_encode_len(*value, PercentEncodingMode::FormUrlencoded)
    }

    /// Decodes one raw byte, `+`, or `%XX` escape.
    ///
    /// # Safety
    /// The caller must provide an input index with at least one readable unit
    /// and a slice satisfying the codec trait's decode preconditions.
    #[inline]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index < input.len());

        let (value, consumed) = percent_decode_byte(input, input_index, true)
            .map_err(|error| {
            ParseError::into_decode_failure_with_consumed(
                error,
                qubit_codec::nz!(3),
            )
        })?;
        debug_assert!(consumed > 0);
        // SAFETY: `percent_decode_byte` returns a non-zero width for every
        // successful raw byte, `+`, or escape.
        let consumed = qubit_codec::nz!(consumed);
        Ok((value, consumed))
    }

    /// Encodes one byte using form URL encoding.
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
        let required =
            percent_encode_len(*value, PercentEncodingMode::FormUrlencoded);
        debug_assert!(output_index + required <= output.len());

        let written = percent_encode_byte(
            *value,
            output,
            output_index,
            PercentEncodingMode::FormUrlencoded,
        );
        debug_assert_eq!(written, required);
        Ok(required)
    }

    /// Decodes one value after end of input has been confirmed.
    ///
    /// # Safety
    /// The caller must provide an input index with at least one readable unit
    /// and a slice satisfying the codec trait's EOF decode preconditions.
    #[inline]
    unsafe fn decode_eof(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index < input.len());

        let (value, consumed) = percent_decode_byte(input, input_index, true)
            .map_err(|error| match error {
            ParseError::Incomplete { .. } => {
                map_misc_decode_failure_with_consumed(
                    MiscCodecError::InvalidEscape {
                        index: input_index,
                        escape: String::from_utf8_lossy(&input[input_index..])
                            .into_owned(),
                        reason: "expected two hexadecimal digits".to_owned(),
                    },
                    qubit_codec::nz!(3),
                )
            }
            ParseError::Invalid(error) => {
                map_misc_decode_failure_with_consumed(
                    error,
                    qubit_codec::nz!(3),
                )
            }
        })?;
        debug_assert!(consumed > 0);
        let consumed = qubit_codec::nz!(consumed);
        Ok((value, consumed))
    }
}
