/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! C string literal byte codec.

use crate::{
    CodecError,
    CodecResult,
    Decoder,
    Encoder,
};

/// Encodes and decodes byte-oriented C string literal fragments.
///
/// This codec is intended for textual formats that embed byte sequences with C
/// escapes, such as `PK\003\004` or `\xd0\xcf`. It decodes into raw bytes and
/// does not require surrounding quotes.
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
    /// Returns [`CodecError::InvalidEscape`] for malformed escape sequences,
    /// [`CodecError::InvalidDigit`] for malformed fixed-width numeric escapes,
    /// and [`CodecError::InvalidCharacter`] for unsupported raw source characters.
    pub fn decode(&self, text: &str) -> CodecResult<Vec<u8>> {
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

impl Encoder<[u8]> for CStringLiteralCodec {
    type Error = CodecError;
    type Output = String;

    /// Encodes bytes into a C string literal fragment.
    fn encode(&self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(CStringLiteralCodec::encode(self, input))
    }
}

impl Decoder<str> for CStringLiteralCodec {
    type Error = CodecError;
    type Output = Vec<u8>;

    /// Decodes a C string literal fragment into bytes.
    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        CStringLiteralCodec::decode(self, input)
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
/// Returns [`CodecError`] when the escape marker is trailing or the escape
/// sequence is malformed.
fn decode_escape(text: &str, chars: &[(usize, char)], position: &mut usize, output: &mut Vec<u8>) -> CodecResult<()> {
    let marker_index = chars[*position].0;
    *position += 1;
    let Some(&(_, escape)) = chars.get(*position) else {
        return Err(invalid_escape(marker_index, "\\", "incomplete escape sequence"));
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
            let value = parse_variable_hex_escape(chars, position, marker_index)?;
            output.push(value);
        }
        'u' => {
            *position += 1;
            let value = parse_fixed_hex_escape(text, chars, position, marker_index, 4)?;
            output.push(value);
        }
        'U' => {
            *position += 1;
            let value = parse_fixed_hex_escape(text, chars, position, marker_index, 8)?;
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
/// Returns [`CodecError::InvalidEscape`] when no hexadecimal digit follows
/// `\x`.
fn parse_variable_hex_escape(chars: &[(usize, char)], position: &mut usize, marker_index: usize) -> CodecResult<u8> {
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
/// Returns [`CodecError::InvalidEscape`] when the escape is too short or too
/// large for one byte, or [`CodecError::InvalidDigit`] when a required digit is
/// not hexadecimal.
fn parse_fixed_hex_escape(
    text: &str,
    chars: &[(usize, char)],
    position: &mut usize,
    marker_index: usize,
    digits: usize,
) -> CodecResult<u8> {
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
            return Err(CodecError::InvalidDigit {
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
            .get(marker_index..chars[*position - 1].0 + chars[*position - 1].1.len_utf8())
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
/// Returns [`CodecError::InvalidCharacter`] when the raw character is not a
/// supported ASCII C string source character.
fn validate_source_character(index: usize, character: char) -> CodecResult<()> {
    if is_source_character(character) {
        return Ok(());
    }
    Err(CodecError::InvalidCharacter {
        index,
        character,
        reason: "raw source character must be printable ASCII or allowed whitespace".to_owned(),
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
fn invalid_escape(index: usize, escape: &str, reason: &str) -> CodecError {
    CodecError::InvalidEscape {
        index,
        escape: escape.to_owned(),
        reason: reason.to_owned(),
    }
}
