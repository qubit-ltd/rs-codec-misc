// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal percent encoding and decoding helpers.

use core::num::NonZeroUsize;

use percent_encoding::percent_encode_byte as encode_percent_byte;
use qubit_utils::nonzero;

use crate::MiscCodecError;
use crate::MiscCodecResult;
use crate::internal::ParseError;
use crate::internal::PercentEncodingMode;

/// Returns the available width of a malformed percent escape.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `index`: Start index of the percent marker.
///
/// # Returns
/// The non-zero number of available units, capped at the three-unit escape
/// width.
#[must_use]
#[inline]
pub(crate) fn percent_invalid_consumed(input: &[u8], index: usize) -> NonZeroUsize {
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
#[must_use]
#[inline]
pub(crate) fn percent_encode_bytes(bytes: &[u8], mode: PercentEncodingMode) -> String {
    let capacity = bytes.iter().map(|byte| percent_encode_len(*byte, mode)).sum();
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
pub(crate) fn percent_decode_bytes(text: &str, plus_as_space: bool) -> MiscCodecResult<Vec<u8>> {
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let (decoded, consumed) = percent_decode_byte(bytes, index, plus_as_space)
            .map_err(|error| percent_parse_error_to_misc(error, bytes, index))?;
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
pub(crate) fn percent_encode_byte(byte: u8, output: &mut [u8], index: usize, mode: PercentEncodingMode) -> usize {
    if mode.is_space_plus(byte) {
        output[index] = b'+';
        return 1;
    }
    if mode.is_unescaped(byte) {
        output[index] = byte;
        return 1;
    }
    output[index..index + 3].copy_from_slice(encode_percent_byte(byte).as_bytes());
    3
}

/// Returns the encoded width for one byte under `mode`.
///
/// `mode` determines whether the byte is emitted directly or escaped.
///
/// # Parameters
/// - `byte`: Byte whose encoded width is measured.
/// - `mode`: Encoding policy selecting the unescaped set and space handling.
///
/// # Returns
/// Number of output units required, either one or three.
#[must_use]
#[inline]
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
/// Returns [`ParseError::Incomplete`] only when the visible escape is a valid
/// prefix that could be completed by additional hexadecimal digits, and
/// [`ParseError::Invalid`] when an available digit makes completion impossible.
#[inline]
pub(crate) fn percent_decode_byte(input: &[u8], index: usize, plus_as_space: bool) -> Result<(u8, usize), ParseError> {
    debug_assert!(index < input.len());
    let available = input.len() - index;
    match input[index] {
        b'%' => {
            if available == 1 {
                return Err(ParseError::Incomplete { required: nonzero(3) });
            }
            let high = percent_hex_value(input[index + 1]).ok_or_else(|| invalid_percent_escape(input, index))?;
            if available == 2 {
                return Err(ParseError::Incomplete { required: nonzero(3) });
            }
            let low = percent_hex_value(input[index + 2]).ok_or_else(|| invalid_percent_escape(input, index))?;
            Ok(((high << 4) | low, 3))
        }
        b'+' if plus_as_space => Ok((b' ', 1)),
        byte => Ok((byte, 1)),
    }
}

/// Decodes one percent value using complete-input rules.
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
/// Returns [`MiscCodecError::InvalidEscape`] when a percent escape is
/// incomplete or contains non-hexadecimal digits.
#[inline]
pub(super) fn percent_decode_byte_eof(
    input: &[u8],
    index: usize,
    plus_as_space: bool,
) -> Result<(u8, usize), MiscCodecError> {
    percent_decode_byte(input, index, plus_as_space).map_err(|error| percent_parse_error_to_misc(error, input, index))
}

/// Converts a streaming parse result into the error used for complete input.
///
/// # Parameters
/// - `error`: Parse failure produced while decoding one value.
/// - `input`: Encoded byte units used to build malformed-escape context.
/// - `index`: Start index of the malformed escape in `input`.
///
/// # Returns
/// The corresponding invalid-escape error for incomplete input, or the
/// original error for an invalid escape.
#[inline]
fn percent_parse_error_to_misc(error: ParseError, input: &[u8], index: usize) -> MiscCodecError {
    match error {
        ParseError::Incomplete { .. } => invalid_percent_escape(input, index),
        ParseError::Invalid(error) => error,
    }
}

/// Builds a malformed percent escape error.
///
/// # Parameters
/// - `input`: Encoded byte units containing the malformed escape.
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
#[inline]
fn percent_hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
