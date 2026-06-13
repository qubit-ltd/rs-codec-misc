// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! C string literal byte codec.

use crate::{Codec, MiscCodecError, MiscCodecResult, ValueDecoder, ValueEncoder};

const UPPER_HEX_DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
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
            )?;
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

unsafe impl Codec for CStringLiteralCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = MiscCodecError;
    type EncodeError = MiscCodecError;

    /// Returns the shortest representation length for one byte.
    #[inline(always)]
    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    /// Returns the longest supported universal escape length for one byte.
    #[inline(always)]
    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        qubit_codec::nz!(10)
    }

    /// Returns the exact C string literal width for one byte.
    #[inline(always)]
    fn encode_len(&self, value: &u8) -> core::num::NonZeroUsize {
        encoded_byte_len(*value)
    }

    /// Decodes one raw byte or one C escape fragment.
    #[inline]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        let (value, consumed) = decode_c_string_literal_byte(input, index)?;
        debug_assert!(consumed > 0);
        // SAFETY: `decode_c_string_literal_byte` returns a non-zero width for
        // every successful raw byte or escape.
        let consumed = unsafe { core::num::NonZeroUsize::new_unchecked(consumed) };
        Ok((value, consumed))
    }

    /// Encodes one byte as a raw byte or C escape fragment.
    #[inline]
    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        let required = encoded_byte_len(*value);
        debug_assert!(index + required.get() <= output.len());

        let written = write_encoded_byte(*value, output, index);
        debug_assert_eq!(written, required.get());
        Ok(required)
    }
}

/// Parsing context for one C string literal unit.
///
/// Complete text parsing preserves owned decoder diagnostics, while streaming
/// byte parsing reports incomplete fragments so buffered callers can retry.
#[derive(Debug, Clone, Copy)]
enum CStringLiteralParseContext<'a> {
    /// Parsing a complete UTF-8 literal fragment.
    CompleteText(&'a str),
    /// Parsing one byte unit for a streaming codec caller.
    StreamingBytes,
}

