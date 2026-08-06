// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Percent text codec.

use crate::{
    MiscCodecError,
    MiscCodecResult,
    internal::{
        ParseError,
        PercentEncodingMode,
    },
    misc_codec_error::map_misc_decode_failure_with_consumed,
};
use core::num::NonZeroUsize;
use percent_encoding::percent_encode_byte as encode_percent_byte;
use qubit_codec::{
    Codec,
    ValueDecoder,
    ValueEncoder,
};

/// Encodes and decodes percent-encoded UTF-8 text.
///
/// Its low-level [`Codec<Value = u8, Unit = u8>`] implementation converts one
/// byte to either one unreserved ASCII unit or a `%XX` escape. UTF-8 validation
/// remains part of the owned [`decode`](Self::decode) helper.
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
        String::from_utf8(percent_decode_bytes(text, false)?)
            .map_err(MiscCodecError::from)
    }
}

impl ValueEncoder<str> for PercentCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Encodes text using percent encoding.
    #[inline]
    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(PercentCodec::encode(self, input))
    }
}

impl ValueDecoder<str> for PercentCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Decodes percent-encoded text.
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

    /// Returns the exact percent-encoded width for one byte.
    #[inline(always)]
    fn encode_len(&self, value: &u8) -> usize {
        percent_encode_len(*value, PercentEncodingMode::Rfc3986)
    }

    /// Decodes one raw byte or `%XX` escape.
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

        let (value, consumed) = percent_decode_byte(input, input_index, false)
            .map_err(|error| {
                ParseError::into_decode_failure_with_consumed(
                    error,
                    qubit_utils::nonzero!(3),
                )
            })?;
        debug_assert!(consumed > 0);
        // SAFETY: `percent_decode_byte` returns a non-zero width for every
        // successful raw byte or escape.
        let consumed = qubit_utils::nonzero!(consumed);
        Ok((value, consumed))
    }

    /// Encodes one byte using percent encoding.
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

        let written = percent_encode_byte(
            *value,
            output,
            output_index,
            PercentEncodingMode::Rfc3986,
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

        let (value, consumed) =
            percent_decode_byte_eof(input, input_index, false).map_err(
                |error| {
                    map_misc_decode_failure_with_consumed(
                        error,
                        percent_invalid_consumed(input, input_index),
                    )
                },
            )?;
        debug_assert!(consumed > 0);
        let consumed = qubit_utils::nonzero!(consumed);
        Ok((value, consumed))
    }
}

/// Returns the available width of a malformed percent escape.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `index`: Start index of the percent marker.
///
/// # Returns
/// The non-zero number of available units, capped at the three-unit escape
/// width.
#[inline(always)]
pub(crate) fn percent_invalid_consumed(
    input: &[u8],
    index: usize,
) -> NonZeroUsize {
    NonZeroUsize::new(input.len().saturating_sub(index).clamp(1, 3))
        .expect("percent decode errors have at least one available unit")
}

/// Percent-encodes UTF-8 bytes.
///
/// # Parameters
/// - `bytes`: Bytes to encode.
/// - `mode`: Encoding policy selecting the unescaped set and space handling.
///
/// # Returns
/// Encoded text.
#[inline]
pub(crate) fn percent_encode_bytes(
    bytes: &[u8],
    mode: PercentEncodingMode,
) -> String {
    let capacity = bytes
        .iter()
        .map(|byte| percent_encode_len(*byte, mode))
        .sum();
    let mut output = String::with_capacity(capacity);
    for byte in bytes {
        if mode.is_space_plus(*byte) {
            output.push('+');
        } else if mode.is_unescaped(*byte) {
            output.push(*byte as char);
        } else {
            output.push_str(encode_percent_byte(*byte));
        }
    }
    output
}

/// Percent-decodes UTF-8 bytes.
///
/// # Parameters
/// - `text`: Text to decode.
/// - `plus_as_space`: Whether `+` should decode to a space byte.
///
/// # Returns
/// Decoded bytes.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidEscape`] for malformed escapes.
#[inline]
pub(crate) fn percent_decode_bytes(
    text: &str,
    plus_as_space: bool,
) -> MiscCodecResult<Vec<u8>> {
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let (decoded, consumed) =
            percent_decode_byte(bytes, index, plus_as_space).map_err(
                |error| percent_parse_error_to_misc(error, bytes, index),
            )?;
        output.push(decoded);
        index += consumed;
    }
    Ok(output)
}

