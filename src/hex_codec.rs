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

const LOWER_DIGITS: &[u8; 16] = b"0123456789abcdef";
const UPPER_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Encodes and decodes hexadecimal byte strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexCodec {
    uppercase: bool,
    prefix: Option<String>,
    separator: Option<String>,
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

    /// Sets a whole-string prefix.
    ///
    /// The prefix is written once before encoded digits and required once
    /// before decoded input.
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
        let capacity = prefix_len + bytes.len() * 2 + bytes.len().saturating_sub(1) * separator_len;
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
        let digits = if self.uppercase {
            UPPER_DIGITS
        } else {
            LOWER_DIGITS
        };
        for (index, byte) in bytes.iter().enumerate() {
            if index > 0
                && let Some(separator) = &self.separator
            {
                output.push_str(separator);
            }
            output.push(digits[(byte >> 4) as usize] as char);
            output.push(digits[(byte & 0x0f) as usize] as char);
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
    /// Returns [`CodecError`] when the prefix is missing, when the normalized
    /// digit count is odd, or when a non-hex digit is found.
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
        let mut rest = text;
        if let Some(prefix) = &self.prefix {
            rest = rest
                .strip_prefix(prefix)
                .ok_or_else(|| CodecError::MissingPrefix {
                    prefix: prefix.clone(),
                })?;
        }
        let digits = self.normalized_digits(rest)?;
        if digits.len() % 2 != 0 {
            return Err(CodecError::OddHexLength {
                digits: digits.len(),
            });
        }
        output.reserve(digits.len() / 2);
        for pair in digits.chunks_exact(2) {
            let high = hex_value(pair[0].1).expect("normalized hex digit should be valid");
            let low = hex_value(pair[1].1).expect("normalized hex digit should be valid");
            output.push((high << 4) | low);
        }
        Ok(())
    }

    /// Normalizes accepted input characters into hex digits.
    ///
    /// # Parameters
    /// - `text`: Text after optional prefix removal.
    ///
    /// # Returns
    /// Hex digits paired with their original character indexes.
    ///
    /// # Errors
    /// Returns [`CodecError::InvalidHexDigit`] for unsupported characters.
    fn normalized_digits(&self, text: &str) -> CodecResult<Vec<(usize, char)>> {
        let mut digits = Vec::with_capacity(text.len());
        let separator = self
            .separator
            .as_deref()
            .filter(|separator| !separator.is_empty());
        let mut index = 0;
        while index < text.len() {
            let rest = &text[index..];
            if let Some(separator) = separator
                && rest.starts_with(separator)
            {
                index += separator.len();
                continue;
            }
            let ch = rest
                .chars()
                .next()
                .expect("index should point to a character boundary");
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