impl CStringLiteralParseContext<'_> {
    /// Tests whether parsing is for a complete text fragment.
    ///
    /// # Returns
    /// `true` when incomplete trailing escapes should be reported as malformed
    /// complete input instead of as retryable incomplete input.
    #[inline(always)]
    fn is_complete_text(self) -> bool {
        matches!(self, Self::CompleteText(_))
    }

    /// Builds the error for a trailing escape marker.
    ///
    /// # Parameters
    /// - `marker_index`: Byte index of the escape marker.
    /// - `available`: Available unit count from `marker_index`.
    ///
    /// # Returns
    /// A malformed escape error for complete text, or an incomplete-input error
    /// for streaming byte parsing.
    fn trailing_escape_error(self, marker_index: usize, available: usize) -> MiscCodecError {
        match self {
            Self::CompleteText(_) => {
                invalid_escape(marker_index, "\\", "incomplete escape sequence")
            }
            Self::StreamingBytes => MiscCodecError::Incomplete {
                required: 2,
                available,
            },
        }
    }

    /// Gets the source character at a byte index for diagnostics.
    ///
    /// # Parameters
    /// - `input`: Encoded byte units.
    /// - `index`: Byte index to inspect.
    ///
    /// # Returns
    /// The UTF-8 source character for complete text, or the byte mapped to a
    /// Unicode scalar value for byte parsing.
    fn source_character(self, input: &[u8], index: usize) -> char {
        match self {
            Self::CompleteText(text) => text
                .get(index..)
                .and_then(|rest| rest.chars().next())
                .unwrap_or(char::from(input[index])),
            Self::StreamingBytes => char::from(input[index]),
        }
    }

    /// Builds a raw source character rejection reason.
    ///
    /// # Returns
    /// The diagnostic reason matching the parsing context.
    #[inline(always)]
    fn raw_source_reason(self) -> &'static str {
        match self {
            Self::CompleteText(_) => {
                "raw source character must be printable ASCII or allowed whitespace"
            }
            Self::StreamingBytes => "raw source byte must be printable ASCII or allowed whitespace",
        }
    }

    /// Builds an escape fragment for diagnostics.
    ///
    /// # Parameters
    /// - `input`: Encoded byte units.
    /// - `start`: Start byte index.
    /// - `end`: Exclusive fallback byte end index for byte parsing.
    ///
    /// # Returns
    /// A displayable escape fragment.
    fn escape_fragment(self, input: &[u8], start: usize, end: usize) -> String {
        match self {
            Self::CompleteText(text) => text
                .get(start..end)
                .or(text.get(start..))
                .unwrap_or("\\")
                .to_owned(),
            Self::StreamingBytes => escape_fragment(input, start, end),
        }
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
fn decode_c_string_literal_byte(input: &[u8], index: usize) -> MiscCodecResult<(u8, usize)> {
    decode_c_string_literal_unit(input, index, CStringLiteralParseContext::StreamingBytes)
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
) -> MiscCodecResult<(u8, usize)> {
    let available = input.len().saturating_sub(index);
    if available == 0 {
        return Err(MiscCodecError::Incomplete {
            required: 1,
            available,
        });
    }
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
            if !context.is_complete_text() {
                ensure_fixed_escape_complete(available, 6)?;
            }
            parse_fixed_hex_escape_units(input, index, 4, context)
        }
        b'U' => {
            if !context.is_complete_text() {
                ensure_fixed_escape_complete(available, 10)?;
            }
            parse_fixed_hex_escape_units(input, index, 8, context)
        }
        b'0'..=b'7' => {
            ensure_octal_escape_complete(input, index, available)?;
            Ok(parse_octal_escape_units(input, index))
        }
        _ => Err(invalid_escape(
            index,
            &context.escape_fragment(input, index, index + 2),
            "unsupported escape sequence",
        )),
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
/// Returns [`MiscCodecError::Incomplete`] when more units are required.
#[inline]
fn ensure_variable_hex_escape_complete(
    _input: &[u8],
    _index: usize,
    available: usize,
) -> MiscCodecResult<()> {
    if available < 3 {
        return Err(MiscCodecError::Incomplete {
            required: 3,
            available,
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
/// Returns [`MiscCodecError::Incomplete`] when more units are required.
#[inline]
fn ensure_fixed_escape_complete(available: usize, required: usize) -> MiscCodecResult<()> {
    if available < required {
        return Err(MiscCodecError::Incomplete {
            required,
            available,
        });
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
/// Returns [`MiscCodecError::Incomplete`] when more units are required.
#[inline]
fn ensure_octal_escape_complete(
    _input: &[u8],
    _index: usize,
    _available: usize,
) -> MiscCodecResult<()> {
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
) -> MiscCodecResult<()> {
    if matches!(byte, b'\t' | b'\n' | 0x0b | 0x0c | b' '..=b'~') {
        return Ok(());
    }
    Err(MiscCodecError::InvalidCharacter {
        index,
        character: context.source_character(input, index),
        reason: context.raw_source_reason().to_owned(),
    })
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
) -> MiscCodecResult<(u8, usize)> {
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
        ));
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
) -> MiscCodecResult<(u8, usize)> {
    let mut value = 0u32;
    let mut index = marker_index + 2;
    for _ in 0..digits {
        let Some(_) = input.get(index) else {
            return Err(invalid_escape(
                marker_index,
                &context.escape_fragment(input, marker_index, input.len()),
                "incomplete universal character escape",
            ));
        };
        let character = context.source_character(input, index);
        let Some(digit) = hex_value(character) else {
            return Err(MiscCodecError::InvalidDigit {
                radix: 16,
                index,
                character,
            });
        };
        value = (value << 4) | u32::from(digit);
        index += 1;
    }
    if value > u32::from(u8::MAX) {
        return Err(invalid_escape(
            marker_index,
            &context.escape_fragment(input, marker_index, index),
            "universal character value must fit in one byte",
        ));
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
fn encoded_byte_len(byte: u8) -> core::num::NonZeroUsize {
    match byte {
        b'\'' | b'"' | b'?' | b'\\' | 0x07 | 0x08 | 0x0c | b'\n' | b'\r' | b'\t' | 0x0b => {
            qubit_codec::nz!(2)
        }
        b' '..=b'~' => core::num::NonZeroUsize::MIN,
        _ => qubit_codec::nz!(4),
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

/// Builds an invalid escape error.
///
/// # Parameters
/// - `index`: Byte index of the escape marker in the original input.
/// - `escape`: Escape fragment that caused the error.
/// - `reason`: Human-readable rejection reason.
///
/// # Returns
/// An invalid escape error.
fn invalid_escape(index: usize, escape: &str, reason: &str) -> MiscCodecError {
    MiscCodecError::InvalidEscape {
        index,
        escape: escape.to_owned(),
        reason: reason.to_owned(),
    }
}