/// Percent-encodes one byte into `output`.
///
/// # Parameters
/// - `byte`: Byte to encode.
/// - `output`: Destination unit buffer.
/// - `index`: Start index in `output`.
/// - `mode`: Encoding policy selecting the unescaped set and space handling.
///
/// # Returns
/// Number of units written.
#[inline]
pub(crate) fn percent_encode_byte(
    byte: u8,
    output: &mut [u8],
    index: usize,
    mode: PercentEncodingMode,
) -> usize {
    if mode.is_space_plus(byte) {
        output[index] = b'+';
        return 1;
    }
    if mode.is_unescaped(byte) {
        output[index] = byte;
        return 1;
    }
    output[index..index + 3]
        .copy_from_slice(encode_percent_byte(byte).as_bytes());
    3
}

/// Returns the encoded width for one byte under `mode`.
#[inline(always)]
pub(crate) fn percent_encode_len(byte: u8, mode: PercentEncodingMode) -> usize {
    if mode.is_space_plus(byte) || mode.is_unescaped(byte) {
        1
    } else {
        3
    }
}

/// Decodes one raw byte or `%XX` escape from `input`.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `index`: Start index in `input`.
/// - `plus_as_space`: Whether `+` should decode to a space byte.
///
/// # Returns
/// Decoded byte and the number of consumed units.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidEscape`] for malformed `%XX` escapes.
#[inline]
pub(crate) fn percent_decode_byte(
    input: &[u8],
    index: usize,
    plus_as_space: bool,
) -> Result<(u8, usize), ParseError> {
    debug_assert!(index < input.len());
    let available = input.len() - index;
    match input[index] {
        b'%' => {
            if available < 3 {
                return Err(ParseError::Incomplete {
                    required: qubit_utils::nonzero!(3),
                });
            }
            let high_byte = input[index + 1];
            let low_byte = input[index + 2];
            let high = percent_hex_value(high_byte)
                .ok_or_else(|| invalid_percent_escape(input, index))?;
            let low = percent_hex_value(low_byte)
                .ok_or_else(|| invalid_percent_escape(input, index))?;
            Ok(((high << 4) | low, 3))
        }
        b'+' if plus_as_space => Ok((b' ', 1)),
        byte => Ok((byte, 1)),
    }
}

/// Decodes one percent value using EOF rules.
#[inline]
fn percent_decode_byte_eof(
    input: &[u8],
    index: usize,
    plus_as_space: bool,
) -> Result<(u8, usize), MiscCodecError> {
    percent_decode_byte(input, index, plus_as_space)
        .map_err(|error| percent_parse_error_to_misc(error, input, index))
}

/// Converts an open-stream parse result into a complete-input error.
#[inline]
fn percent_parse_error_to_misc(
    error: ParseError,
    input: &[u8],
    index: usize,
) -> MiscCodecError {
    match error {
        ParseError::Incomplete { .. } => invalid_percent_escape(input, index),
        ParseError::Invalid(error) => error,
    }
}

/// Builds a malformed percent escape error.
///
/// # Parameters
/// - `index`: Byte index of the `%` marker in the original input.
///
/// # Returns
/// An invalid escape error for a `%XX` sequence.
fn invalid_percent_escape(input: &[u8], index: usize) -> MiscCodecError {
    let end = index.saturating_add(3).min(input.len());
    MiscCodecError::InvalidEscape {
        index,
        escape: String::from_utf8_lossy(&input[index..end]).into_owned(),
        reason: "expected two hexadecimal digits".to_owned(),
    }
}

/// Converts one ASCII hex byte to its nibble value.
///
/// # Parameters
/// - `byte`: ASCII byte to inspect.
///
/// # Returns
/// Nibble value, or `None` when `byte` is not hex.
#[inline(always)]
fn percent_hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
