// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! C string literal byte codec.

use core::num::NonZeroUsize;

use qubit_codec::Codec;
use qubit_codec::DecodeFailure;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_utils::nonzero;

use crate::MiscCodecError;
use crate::MiscCodecResult;
use crate::internal::CStringLiteralParseContext;
use crate::internal::ParseError;
use crate::misc_codec_error::map_misc_decode_failure_with_consumed;

const UPPER_HEX_DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E',
    'F',
];

/// Encodes and decodes byte-oriented C string literal fragments.
///
/// This codec is intended for textual formats that embed byte sequences with C
/// escapes, such as `PK\003\004` or `\xd0\xcf`. It decodes into raw bytes and
/// does not require surrounding quotes.
///
/// Its low-level [`Codec<Value = u8, Unit = u8>`] implementation handles one
/// raw byte or one C escape fragment. Whole-fragment iteration remains part of
/// the owned [`encode`](Self::encode) and [`decode`](Self::decode) helpers.
#[must_use]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CStringLiteralCodec;

impl CStringLiteralCodec {
    /// Creates a C string literal codec.
    ///
    /// # Returns
    /// A stateless C string literal codec.
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Encodes bytes into a C string literal fragment.
    ///
    /// # Parameters
    /// - `bytes`: Raw bytes to encode.
    ///
    /// # Returns
    /// A C string literal fragment without surrounding quotes.
    #[inline]
    #[must_use]
    pub fn encode(&self, bytes: &[u8]) -> String {
        let mut output = String::with_capacity(bytes.len());
        for byte in bytes {
            push_encoded_byte(*byte, &mut output);
        }
        output
    }

    /// Decodes a C string literal fragment into bytes.
    ///
    /// # Parameters
    /// - `text`: C string literal fragment without surrounding quotes.
    ///
    /// # Returns
    /// Decoded raw bytes.
    ///
    /// # Errors
    /// Returns [`MiscCodecError::InvalidEscape`] for malformed escape
    /// sequences, [`MiscCodecError::InvalidDigit`] for malformed
    /// fixed-width numeric escapes,
    /// and [`MiscCodecError::InvalidCharacter`] for unsupported raw source
    /// characters.
    #[inline]
    pub fn decode(&self, text: &str) -> MiscCodecResult<Vec<u8>> {
        let input = text.as_bytes();
        let mut output = Vec::with_capacity(text.len());
        let mut index = 0;
        while index < input.len() {
            let (decoded, consumed) = decode_c_string_literal_unit(
                input,
                index,
                CStringLiteralParseContext::CompleteText(text),
            )
            .map_err(|error| {
                c_string_parse_error_to_misc(error, input, index)
            })?;
            debug_assert!(consumed > 0);
            output.push(decoded);
            index += consumed;
        }
        Ok(output)
    }
}

impl ValueEncoder<[u8]> for CStringLiteralCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Encodes bytes into a C string literal fragment.
    #[inline]
    fn encode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(CStringLiteralCodec::encode(self, input))
    }
}

impl ValueDecoder<str> for CStringLiteralCodec {
    type Error = MiscCodecError;
    type Output = Vec<u8>;

    /// Decodes a C string literal fragment into bytes.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        CStringLiteralCodec::decode(self, input)
    }
}

impl Codec for CStringLiteralCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = MiscCodecError;
    type EncodeError = MiscCodecError;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 4;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 10;

    /// Returns the exact C string literal width for one byte.
    #[inline(always)]
    fn encode_len(&self, value: &u8) -> usize {
        encoded_byte_len(*value)
    }

    /// Decodes one raw byte or one C escape fragment.
    ///
    /// # Safety
    /// The caller must provide an input index with at least one readable unit
    /// and a slice satisfying the codec trait's decode preconditions.
    #[inline]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        debug_assert!(input_index < input.len());

        let (value, consumed) =
            decode_c_string_literal_byte(input, input_index).map_err(
                |error| {
                    map_c_string_parse_error_to_decode_failure(
                        error,
                        input,
                        input_index,
                    )
                },
            )?;
        debug_assert!(consumed > 0);
        // SAFETY: `decode_c_string_literal_byte` returns a non-zero width for
        // every successful raw byte or escape.
        let consumed = nonzero(consumed);
        Ok((value, consumed))
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
    ) -> Result<(u8, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        debug_assert!(input_index < input.len());

        let (value, consumed) = decode_c_string_literal_unit(
            input,
            input_index,
            CStringLiteralParseContext::EofBytes,
        )
        .map_err(|error| {
            map_c_string_parse_error_to_decode_failure(
                error,
                input,
                input_index,
            )
        })?;
        debug_assert!(consumed > 0);
        let consumed = nonzero(consumed);
        Ok((value, consumed))
    }

    /// Encodes one byte as a raw byte or C escape fragment.
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
        let required = encoded_byte_len(*value);
        debug_assert!(output_index + required <= output.len());

        let written = write_encoded_byte(*value, output, output_index);
        debug_assert_eq!(written, required);
        Ok(required)
    }
}

