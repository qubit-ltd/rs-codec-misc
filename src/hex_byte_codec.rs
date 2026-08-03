// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Low-level hexadecimal byte codec.

use crate::{
    MiscCodecError,
    hex_codec::{
        hex_digit,
        hex_value,
        invalid_hex_digit,
    },
    misc_codec_error::map_misc_decode_failure_with_consumed,
};
use qubit_codec::Codec;

/// Encodes and decodes one byte as two ASCII hexadecimal units.
///
/// `HexByteCodec` is the low-level [`Codec`] implementation for streaming or
/// generic codec call sites. It does not understand whole-string prefixes,
/// per-byte prefixes, separators, or whitespace. Use [`crate::HexCodec`] for
/// owned byte-slice helpers with those formatting options.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HexByteCodec {
    uppercase: bool,
}

impl HexByteCodec {
    /// Creates a lowercase single-byte hexadecimal codec.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self { uppercase: false }
    }

    /// Creates an uppercase single-byte hexadecimal codec.
    #[must_use]
    #[inline]
    pub const fn upper() -> Self {
        Self { uppercase: true }
    }

    /// Sets whether encoded digits should be uppercase.
    #[must_use]
    #[inline]
    pub const fn with_uppercase(mut self, uppercase: bool) -> Self {
        self.uppercase = uppercase;
        self
    }

    /// Returns whether this byte codec emits uppercase hexadecimal digits.
    #[must_use]
    #[inline]
    pub const fn is_uppercase(self) -> bool {
        self.uppercase
    }
}

impl Codec for HexByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = MiscCodecError;
    type EncodeError = MiscCodecError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    /// Decodes one byte from two ASCII hexadecimal digits.
    ///
    /// # Safety
    /// The caller must provide two readable units at `input_index`.
    #[inline]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index + 2 <= input.len());

        let high_char = char::from(input[input_index]);
        let low_char = char::from(input[input_index + 1]);
        let high = hex_value(high_char)
            .ok_or_else(|| invalid_hex_digit(input_index, high_char))
            .map_err(|error| {
                map_misc_decode_failure_with_consumed(
                    error,
                    qubit_codec::nz!(2),
                )
            })?;
        let low = hex_value(low_char)
            .ok_or_else(|| invalid_hex_digit(input_index + 1, low_char))
            .map_err(|error| {
                map_misc_decode_failure_with_consumed(
                    error,
                    qubit_codec::nz!(2),
                )
            })?;
        Ok(((high << 4) | low, qubit_codec::nz!(2)))
    }

    /// Encodes one byte as two ASCII hexadecimal digits.
    ///
    /// # Safety
    /// The caller must provide two writable output units at `output_index`.
    #[inline]
    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(output_index + 2 <= output.len());

        output[output_index] = hex_digit(*value >> 4, self.uppercase) as u8;
        output[output_index + 1] =
            hex_digit(*value & 0x0f, self.uppercase) as u8;
        Ok(2)
    }
}
