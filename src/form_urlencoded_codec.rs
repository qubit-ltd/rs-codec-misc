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
};
use crate::{
    Codec,
    MiscCodecError,
    MiscCodecResult,
    ValueDecoder,
    ValueEncoder,
    misc_codec_error::map_misc_decode_failure,
};

/// Encodes and decodes `application/x-www-form-urlencoded` text fragments.
///
/// Its low-level [`Codec<Value = u8, Unit = u8>`] implementation converts one
/// byte at a time, including the form-specific space and `+` mapping. UTF-8
/// validation remains part of the owned [`decode`](Self::decode) helper.
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

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(1);
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(3);

    /// Returns the exact form-url-encoded width for one byte.
    #[inline(always)]
    fn encode_len(&self, value: &u8) -> core::num::NonZeroUsize {
        if *value == b' '
            || value.is_ascii_alphanumeric()
            || matches!(*value, b'-' | b'.' | b'_' | b'~')
        {
            qubit_io::nz!(1)
        } else {
            qubit_io::nz!(3)
        }
    }

    /// Decodes one raw byte, `+`, or `%XX` escape.
    #[inline]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(index < input.len());

        let (value, consumed) = percent_decode_byte(input, index, true)
            .map_err(map_misc_decode_failure)?;
        debug_assert!(consumed > 0);
        // SAFETY: `percent_decode_byte` returns a non-zero width for every
        // successful raw byte, `+`, or escape.
        let consumed = qubit_io::nz!(consumed);
        Ok((value, consumed))
    }

    /// Encodes one byte using form URL encoding.
    #[inline]
    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(
            index
                + if *value == b' '
                    || value.is_ascii_alphanumeric()
                    || matches!(*value, b'-' | b'.' | b'_' | b'~')
                {
                    1
                } else {
                    3
                }
                <= output.len()
        );

        let written = percent_encode_byte(*value, output, index, true);
        let required = <Self as Codec>::encode_len(self, value);
        debug_assert_eq!(written, required.get());
        Ok(required)
    }
}