/// Encodes one byte into the destination string.
///
/// # Parameters
/// - `byte`: Byte to encode.
/// - `output`: Destination string.
#[inline]
fn push_encoded_byte(byte: u8, output: &mut String) {
    match byte {
        b'\'' => output.push_str("\\'"),
        b'"' => output.push_str("\\\""),
        b'?' => output.push_str("\\?"),
        b'\\' => output.push_str("\\\\"),
        0x07 => output.push_str("\\a"),
        0x08 => output.push_str("\\b"),
        0x0c => output.push_str("\\f"),
        b'\n' => output.push_str("\\n"),
        b'\r' => output.push_str("\\r"),
        b'\t' => output.push_str("\\t"),
        0x0b => output.push_str("\\v"),
        b' '..=b'~' => output.push(byte as char),
        _ => {
            output.push('\\');
            output.push('x');
            output.push(uppercase_hex_digit(byte >> 4));
            output.push(uppercase_hex_digit(byte & 0x0f));
        }
    }
}

/// Decodes one byte-oriented C string literal fragment from `input`.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `index`: Start index in `input`.
///
/// # Returns
/// Decoded byte and consumed unit count.
///
/// # Errors
/// Returns [`MiscCodecError`] when the raw byte or escape fragment is invalid.
#[inline]
fn decode_c_string_literal_byte(
    input: &[u8],
    index: usize,
) -> Result<(u8, usize), ParseError> {
    decode_c_string_literal_unit(
        input,
        index,
        CStringLiteralParseContext::StreamingBytes,
    )
}

/// Decodes one C string literal unit from `input`.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `index`: Start index in `input`.
/// - `context`: Complete-text or streaming-byte parsing context.
///
/// # Returns
/// Decoded byte and consumed unit count.
///
/// # Errors
/// Returns [`MiscCodecError`] when the raw byte or escape fragment is invalid.
#[inline]
fn decode_c_string_literal_unit(
    input: &[u8],
    index: usize,
    context: CStringLiteralParseContext<'_>,
) -> Result<(u8, usize), ParseError> {
    debug_assert!(index < input.len());
    let available = input.len() - index;
    let byte = input[index];
    if byte != b'\\' {
        validate_source_unit(input, index, byte, context)?;
        return Ok((byte, 1));
    }
    if available < 2 {
        return Err(context.trailing_escape_error(index, available));
    }
    let escape = input[index + 1];
    match escape {
        b' ' => Ok((b' ', 2)),
        b'\'' => Ok((b'\'', 2)),
        b'"' => Ok((b'"', 2)),
        b'?' => Ok((b'?', 2)),
        b'\\' => Ok((b'\\', 2)),
        b'a' => Ok((0x07, 2)),
        b'b' => Ok((0x08, 2)),
        b'f' => Ok((0x0c, 2)),
        b'n' => Ok((b'\n', 2)),
        b'r' => Ok((b'\r', 2)),
        b't' => Ok((b'\t', 2)),
        b'v' => Ok((0x0b, 2)),
        b'x' | b'X' => {
            if !context.is_complete_text() {
                ensure_variable_hex_escape_complete(input, index, available)?;
            }
            parse_variable_hex_escape_units(input, index)
        }
        b'u' => {
            if matches!(context, CStringLiteralParseContext::CompleteText(_))
                || context.is_streaming()
            {
                ensure_fixed_escape_complete(available, nonzero(6))?;
            }
            parse_fixed_hex_escape_units(input, index, 4, context)
        }
        b'U' => {
            if matches!(context, CStringLiteralParseContext::CompleteText(_))
                || context.is_streaming()
            {
                ensure_fixed_escape_complete(available, nonzero(10))?;
            }
            parse_fixed_hex_escape_units(input, index, 8, context)
        }
        b'0'..=b'7' => {
            if context.is_streaming() {
                ensure_octal_escape_complete(input, index, available)?;
            }
            Ok(parse_octal_escape_units(input, index))
        }
        _ => Err(invalid_escape(
            index,
            &context.escape_fragment(input, index, index + 2),
            "unsupported escape sequence",
        )
        .into()),
    }
}

