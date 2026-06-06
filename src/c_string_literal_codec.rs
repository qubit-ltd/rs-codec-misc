// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! C string literal byte codec.

use crate::{
    Codec,
    MiscCodecError,
    MiscCodecResult,
    ValueDecoder,
    ValueEncoder,
};

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
    pub fn decode(&self, text: &str) -> MiscCodecResult<Vec<u8>> {
        let chars = text.char_indices().collect::<Vec<_>>();
        let mut output = Vec::with_capacity(text.len());
        let mut position = 0;
        while let Some(&(index, character)) = chars.get(position) {
            if character == '\\' {
                decode_escape(text, &chars, &mut position, &mut output)?;
                continue;
            }
            validate_source_character(index, character)?;
            output.push(character as u8);
            position += 1;
        }
        Ok(output)
    }
}

impl ValueEncoder<[u8]> for CStringLiteralCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Encodes bytes into a C string literal fragment.
    fn encode(&self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(CStringLiteralCodec::encode(self, input))
    }
}

impl ValueDecoder<str> for CStringLiteralCodec {
    type Error = MiscCodecError;
    type Output = Vec<u8>;

    /// Decodes a C string literal fragment into bytes.
    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        CStringLiteralCodec::decode(self, input)
    }
}

unsafe impl Codec for CStringLiteralCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = MiscCodecError;
    type EncodeError = MiscCodecError;

    /// Returns the shortest representation length for one byte.
    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    /// Returns the longest supported universal escape length for one byte.
    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(10) }
    }

    /// Decodes one raw byte or one C escape fragment.
    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        let (value, consumed) = decode_c_string_literal_byte(input, index)?;
        debug_assert!(consumed > 0);
        // SAFETY: `decode_c_string_literal_byte` returns a non-zero width for
        // every successful raw byte or escape.
        let consumed =
            unsafe { core::num::NonZeroUsize::new_unchecked(consumed) };
        Ok((value, consumed))
    }

    /// Encodes one byte as a raw byte or C escape fragment.
    unsafe fn encode_unchecked(
        &self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::EncodeError> {
        let required = match *value {
            b'\'' | b'"' | b'?' | b'\\' | 0x07 | 0x08 | 0x0c | b'\n'
            | b'\r' | b'\t' | 0x0b => 2,
            b' '..=b'~' => 1,
            _ => 4,
        };
        debug_assert!(index + required <= output.len());

        Ok(write_encoded_byte(*value, output, index))
    }
}

/// Decodes one escape sequence at the current position.
///
/// # Parameters
/// - `text`: Original input text.
/// - `chars`: Indexed characters from `text`.
/// - `position`: Current character position, pointing at `\`.
/// - `output`: Destination byte buffer.
///
/// # Errors
/// Returns [`MiscCodecError`] when the escape marker is trailing or the escape
/// sequence is malformed.
fn decode_escape(
    text: &str,
    chars: &[(usize, char)],
    position: &mut usize,
    output: &mut Vec<u8>,
) -> MiscCodecResult<()> {
    let marker_index = chars[*position].0;
    *position += 1;
    let Some(&(_, escape)) = chars.get(*position) else {
        return Err(invalid_escape(
            marker_index,
            "\\",
            "incomplete escape sequence",
        ));
    };
    match escape {
        ' ' => push_simple_escape(position, output, b' '),
        '\'' => push_simple_escape(position, output, b'\''),
        '"' => push_simple_escape(position, output, b'"'),
        '?' => push_simple_escape(position, output, b'?'),
        '\\' => push_simple_escape(position, output, b'\\'),
        'a' => push_simple_escape(position, output, 0x07),
        'b' => push_simple_escape(position, output, 0x08),
        'f' => push_simple_escape(position, output, 0x0c),
        'n' => push_simple_escape(position, output, b'\n'),
        'r' => push_simple_escape(position, output, b'\r'),
        't' => push_simple_escape(position, output, b'\t'),
        'v' => push_simple_escape(position, output, 0x0b),
        'x' | 'X' => {
            *position += 1;
            let value =
                parse_variable_hex_escape(chars, position, marker_index)?;
            output.push(value);
        }
        'u' => {
            *position += 1;
            let value =
                parse_fixed_hex_escape(text, chars, position, marker_index, 4)?;
            output.push(value);
        }
        'U' => {
            *position += 1;
            let value =
                parse_fixed_hex_escape(text, chars, position, marker_index, 8)?;
            output.push(value);
        }
        '0'..='7' => {
            let value = parse_octal_escape(chars, position);
            output.push(value);
        }
        _ => {
            return Err(invalid_escape(
                marker_index,
                &format!("\\{escape}"),
                "unsupported escape sequence",
            ));
        }
    }
    Ok(())
}

