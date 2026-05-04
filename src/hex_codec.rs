/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Hexadecimal byte codec.

use crate::{
    CodecError,
    CodecResult,
    Decoder,
    Encoder,
};

/// Encodes and decodes hexadecimal byte strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexCodec {
    /// Whether to use uppercase hexadecimal digits.
    uppercase: bool,
    /// The prefix to use before each encoded byte.
    prefix: Option<String>,
    /// The separator to use between bytes in the encoded string.
    separator: Option<String>,
    /// Whether to ignore ASCII whitespace while decoding.
    ignore_ascii_whitespace: bool,
}

impl HexCodec {
    /// Creates a lowercase codec without prefix or separators.
    ///
    /// # Returns
    /// A hexadecimal codec using lowercase digits.
    pub fn new() -> Self {
        Self {
            uppercase: false,
            prefix: None,
            separator: None,
            ignore_ascii_whitespace: false,
        }
    }

    /// Creates an uppercase codec without prefix or separators.
    ///
    /// # Returns
    /// A hexadecimal codec using uppercase digits.
    pub fn upper() -> Self {
        Self::new().with_uppercase(true)
    }

    /// Sets whether encoded digits should be uppercase.
    ///
    /// # Parameters
    /// - `uppercase`: Whether to use uppercase hexadecimal digits.
    ///
    /// # Returns
    /// The updated codec.
    pub fn with_uppercase(mut self, uppercase: bool) -> Self {
        self.uppercase = uppercase;
        self
    }