/// Ensures a variable-width `\x` escape has enough units to decide one value.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `index`: Start index of the escape marker.
/// - `available`: Available unit count from `index`.
///
/// # Errors
/// Returns an internal incomplete parser outcome when more units are required.
#[inline]
fn ensure_variable_hex_escape_complete(
    input: &[u8],
    index: usize,
    available: usize,
) -> Result<(), ParseError> {
    if available < 3 {
        return Err(ParseError::Incomplete {
            required: nonzero(3),
        });
    }
    let mut digit_count = 0usize;
    while digit_count < 2 {
        let Some(&byte) = input.get(index + 2 + digit_count) else {
            break;
        };
        if hex_value(char::from(byte)).is_none() {
            break;
        }
        digit_count += 1;
    }
    if digit_count == 1 && index + 3 == input.len() {
        return Err(ParseError::Incomplete {
            required: nonzero(4),
        });
    }
    Ok(())
}

/// Ensures a fixed-width universal byte escape has enough units.
///
/// # Parameters
/// - `available`: Available unit count from `index`.
/// - `required`: Required unit count for this escape form.
///
/// # Errors
/// Returns an internal incomplete parser outcome when more units are required.
#[inline]
fn ensure_fixed_escape_complete(
    available: usize,
    required: core::num::NonZeroUsize,
) -> Result<(), ParseError> {
    if available < required.get() {
        return Err(ParseError::Incomplete { required });
    }
    Ok(())
}

/// Ensures an octal escape has enough units to decide one value.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `index`: Start index of the escape marker.
/// - `available`: Available unit count from `index`.
///
/// # Errors
/// Returns an internal incomplete parser outcome when more units are required.
#[inline]
fn ensure_octal_escape_complete(
    input: &[u8],
    index: usize,
    _available: usize,
) -> Result<(), ParseError> {
    let mut digit_count = 0usize;
    while digit_count < 3 {
        let Some(&byte) = input.get(index + 1 + digit_count) else {
            break;
        };
        if octal_value(char::from(byte)).is_none() {
            break;
        }
        digit_count += 1;
    }
    if digit_count < 3 && index + 1 + digit_count == input.len() {
        return Err(ParseError::Incomplete {
            required: nonzero(2 + digit_count),
        });
    }
    Ok(())
}

/// Validates a raw source unit.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `index`: Byte index in the encoded input.
/// - `byte`: Raw source byte.
/// - `context`: Parsing context used for diagnostics.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidCharacter`] when the byte is not allowed as
/// a raw C string source byte.
#[inline]
fn validate_source_unit(
    input: &[u8],
    index: usize,
    byte: u8,
    context: CStringLiteralParseContext<'_>,
) -> Result<(), ParseError> {
    if matches!(byte, b'\t' | b'\n' | 0x0b | 0x0c | b' '..=b'~') {
        return Ok(());
    }
    Err(MiscCodecError::InvalidCharacter {
        index,
        character: context.source_character(input, index),
        reason: context.raw_source_reason().to_owned(),
    }
    .into())
}

/// Parses a byte-oriented `\x` escape from `input`.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `marker_index`: Byte index of the escape marker.
///
/// # Returns
/// Decoded byte and consumed unit count.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidEscape`] when no hexadecimal digit follows
/// `\x`.
#[inline]
fn parse_variable_hex_escape_units(
    input: &[u8],
    marker_index: usize,
) -> Result<(u8, usize), ParseError> {
    let mut value = 0u8;
    let mut digit_count = 0usize;
    let mut index = marker_index + 2;
    while digit_count < 2 {
        let Some(&byte) = input.get(index) else {
            break;
        };
        let Some(digit) = hex_value(char::from(byte)) else {
            break;
        };
        value = (value << 4) | digit;
        index += 1;
        digit_count += 1;
    }
    if digit_count == 0 {
        return Err(invalid_escape(
            marker_index,
            "\\x",
            "expected at least one hexadecimal digit",
        )
        .into());
    }
    Ok((value, 2 + digit_count))
}