/// Pushes a simple one-character escape result.
///
/// # Parameters
/// - `position`: Current character position, pointing at the escape character.
/// - `output`: Destination byte buffer.
/// - `byte`: Byte produced by the escape sequence.
fn push_simple_escape(position: &mut usize, output: &mut Vec<u8>, byte: u8) {
    output.push(byte);
    *position += 1;
}

/// Parses a variable-width hexadecimal byte escape.
///
/// # Parameters
/// - `chars`: Indexed characters from the original input.
/// - `position`: Current character position after `\x`.
/// - `marker_index`: Byte index of the escape marker.
///
/// # Returns
/// The decoded byte.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidEscape`] when no hexadecimal digit follows
/// `\x`.
fn parse_variable_hex_escape(
    chars: &[(usize, char)],
    position: &mut usize,
    marker_index: usize,
) -> MiscCodecResult<u8> {
    let mut value = 0u8;
    let mut digit_count = 0;
    while digit_count < 2 {
        let Some(&(_, character)) = chars.get(*position) else {
            break;
        };
        let Some(digit) = hex_value(character) else {
            break;
        };
        value = (value << 4) | digit;
        *position += 1;
        digit_count += 1;
    }
    if digit_count == 0 {
        return Err(invalid_escape(
            marker_index,
            "\\x",
            "expected at least one hexadecimal digit",
        ));
    }
    Ok(value)
}

/// Parses a fixed-width universal byte escape.
///
/// # Parameters
/// - `text`: Original input text.
/// - `chars`: Indexed characters from `text`.
/// - `position`: Current character position after `\u` or `\U`.
/// - `marker_index`: Byte index of the escape marker.
/// - `digits`: Required number of hexadecimal digits.
///
/// # Returns
/// The decoded byte.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidEscape`] when the escape is too short or
/// too large for one byte, or [`MiscCodecError::InvalidDigit`] when a required
/// digit is not hexadecimal.
fn parse_fixed_hex_escape(
    text: &str,
    chars: &[(usize, char)],
    position: &mut usize,
    marker_index: usize,
    digits: usize,
) -> MiscCodecResult<u8> {
    let mut value = 0u32;
    for _ in 0..digits {
        let Some(&(index, character)) = chars.get(*position) else {
            let escape = text.get(marker_index..).unwrap_or("\\");
            return Err(invalid_escape(
                marker_index,
                escape,
                "incomplete universal character escape",
            ));
        };
        let Some(digit) = hex_value(character) else {
            return Err(MiscCodecError::InvalidDigit {
                radix: 16,
                index,
                character,
            });
        };
        value = (value << 4) | u32::from(digit);
        *position += 1;
    }
    if value > u32::from(u8::MAX) {
        let escape = text
            .get(
                marker_index
                    ..chars[*position - 1].0
                        + chars[*position - 1].1.len_utf8(),
            )
            .unwrap_or("\\u");
        return Err(invalid_escape(
            marker_index,
            escape,
            "universal character value must fit in one byte",
        ));
    }
    Ok(value as u8)
}

/// Parses an octal byte escape.
///
/// # Parameters
/// - `chars`: Indexed characters from the original input.
/// - `position`: Current character position, pointing at the first octal digit.
///
/// # Returns
/// The decoded byte. Values above `0o377` are truncated to their low byte to
/// match byte-oriented C literal usage.
fn parse_octal_escape(chars: &[(usize, char)], position: &mut usize) -> u8 {
    let mut value = 0u16;
    let mut digit_count = 0;
    while digit_count < 3 {
        let Some(&(_, character)) = chars.get(*position) else {
            break;
        };
        let Some(digit) = octal_value(character) else {
            break;
        };
        value = (value << 3) | u16::from(digit);
        *position += 1;
        digit_count += 1;
    }
    value as u8
}

