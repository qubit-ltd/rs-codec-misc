// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Base64 quantum codec.

use crate::{
    Codec,
    MiscCodecError,
    MiscCodecResult,
    misc_codec_error::map_misc_decode_failure,
};

/// Encodes and decodes one complete Base64 quantum.
///
/// A quantum maps exactly three raw bytes to four Base64 units. It does not
/// handle final short input groups or `=` padding; callers that process streams
/// must finalize those cases in a transcoder or facade layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Base64QuantumCodec {
    url_safe: bool,
}

impl Base64QuantumCodec {
    /// Creates a standard-alphabet Base64 quantum codec.
    ///
    /// # Returns
    /// Standard Base64 quantum codec.
    #[inline]
    pub fn standard() -> Self {
        Self { url_safe: false }
    }

    /// Creates a URL-safe-alphabet Base64 quantum codec.
    ///
    /// # Returns
    /// URL-safe Base64 quantum codec.
    #[inline]
    pub fn url_safe() -> Self {
        Self { url_safe: true }
    }

    /// Selects the alphabet for this quantum codec.
    ///
    /// # Returns
    /// The 64-byte alphabet used for encoding.
    #[inline(always)]
    fn alphabet(&self) -> &'static [u8; 64] {
        if self.url_safe {
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"
        } else {
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        }
    }

    /// Converts one Base64 unit to a sextet.
    ///
    /// # Parameters
    /// - `unit`: Encoded Base64 unit.
    /// - `index`: Unit index in the original input.
    ///
    /// # Returns
    /// Sextet value.
    ///
    /// # Errors
    /// Returns [`MiscCodecError::InvalidInput`] when `unit` is not valid for
    /// this quantum codec's alphabet.
    #[inline]
    fn decode_unit(&self, unit: u8, index: usize) -> MiscCodecResult<u8> {
        match unit {
            b'A'..=b'Z' => Ok(unit - b'A'),
            b'a'..=b'z' => Ok(unit - b'a' + 26),
            b'0'..=b'9' => Ok(unit - b'0' + 52),
            b'+' if !self.url_safe => Ok(62),
            b'/' if !self.url_safe => Ok(63),
            b'-' if self.url_safe => Ok(62),
            b'_' if self.url_safe => Ok(63),
            _ => Err(MiscCodecError::InvalidInput {
                codec: "base64-quantum",
                reason: format!(
                    "invalid Base64 unit '{}' at index {}",
                    char::from(unit),
                    index
                ),
            }),
        }
    }
}

impl Default for Base64QuantumCodec {
    /// Creates a standard-alphabet Base64 quantum codec.
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

impl Codec for Base64QuantumCodec {
    type Value = [u8; 3];
    type Unit = u8;
    type DecodeError = MiscCodecError;
    type EncodeError = MiscCodecError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(4);
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(4);

    /// Decodes one complete four-unit Base64 quantum.
    #[inline]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        ([u8; 3], core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index + 4 <= input.len());

        let first = self
            .decode_unit(input[input_index], input_index)
            .map_err(map_misc_decode_failure)?;
        let second = self
            .decode_unit(input[input_index + 1], input_index + 1)
            .map_err(map_misc_decode_failure)?;
        let third = self
            .decode_unit(input[input_index + 2], input_index + 2)
            .map_err(map_misc_decode_failure)?;
        let fourth = self
            .decode_unit(input[input_index + 3], input_index + 3)
            .map_err(map_misc_decode_failure)?;
        Ok((
            [
                (first << 2) | (second >> 4),
                (second << 4) | (third >> 2),
                (third << 6) | fourth,
            ],
            qubit_io::nz!(4),
        ))
    }

    /// Encodes one complete three-byte Base64 quantum.
    #[inline]
    unsafe fn encode(
        &mut self,
        value: &[u8; 3],
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(output_index + 4 <= output.len());

        let alphabet = self.alphabet();
        output[output_index] = alphabet[(value[0] >> 2) as usize];
        output[output_index + 1] =
            alphabet[(((value[0] & 0x03) << 4) | (value[1] >> 4)) as usize];
        output[output_index + 2] =
            alphabet[(((value[1] & 0x0f) << 2) | (value[2] >> 6)) as usize];
        output[output_index + 3] = alphabet[(value[2] & 0x3f) as usize];
        Ok(qubit_io::nz!(4))
    }
}