    /// Sets a per-byte prefix.
    ///
    /// The prefix is written before every encoded byte and required before
    /// every decoded byte. For example, using prefix `0x` and separator ` `
    /// encodes bytes as `0x1F 0x8B`.
    ///
    /// This is not a whole-output prefix: `[0x1F, 0x8B]` is encoded as
    /// `0x1F 0x8B`, not `0x1F 8B`.
    ///
    /// # Parameters
    /// - `prefix`: Prefix text such as `0x`.
    ///
    /// # Returns
    /// The updated codec.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Sets a separator written and accepted between encoded bytes.
    ///
    /// # Parameters
    /// - `separator`: Separator text.
    ///
    /// # Returns
    /// The updated codec.
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = Some(separator.into());
        self
    }

    /// Sets whether ASCII whitespace is ignored while decoding.
    ///
    /// # Parameters
    /// - `ignore`: Whether to ignore ASCII whitespace.
    ///
    /// # Returns
    /// The updated codec.
    pub fn with_ignored_ascii_whitespace(mut self, ignore: bool) -> Self {
        self.ignore_ascii_whitespace = ignore;
        self
    }

    /// Encodes bytes into a hexadecimal string.
    ///
    /// # Parameters
    /// - `bytes`: Bytes to encode.
    ///
    /// # Returns
    /// Hexadecimal text.
    pub fn encode(&self, bytes: &[u8]) -> String {
        let separator_len = self.separator.as_ref().map_or(0, String::len);
        let prefix_len = self.prefix.as_ref().map_or(0, String::len);
        let capacity = bytes
            .len()
            .saturating_mul(prefix_len.saturating_add(2))
            .saturating_add(bytes.len().saturating_sub(1).saturating_mul(separator_len));
        let mut output = String::with_capacity(capacity);
        self.encode_into(bytes, &mut output);
        output
    }

    /// Encodes bytes into an existing string.
    ///
    /// # Parameters
    /// - `bytes`: Bytes to encode.
    /// - `output`: Destination string.
    pub fn encode_into(&self, bytes: &[u8], output: &mut String) {
        for (index, byte) in bytes.iter().enumerate() {
            if index > 0
                && let Some(separator) = &self.separator
            {
                output.push_str(separator);
            }
            if let Some(prefix) = &self.prefix {
                output.push_str(prefix);
            }
            push_hex_byte(*byte, self.uppercase, output);
        }
    }

    /// Decodes hexadecimal text into bytes.
    ///
    /// # Parameters
    /// - `text`: Hexadecimal text.
    ///
    /// # Returns
    /// Decoded bytes.
    ///
    /// # Errors
    /// Returns [`CodecError`] when a configured per-byte prefix is missing,
    /// when the normalized digit count is odd, or when a non-hex digit is found.
    pub fn decode(&self, text: &str) -> CodecResult<Vec<u8>> {
        let mut output = Vec::new();
        self.decode_into(text, &mut output)?;
        Ok(output)
    }

    /// Decodes hexadecimal text into an existing byte vector.
    ///
    /// # Parameters
    /// - `text`: Hexadecimal text.
    /// - `output`: Destination byte vector.
    ///
    /// # Errors
    /// Returns [`CodecError`] when the input is malformed.
    pub fn decode_into(&self, text: &str, output: &mut Vec<u8>) -> CodecResult<()> {
        let digits = self.normalized_digits(text)?;
        if digits.len() % 2 != 0 {
            return Err(CodecError::OddHexLength {
                digits: digits.len(),
            });
        }
        output.reserve(digits.len() / 2);
        for pair in digits.chunks_exact(2) {
            let mut pair = pair.iter();
            let Some(&(high_index, high_char)) = pair.next() else {
                continue;
            };
            let Some(&(low_index, low_char)) = pair.next() else {
                continue;
            };
            let high = hex_value(high_char).ok_or(CodecError::InvalidHexDigit {
                index: high_index,
                character: high_char,
            })?;
            let low = hex_value(low_char).ok_or(CodecError::InvalidHexDigit {
                index: low_index,
                character: low_char,
            })?;
            output.push((high << 4) | low);
        }
        Ok(())
    }

    /// Normalizes accepted input characters into hex digits.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    ///
    /// # Returns
    /// Hex digits paired with their original character indexes.
    ///
    /// # Errors
    /// Returns [`CodecError::InvalidHexDigit`] for unsupported characters.
    fn normalized_digits(&self, text: &str) -> CodecResult<Vec<(usize, char)>> {
        if let Some(prefix) = self.prefix.as_deref().filter(|prefix| !prefix.is_empty()) {
            return self.normalized_prefixed_digits(text, prefix);
        }
        self.normalized_unprefixed_digits(text)
    }

    /// Normalizes unprefixed input characters into hex digits.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    ///
    /// # Returns
    /// Hex digits paired with their original character indexes.
    ///
    /// # Errors
    /// Returns [`CodecError::InvalidHexDigit`] for unsupported characters.
    fn normalized_unprefixed_digits(&self, text: &str) -> CodecResult<Vec<(usize, char)>> {
        let mut digits = Vec::with_capacity(text.len());
        let separator = self
            .separator
            .as_deref()
            .filter(|separator| !separator.is_empty());
        let mut index = 0;
        while index < text.len() {
            let Some(rest) = text.get(index..) else {
                break;
            };
            if let Some(separator) = separator
                && rest.starts_with(separator)
            {
                index += separator.len();
                continue;
            }
            let Some(ch) = rest.chars().next() else {
                break;
            };
            if self.ignore_ascii_whitespace && ch.is_ascii_whitespace() {
                index += ch.len_utf8();
                continue;
            }
            if hex_value(ch).is_some() {
                digits.push((index, ch));
                index += ch.len_utf8();
                continue;
            }
            return Err(CodecError::InvalidHexDigit {
                index,
                character: ch,
            });
        }
        Ok(digits)
    }

    /// Normalizes prefixed input characters into hex digits.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    /// - `prefix`: Required prefix before each byte.
    ///
    /// # Returns
    /// Hex digits paired with their original character indexes.
    ///
    /// # Errors
    /// Returns [`CodecError::MissingPrefix`] when a byte prefix is missing, or
    /// [`CodecError::InvalidHexDigit`] for unsupported characters.
    fn normalized_prefixed_digits(
        &self,
        text: &str,
        prefix: &str,
    ) -> CodecResult<Vec<(usize, char)>> {
        let mut digits = Vec::with_capacity(text.len());
        let separator = self
            .separator
            .as_deref()
            .filter(|separator| !separator.is_empty());
        let mut index = 0;
        while index < text.len() {
            index = self.skip_ignored(text, index, separator);
            if index >= text.len() {
                break;
            }
            let Some(rest) = text.get(index..) else {
                break;
            };
            if !rest.starts_with(prefix) {
                return Err(CodecError::MissingPrefix {
                    prefix: prefix.to_owned(),
                });
            }
            index += prefix.len();

            let mut digit_count = 0;
            while digit_count < 2 && index < text.len() {
                let Some(rest) = text.get(index..) else {
                    break;
                };
                let Some(ch) = rest.chars().next() else {
                    break;
                };
                if self.ignore_ascii_whitespace && ch.is_ascii_whitespace() {
                    index += ch.len_utf8();
                    continue;
                }
                if hex_value(ch).is_some() {
                    digits.push((index, ch));
                    index += ch.len_utf8();
                    digit_count += 1;
                    continue;
                }
                return Err(CodecError::InvalidHexDigit {
                    index,
                    character: ch,
                });
            }
        }
        Ok(digits)
    }

    /// Skips configured separators and ignored ASCII whitespace.
    ///
    /// # Parameters
    /// - `text`: Text being decoded.
    /// - `index`: Current byte index.
    /// - `separator`: Optional configured separator.
    ///
    /// # Returns
    /// The next byte index that should be parsed.
    fn skip_ignored(&self, text: &str, mut index: usize, separator: Option<&str>) -> usize {
        loop {
            let Some(rest) = text.get(index..) else {
                return index;
            };
            if let Some(separator) = separator
                && rest.starts_with(separator)
            {
                index += separator.len();
                continue;
            }
            let Some(ch) = rest.chars().next() else {
                return index;
            };
            if self.ignore_ascii_whitespace && ch.is_ascii_whitespace() {
                index += ch.len_utf8();
                continue;
            }
            return index;
        }
    }
}

