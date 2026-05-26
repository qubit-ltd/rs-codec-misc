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
    Codec,
    Decoder,
    Encoder,
    MiscCodecError,
    MiscCodecResult,
};

/// Encodes and decodes hexadecimal byte strings.
///
/// Its low-level [`Codec<u8, u8>`] implementation handles exactly one byte as
/// two ASCII hexadecimal units. Whole-string prefix, per-byte prefix,
/// separator, and whitespace handling remain part of the owned
/// [`encode`](Self::encode) and [`decode`](Self::decode) helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexCodec {
    /// Whether to use uppercase hexadecimal digits.
    uppercase: bool,
    /// The prefix to use before the whole encoded string.
    prefix: Option<String>,
    /// The prefix to use before each encoded byte.
    byte_prefix: Option<String>,
    /// The separator to use between bytes in the encoded string.
    separator: Option<String>,
    /// Whether to ignore ASCII whitespace while decoding.
    ignore_ascii_whitespace: bool,
    /// Whether to ignore ASCII case when matching configured prefixes.
    ignore_prefix_case: bool,
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
            byte_prefix: None,
            separator: None,
            ignore_ascii_whitespace: false,
            ignore_prefix_case: false,
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

    /// Sets a whole-output prefix.
    ///
    /// The prefix is written once before the encoded bytes and required once
    /// before decoded input. For example, using prefix `0x` encodes bytes as
    /// `0x1f8b`.
    ///
    /// # Parameters
    /// - `prefix`: Whole-output prefix text such as `0x`.
    ///
    /// # Returns
    /// The updated codec.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Sets a per-byte prefix.
    ///
    /// The prefix is written before every encoded byte and required before
    /// every decoded byte. For example, using byte prefix `0x` and separator
    /// ` ` encodes bytes as `0x1f 0x8b`.
    ///
    /// # Parameters
    /// - `prefix`: Per-byte prefix text such as `0x`.
    ///
    /// # Returns
    /// The updated codec.
    pub fn with_byte_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.byte_prefix = Some(prefix.into());
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

    /// Sets whether ASCII case is ignored when decoding configured prefixes.
    ///
    /// This option affects whole-output prefixes and per-byte prefixes during
    /// decoding only. Encoding writes prefixes exactly as configured.
    ///
    /// # Parameters
    /// - `ignore`: Whether to ignore ASCII case while matching prefixes.
    ///
    /// # Returns
    /// The updated codec.
    pub fn with_ignore_prefix_case(mut self, ignore: bool) -> Self {
        self.ignore_prefix_case = ignore;
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
        let byte_prefix_len = self.byte_prefix.as_ref().map_or(0, String::len);
        let capacity = prefix_len.saturating_add(
            bytes
                .len()
                .saturating_mul(byte_prefix_len.saturating_add(2))
                .saturating_add(bytes.len().saturating_sub(1).saturating_mul(separator_len)),
        );
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
        if let Some(prefix) = &self.prefix {
            output.push_str(prefix);
        }
        for (index, byte) in bytes.iter().enumerate() {
            if index > 0
                && let Some(separator) = &self.separator
            {
                output.push_str(separator);
            }
            if let Some(byte_prefix) = &self.byte_prefix {
                output.push_str(byte_prefix);
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
    /// Returns [`MiscCodecError`] when a configured whole or per-byte prefix is missing,
    /// when the normalized digit count is odd, or when a non-hex digit is found.
    pub fn decode(&self, text: &str) -> MiscCodecResult<Vec<u8>> {
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
    /// Returns [`MiscCodecError`] when the input is malformed.
    pub fn decode_into(&self, text: &str, output: &mut Vec<u8>) -> MiscCodecResult<()> {
        let digits = self.normalized_digits(text)?;
        if digits.len() % 2 != 0 {
            return Err(invalid_hex_length(digits.len()));
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
            let high = hex_value(high_char).ok_or(invalid_hex_digit(high_index, high_char))?;
            let low = hex_value(low_char).ok_or(invalid_hex_digit(low_index, low_char))?;
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
    /// Returns [`MiscCodecError::InvalidDigit`] for unsupported characters.
    fn normalized_digits(&self, text: &str) -> MiscCodecResult<Vec<(usize, char)>> {
        let start_index = self.consume_prefix(text)?;
        if let Some(byte_prefix) = self.byte_prefix.as_deref().filter(|prefix| !prefix.is_empty()) {
            return self.normalized_byte_prefixed_digits(text, byte_prefix, start_index);
        }
        self.normalized_unprefixed_digits(text, start_index)
    }

    /// Consumes the configured whole-output prefix.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    ///
    /// # Returns
    /// Byte index where byte parsing should start.
    ///
    /// # Errors
    /// Returns [`MiscCodecError::MissingPrefix`] when a non-empty whole-output
    /// prefix is configured but absent.
    fn consume_prefix(&self, text: &str) -> MiscCodecResult<usize> {
        let Some(prefix) = self.prefix.as_deref().filter(|prefix| !prefix.is_empty()) else {
            return Ok(0);
        };
        let index = self.skip_ascii_whitespace(text, 0);
        let Some(rest) = text.get(index..) else {
            return Err(MiscCodecError::MissingPrefix {
                prefix: prefix.to_owned(),
            });
        };
        if self.starts_with_prefix(rest, prefix) {
            Ok(index + prefix.len())
        } else {
            Err(MiscCodecError::MissingPrefix {
                prefix: prefix.to_owned(),
            })
        }
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
    /// Returns [`MiscCodecError::InvalidDigit`] for unsupported characters.
    fn normalized_unprefixed_digits(&self, text: &str, mut index: usize) -> MiscCodecResult<Vec<(usize, char)>> {
        let mut digits = Vec::with_capacity(text.len());
        let separator = self.separator.as_deref().filter(|separator| !separator.is_empty());
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
            return Err(invalid_hex_digit(index, ch));
        }
        Ok(digits)
    }

    /// Normalizes byte-prefixed input characters into hex digits.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    /// - `prefix`: Required prefix before each byte.
    /// - `index`: Byte index where parsing should start.
    ///
    /// # Returns
    /// Hex digits paired with their original character indexes.
    ///
    /// # Errors
    /// Returns [`MiscCodecError::MissingPrefix`] when a byte prefix is missing, or
    /// [`MiscCodecError::InvalidDigit`] for unsupported characters.
    fn normalized_byte_prefixed_digits(
        &self,
        text: &str,
        prefix: &str,
        mut index: usize,
    ) -> MiscCodecResult<Vec<(usize, char)>> {
        let mut digits = Vec::with_capacity(text.len());
        let separator = self.separator.as_deref().filter(|separator| !separator.is_empty());
        while index < text.len() {
            index = self.skip_ignored(text, index, separator);
            if index >= text.len() {
                break;
            }
            let Some(rest) = text.get(index..) else {
                break;
            };
            if !self.starts_with_prefix(rest, prefix) {
                return Err(MiscCodecError::MissingPrefix {
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
                return Err(invalid_hex_digit(index, ch));
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

    /// Skips ignored leading ASCII whitespace.
    ///
    /// # Parameters
    /// - `text`: Text being decoded.
    /// - `index`: Current byte index.
    ///
    /// # Returns
    /// The next byte index after ignored ASCII whitespace.
    fn skip_ascii_whitespace(&self, text: &str, mut index: usize) -> usize {
        while self.ignore_ascii_whitespace && index < text.len() {
            let Some(rest) = text.get(index..) else {
                return index;
            };
            let Some(ch) = rest.chars().next() else {
                return index;
            };
            if !ch.is_ascii_whitespace() {
                return index;
            }
            index += ch.len_utf8();
        }
        index
    }

    /// Tests whether `text` starts with a configured prefix.
    ///
    /// # Parameters
    /// - `text`: Text slice to inspect.
    /// - `prefix`: Configured prefix.
    ///
    /// # Returns
    /// `true` when `text` starts with `prefix`, honoring the configured
    /// ASCII case sensitivity for decoding prefixes.
    fn starts_with_prefix(&self, text: &str, prefix: &str) -> bool {
        if !self.ignore_prefix_case {
            return text.starts_with(prefix);
        }
        let Some(candidate) = text.get(..prefix.len()) else {
            return false;
        };
        candidate.eq_ignore_ascii_case(prefix)
    }
}

impl Default for HexCodec {
    /// Creates a lowercase codec without prefix or separators.
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder<[u8]> for HexCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Encodes bytes into hexadecimal text.
    fn encode(&self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(HexCodec::encode(self, input))
    }
}

impl Decoder<str> for HexCodec {
    type Error = MiscCodecError;
    type Output = Vec<u8>;

    /// Decodes hexadecimal text into bytes.
    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        HexCodec::decode(self, input)
    }
}

unsafe impl Codec<u8, u8> for HexCodec {
    type DecodeError = MiscCodecError;
    type EncodeError = MiscCodecError;

    /// Returns the two hexadecimal digits needed for one byte.
    fn min_units_per_value(&self) -> usize {
        2
    }

    /// Returns the two hexadecimal digits needed for one byte.
    fn max_units_per_value(&self) -> usize {
        2
    }

    /// Decodes one byte from two ASCII hexadecimal digits.
    unsafe fn decode_unchecked(&self, input: &[u8], index: usize) -> Result<(u8, usize), Self::DecodeError> {
        debug_assert!(index + 2 <= input.len());

        let high_char = char::from(input[index]);
        let low_char = char::from(input[index + 1]);
        let high = hex_value(high_char).ok_or_else(|| invalid_hex_digit(index, high_char))?;
        let low = hex_value(low_char).ok_or_else(|| invalid_hex_digit(index + 1, low_char))?;
        Ok(((high << 4) | low, 2))
    }

    /// Encodes one byte as two ASCII hexadecimal digits.
    unsafe fn encode_unchecked(&self, value: u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index + 2 <= output.len());

        output[index] = hex_digit(value >> 4, self.uppercase) as u8;
        output[index + 1] = hex_digit(value & 0x0f, self.uppercase) as u8;
        Ok(2)
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

/// Builds an invalid hexadecimal digit error.
///
/// # Parameters
/// - `index`: Byte index of the invalid character in the original input.
/// - `character`: Invalid character.
///
/// # Returns
/// A radix-16 digit error.
fn invalid_hex_digit(index: usize, character: char) -> MiscCodecError {
    MiscCodecError::InvalidDigit {
        radix: 16,
        index,
        character,
    }
}

/// Builds an invalid hexadecimal length error.
///
/// # Parameters
/// - `actual`: Number of normalized hexadecimal digits.
///
/// # Returns
/// An invalid length error describing the even-digit requirement.
fn invalid_hex_length(actual: usize) -> MiscCodecError {
    MiscCodecError::InvalidLength {
        context: "hex digits",
        expected: "an even number of digits".to_owned(),
        actual,
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