/// Validates a raw source character.
///
/// # Parameters
/// - `index`: Byte index of `character` in the original input.
/// - `character`: Raw, unescaped source character.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidCharacter`] when the raw character is not a
/// supported ASCII C string source character.
fn validate_source_character(
    index: usize,
    character: char,
) -> MiscCodecResult<()> {
    if is_source_character(character) {
        return Ok(());
    }
    Err(MiscCodecError::InvalidCharacter {
        index,
        character,
        reason:
            "raw source character must be printable ASCII or allowed whitespace"
                .to_owned(),
    })
}

/// Tests whether a raw character may appear unescaped.
///
/// # Parameters
/// - `character`: Character to inspect.
///
/// # Returns
/// `true` when `character` is accepted as a raw C string source character.
fn is_source_character(character: char) -> bool {
    matches!(character, '\t' | '\n' | '\u{0b}' | '\u{0c}' | ' '..='~')
}

/// Encodes one byte into the destination string.
///
/// # Parameters
/// - `byte`: Byte to encode.
/// - `output`: Destination string.
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
fn decode_c_string_literal_byte(
    input: &[u8],
    index: usize,
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
        validate_source_byte(index, byte)?;
        return Ok((byte, 1));
    }
    if available < 2 {
        return Err(MiscCodecError::Incomplete {
            required: 2,
            available,
        });
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
            ensure_variable_hex_escape_complete(input, index, available)?;
            parse_variable_hex_escape_units(input, index)
        }
        b'u' => {
            ensure_fixed_escape_complete(available, 6)?;
            parse_fixed_hex_escape_units(input, index, 4)
        }
        b'U' => {
            ensure_fixed_escape_complete(available, 10)?;
            parse_fixed_hex_escape_units(input, index, 8)
        }
        b'0'..=b'7' => {
            ensure_octal_escape_complete(input, index, available)?;
            Ok(parse_octal_escape_units(input, index))
        }
        _ => Err(invalid_escape(
            index,
            &escape_fragment(input, index, index + 2),
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
fn ensure_fixed_escape_complete(
    available: usize,
    required: usize,
) -> MiscCodecResult<()> {
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
fn ensure_octal_escape_complete(
    _input: &[u8],
    _index: usize,
    _available: usize,
) -> MiscCodecResult<()> {
    Ok(())
}

/// Validates a raw source byte.
///
/// # Parameters
/// - `index`: Byte index in the encoded input.
/// - `byte`: Raw source byte.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidCharacter`] when the byte is not allowed as
/// a raw C string source byte.
fn validate_source_byte(index: usize, byte: u8) -> MiscCodecResult<()> {
    if matches!(byte, b'\t' | b'\n' | 0x0b | 0x0c | b' '..=b'~') {
        return Ok(());
    }
    Err(MiscCodecError::InvalidCharacter {
        index,
        character: char::from(byte),
        reason: "raw source byte must be printable ASCII or allowed whitespace"
            .to_owned(),
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
fn parse_fixed_hex_escape_units(
    input: &[u8],
    marker_index: usize,
    digits: usize,
) -> MiscCodecResult<(u8, usize)> {
    let mut value = 0u32;
    let mut index = marker_index + 2;
    for _ in 0..digits {
        let Some(&byte) = input.get(index) else {
            return Err(invalid_escape(
                marker_index,
                &escape_fragment(input, marker_index, input.len()),
                "incomplete universal character escape",
            ));
        };
        let Some(digit) = hex_value(char::from(byte)) else {
            return Err(MiscCodecError::InvalidDigit {
                radix: 16,
                index,
                character: char::from(byte),
            });
        };
        value = (value << 4) | u32::from(digit);
        index += 1;
    }
    if value > u32::from(u8::MAX) {
        return Err(invalid_escape(
            marker_index,
            &escape_fragment(input, marker_index, index),
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

/// Encodes one byte into `output`.
///
/// # Parameters
/// - `byte`: Byte to encode.
/// - `output`: Destination unit buffer.
/// - `index`: Start index in `output`.
///
/// # Returns
/// Number of units written.
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
fn uppercase_hex_digit(value: u8) -> char {
    match value & 0x0f {
        0x0 => '0',
        0x1 => '1',
        0x2 => '2',
        0x3 => '3',
        0x4 => '4',
        0x5 => '5',
        0x6 => '6',
        0x7 => '7',
        0x8 => '8',
        0x9 => '9',
        0x0a => 'A',
        0x0b => 'B',
        0x0c => 'C',
        0x0d => 'D',
        0x0e => 'E',
        _ => 'F',
    }
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