/// Parses a fixed-width universal byte escape from `input`.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `marker_index`: Byte index of the escape marker.
/// - `digits`: Required hexadecimal digit count.
///
/// # Returns
/// Decoded byte and consumed unit count.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidEscape`] when the escape is incomplete or
/// larger than one byte, or [`MiscCodecError::InvalidDigit`] when a required
/// digit is not hexadecimal.
#[inline]
fn parse_fixed_hex_escape_units(
    input: &[u8],
    marker_index: usize,
    digits: usize,
    context: CStringLiteralParseContext<'_>,
) -> Result<(u8, usize), ParseError> {
    let mut value = 0u32;
    let mut index = marker_index + 2;
    for _ in 0..digits {
        let Some(_) = input.get(index) else {
            return Err(invalid_escape(
                marker_index,
                &context.escape_fragment(input, marker_index, input.len()),
                "incomplete universal character escape",
            )
            .into());
        };
        let character = context.source_character(input, index);
        let Some(digit) = hex_value(character) else {
            return Err(MiscCodecError::InvalidDigit {
                radix: 16,
                index,
                character,
            }
            .into());
        };
        value = (value << 4) | u32::from(digit);
        index += 1;
    }
    if value > u32::from(u8::MAX) {
        return Err(invalid_escape(
            marker_index,
            &context.escape_fragment(input, marker_index, index),
            "universal character value must fit in one byte",
        )
        .into());
    }
    Ok((value as u8, 2 + digits))
}

/// Parses an octal byte escape from `input`.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `marker_index`: Byte index of the escape marker.
///
/// # Returns
/// Decoded byte and consumed unit count. Values above `0o377` are truncated to
/// their low byte to match the owned decoder.
#[inline]
fn parse_octal_escape_units(input: &[u8], marker_index: usize) -> (u8, usize) {
    let mut value = 0u16;
    let mut digit_count = 0usize;
    let mut index = marker_index + 1;
    while digit_count < 3 {
        let Some(&byte) = input.get(index) else {
            break;
        };
        let Some(digit) = octal_value(char::from(byte)) else {
            break;
        };
        value = (value << 3) | u16::from(digit);
        index += 1;
        digit_count += 1;
    }
    (value as u8, 1 + digit_count)
}

/// Returns the encoded width for one byte.
///
/// # Parameters
/// - `byte`: Byte to inspect.
///
/// # Returns
/// Number of units written by [`write_encoded_byte`].
#[must_use]
#[inline(always)]
fn encoded_byte_len(byte: u8) -> usize {
    match byte {
        b'\'' | b'"' | b'?' | b'\\' | 0x07 | 0x08 | 0x0c | b'\n' | b'\r'
        | b'\t' | 0x0b => 2,
        b' '..=b'~' => 1,
        _ => 4,
    }
}

/// Encodes one byte into `output`.
///
/// # Parameters
/// - `byte`: Byte to encode.
/// - `output`: Destination unit buffer.
/// - `index`: Start index in `output`.
///
/// # Returns
/// Number of units written.
#[inline]
fn write_encoded_byte(byte: u8, output: &mut [u8], index: usize) -> usize {
    match byte {
        b'\'' => write_ascii_escape(output, index, b'\''),
        b'"' => write_ascii_escape(output, index, b'"'),
        b'?' => write_ascii_escape(output, index, b'?'),
        b'\\' => write_ascii_escape(output, index, b'\\'),
        0x07 => write_ascii_escape(output, index, b'a'),
        0x08 => write_ascii_escape(output, index, b'b'),
        0x0c => write_ascii_escape(output, index, b'f'),
        b'\n' => write_ascii_escape(output, index, b'n'),
        b'\r' => write_ascii_escape(output, index, b'r'),
        b'\t' => write_ascii_escape(output, index, b't'),
        0x0b => write_ascii_escape(output, index, b'v'),
        b' '..=b'~' => {
            output[index] = byte;
            1
        }
        _ => {
            output[index] = b'\\';
            output[index + 1] = b'x';
            output[index + 2] = uppercase_hex_digit(byte >> 4) as u8;
            output[index + 3] = uppercase_hex_digit(byte & 0x0f) as u8;
            4
        }
    }
}

