/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Percent text codec.

use crate::{
    CodecError,
    CodecResult,
    Decoder,
    Encoder,
};

/// Encodes and decodes percent-encoded UTF-8 text.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PercentCodec;

impl PercentCodec {
    /// Creates a percent codec.
    ///
    /// # Returns
    /// Percent codec.
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
    pub fn encode(&self, text: &str) -> String {
        percent_encode_bytes(text.as_bytes(), false)
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
    /// Returns [`CodecError`] when a percent escape is malformed or decoded
    /// bytes are not valid UTF-8.
    pub fn decode(&self, text: &str) -> CodecResult<String> {
        String::from_utf8(percent_decode_bytes(text, false)?).map_err(CodecError::from)
    }
}

impl Encoder<str> for PercentCodec {
    type Error = CodecError;
    type Output = String;

    /// Encodes text using percent encoding.
    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(PercentCodec::encode(self, input))
    }
}

impl Decoder<str> for PercentCodec {
    type Error = CodecError;
    type Output = String;

    /// Decodes percent-encoded text.
    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        PercentCodec::decode(self, input)
    }
}

/// Percent-encodes UTF-8 bytes.
///
/// # Parameters
/// - `bytes`: Bytes to encode.
/// - `space_as_plus`: Whether spaces should be encoded as `+`.
///
/// # Returns
/// Encoded text.
pub(crate) fn percent_encode_bytes(bytes: &[u8], space_as_plus: bool) -> String {
    let mut output = String::with_capacity(bytes.len());
    for byte in bytes {
        if *byte == b' ' && space_as_plus {
            output.push('+');
        } else if is_unreserved(*byte) {
            output.push(*byte as char);
        } else {
            output.push('%');
            output.push(percent_hex_digit(byte >> 4));
            output.push(percent_hex_digit(byte & 0x0f));
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
/// Returns [`CodecError::InvalidEscape`] for malformed escapes.
pub(crate) fn percent_decode_bytes(text: &str, plus_as_space: bool) -> CodecResult<Vec<u8>> {
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'%' => {
                let (Some(&high_byte), Some(&low_byte)) = (bytes.get(index + 1), bytes.get(index + 2)) else {
                    return Err(invalid_percent_escape(index));
                };
                let high = percent_hex_value(high_byte).ok_or_else(|| invalid_percent_escape(index))?;
                let low = percent_hex_value(low_byte).ok_or_else(|| invalid_percent_escape(index))?;
                output.push((high << 4) | low);
                index += 3;
            }
            b'+' if plus_as_space => {
                output.push(b' ');
                index += 1;
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    Ok(output)
}

/// Builds a malformed percent escape error.
///
/// # Parameters
/// - `index`: Byte index of the `%` marker in the original input.
///
/// # Returns
/// An invalid escape error for a `%XX` sequence.
fn invalid_percent_escape(index: usize) -> CodecError {
    CodecError::InvalidEscape {
        index,
        escape: "%".to_owned(),
        reason: "expected two hexadecimal digits".to_owned(),
    }
}

/// Tests whether a byte may be left unescaped.
///
/// # Parameters
/// - `byte`: Byte to inspect.
///
/// # Returns
/// `true` for RFC 3986 unreserved bytes.
fn is_unreserved(byte: u8) -> bool {
    matches!(
        byte,
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~'
    )
}

/// Converts one ASCII hex byte to its nibble value.
///
/// # Parameters
/// - `byte`: ASCII byte to inspect.
///
/// # Returns
/// Nibble value, or `None` when `byte` is not hex.
fn percent_hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Converts one nibble to an uppercase hexadecimal digit.
///
/// # Parameters
/// - `value`: Nibble value.
///
/// # Returns
/// Uppercase hexadecimal digit. Values above `0x0f` are masked to their low nibble.
fn percent_hex_digit(value: u8) -> char {
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