impl Default for HexCodec {
    /// Creates a lowercase codec without prefix or separators.
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder<[u8]> for HexCodec {
    type Error = CodecError;
    type Output = String;

    /// Encodes bytes into hexadecimal text.
    fn encode(&self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(HexCodec::encode(self, input))
    }
}

impl Decoder<str> for HexCodec {
    type Error = CodecError;
    type Output = Vec<u8>;

    /// Decodes hexadecimal text into bytes.
    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        HexCodec::decode(self, input)
    }
}

/// Converts one hex digit to its value.
///
/// # Parameters
/// - `ch`: Character to inspect.
///
/// # Returns
/// Nibble value, or `None` when `ch` is not a hex digit.
fn hex_value(ch: char) -> Option<u8> {
    match ch {
        '0'..='9' => Some(ch as u8 - b'0'),
        'a'..='f' => Some(ch as u8 - b'a' + 10),
        'A'..='F' => Some(ch as u8 - b'A' + 10),
        _ => None,
    }
}

/// Appends one encoded byte to `output`.
///
/// # Parameters
/// - `byte`: Byte to encode.
/// - `uppercase`: Whether to use uppercase digits.
/// - `output`: Destination string.
fn push_hex_byte(byte: u8, uppercase: bool, output: &mut String) {
    output.push(hex_digit(byte >> 4, uppercase));
    output.push(hex_digit(byte & 0x0f, uppercase));
}

/// Converts one nibble to a hexadecimal digit.
///
/// # Parameters
/// - `value`: Nibble value.
/// - `uppercase`: Whether to use uppercase digits.
///
/// # Returns
/// Hexadecimal digit. Values above `0x0f` are masked to their low nibble.
fn hex_digit(value: u8, uppercase: bool) -> char {
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
        0x0a if uppercase => 'A',
        0x0b if uppercase => 'B',
        0x0c if uppercase => 'C',
        0x0d if uppercase => 'D',
        0x0e if uppercase => 'E',
        0x0f if uppercase => 'F',
        0x0a => 'a',
        0x0b => 'b',
        0x0c => 'c',
        0x0d => 'd',
        0x0e => 'e',
        _ => 'f',
    }
}