/// Writes a two-unit backslash escape.
///
/// # Parameters
/// - `output`: Destination unit buffer.
/// - `index`: Start index in `output`.
/// - `escape`: ASCII escape letter after the backslash.
///
/// # Returns
/// Number of units written.
#[inline(always)]
fn write_ascii_escape(output: &mut [u8], index: usize, escape: u8) -> usize {
    output[index] = b'\\';
    output[index + 1] = escape;
    2
}

/// Builds an ASCII-ish escape fragment from encoded units.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `start`: Start index.
/// - `end`: Exclusive end index.
///
/// # Returns
/// String fragment used in diagnostics.
fn escape_fragment(input: &[u8], start: usize, end: usize) -> String {
    let bounded_end = end.min(input.len());
    input[start..bounded_end]
        .iter()
        .map(|byte| char::from(*byte))
        .collect()
}

/// Converts one hexadecimal character to its nibble value.
///
/// # Parameters
/// - `character`: Character to inspect.
///
/// # Returns
/// Nibble value, or `None` when `character` is not hexadecimal.
#[inline(always)]
fn hex_value(character: char) -> Option<u8> {
    match character {
        '0'..='9' => Some(character as u8 - b'0'),
        'a'..='f' => Some(character as u8 - b'a' + 10),
        'A'..='F' => Some(character as u8 - b'A' + 10),
        _ => None,
    }
}

/// Converts one octal character to its value.
///
/// # Parameters
/// - `character`: Character to inspect.
///
/// # Returns
/// Octal digit value, or `None` when `character` is not octal.
#[inline(always)]
fn octal_value(character: char) -> Option<u8> {
    match character {
        '0'..='7' => Some(character as u8 - b'0'),
        _ => None,
    }
}

/// Converts one nibble to an uppercase hexadecimal digit.
///
/// # Parameters
/// - `value`: Nibble value. Values above `0x0f` are masked to their low nibble.
///
/// # Returns
/// Uppercase hexadecimal digit.
#[inline(always)]
fn uppercase_hex_digit(value: u8) -> char {
    UPPER_HEX_DIGITS[(value & 0x0f) as usize]
}

/// Converts a parser outcome into an owned complete-input error.
///
/// # Parameters
/// - `error`: Parser outcome to convert.
/// - `input`: Encoded source units.
/// - `index`: Start index of the current value.
///
/// # Returns
/// A concrete codec error suitable for complete or EOF-confirmed input.
#[inline]
fn c_string_parse_error_to_misc(
    error: ParseError,
    input: &[u8],
    index: usize,
) -> MiscCodecError {
    match error {
        ParseError::Incomplete { required, .. } => {
            MiscCodecError::InvalidEscape {
                index,
                escape: escape_fragment(input, index, input.len()),
                reason: format!(
                    "incomplete escape sequence; expected at least {required} units"
                ),
            }
        }
        ParseError::Invalid(error) => error,
    }
}

/// Maps a C string parser outcome while retaining the malformed fragment width.
#[inline]
fn map_c_string_parse_error_to_decode_failure(
    error: ParseError,
    input: &[u8],
    index: usize,
) -> DecodeFailure<MiscCodecError> {
    match error {
        ParseError::Incomplete { required, .. } => {
            DecodeFailure::incomplete(required)
        }
        ParseError::Invalid(error) => map_misc_decode_failure_with_consumed(
            error,
            c_string_invalid_consumed(input, index),
        ),
    }
}

/// Returns the number of units belonging to a malformed C string fragment.
#[inline]
fn c_string_invalid_consumed(input: &[u8], index: usize) -> NonZeroUsize {
    let available = input.len().saturating_sub(index);
    let width = match input.get(index..index.saturating_add(2)) {
        Some([b'\\', b'u']) => 6,
        Some([b'\\', b'U']) => 10,
        Some([b'\\', _]) => 2,
        _ => 1,
    };
    NonZeroUsize::new(width.min(available).max(1))
        .expect("malformed C string input has at least one unit")
}

/// Builds an invalid escape error.
///
/// # Parameters
/// - `index`: Byte index of the escape marker in the original input.
/// - `escape`: Escape fragment that caused the error.
/// - `reason`: Human-readable rejection reason.
///
/// # Returns
/// An invalid escape error.
pub(crate) fn invalid_escape(
    index: usize,
    escape: &str,
    reason: &str,
) -> MiscCodecError {
    MiscCodecError::InvalidEscape {
        index,
        escape: escape.to_owned(),
        reason: reason.to_owned(),
    }
}
