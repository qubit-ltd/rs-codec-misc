// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Hexadecimal byte codec.

use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;

use crate::MiscCodecError;
use crate::MiscCodecResult;
use crate::hex_codec_config::HexCodecConfig;

const LOWER_HEX_DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e',
    'f',
];

const UPPER_HEX_DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E',
    'F',
];

/// Encodes and decodes hexadecimal byte strings.
///
/// `HexCodec` is an owned facade for whole byte slices. Whole-string prefix,
/// per-byte prefix, separator, and whitespace handling are part of
/// [`encode`](Self::encode) and [`decode`](Self::decode). Use
/// [`crate::HexByteCodec`] when a low-level
/// [`qubit_codec::Codec<Value = u8, Unit = u8>`] is
/// required.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexCodec {
    config: HexCodecConfig,
}

impl HexCodec {
    /// Creates a lowercase codec without prefix or separators.
    ///
    /// # Returns
    /// A hexadecimal codec using lowercase digits.
    #[inline]
    pub fn new() -> Self {
        Self::from_config(HexCodecConfig::new())
    }

    /// Creates an uppercase codec without prefix or separators.
    ///
    /// # Returns
    /// A hexadecimal codec using uppercase digits.
    #[inline]
    pub fn upper() -> Self {
        Self::from_config(HexCodecConfig::new().with_uppercase(true))
    }

    /// Creates a codec from an explicit configuration.
    #[inline]
    pub fn from_config(config: HexCodecConfig) -> Self {
        Self { config }
    }

    /// Returns the configuration used by this codec.
    #[inline]
    pub fn config(&self) -> &HexCodecConfig {
        &self.config
    }

    /// Sets whether encoded digits should be uppercase.
    ///
    /// # Parameters
    /// - `uppercase`: Whether to use uppercase hexadecimal digits.
    ///
    /// # Returns
    /// The updated codec.
    #[inline]
    pub fn with_uppercase(mut self, uppercase: bool) -> Self {
        self.config = self.config.with_uppercase(uppercase);
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
    #[inline]
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config = self.config.with_prefix(prefix);
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
    #[inline]
    pub fn with_byte_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config = self.config.with_byte_prefix(prefix);
        self
    }

    /// Sets a separator written and accepted between encoded bytes.
    ///
    /// # Parameters
    /// - `separator`: Separator text.
    ///
    /// # Returns
    /// The updated codec.
    #[inline]
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.config = self.config.with_separator(separator);
        self
    }

    /// Sets whether ASCII whitespace is ignored while decoding.
    ///
    /// # Parameters
    /// - `ignore`: Whether to ignore ASCII whitespace.
    ///
    /// # Returns
    /// The updated codec.
    #[inline]
    pub fn with_ignored_ascii_whitespace(mut self, ignore: bool) -> Self {
        self.config = self.config.with_ignored_ascii_whitespace(ignore);
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
    #[inline]
    pub fn with_ignore_prefix_case(mut self, ignore: bool) -> Self {
        self.config = self.config.with_ignore_prefix_case(ignore);
        self
    }

    /// Encodes bytes into a hexadecimal string.
    ///
    /// # Parameters
    /// - `bytes`: Bytes to encode.
    ///
    /// # Returns
    /// Hexadecimal text.
    #[inline]
    #[must_use]
    pub fn encode(&self, bytes: &[u8]) -> String {
        let separator_len =
            self.config.separator.as_ref().map_or(0, String::len);
        let prefix_len = self.config.prefix.as_ref().map_or(0, String::len);
        let byte_prefix_len =
            self.config.byte_prefix.as_ref().map_or(0, String::len);
        let capacity = prefix_len.saturating_add(
            bytes
                .len()
                .saturating_mul(byte_prefix_len.saturating_add(2))
                .saturating_add(
                    bytes.len().saturating_sub(1).saturating_mul(separator_len),
                ),
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
    #[inline]
    pub fn encode_into(&self, bytes: &[u8], output: &mut String) {
        if let Some(prefix) = &self.config.prefix {
            output.push_str(prefix);
        }
        for (index, byte) in bytes.iter().enumerate() {
            if index > 0
                && let Some(separator) = &self.config.separator
            {
                output.push_str(separator);
            }
            if let Some(byte_prefix) = &self.config.byte_prefix {
                output.push_str(byte_prefix);
            }
            push_hex_byte(*byte, self.config.uppercase, output);
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
    /// Returns [`MiscCodecError`] when a configured whole or per-byte prefix is
    /// missing, when the normalized digit count is odd, or when a non-hex
    /// digit is found.
    #[inline]
    pub fn decode(&self, text: &str) -> MiscCodecResult<Vec<u8>> {
        self.decode_to_vec(text)
    }

    /// Decodes hexadecimal text into an existing byte vector.
    ///
    /// # Parameters
    /// - `text`: Hexadecimal text.
    /// - `output`: Destination byte vector.
    ///
    /// # Errors
    /// Returns [`MiscCodecError`] when the input is malformed.
    ///
    /// # TODO
    /// This method currently decodes into a temporary vector before appending
    /// so that an error leaves `output` unchanged. A future optimization can
    /// decode directly into `output` with a checkpoint-and-rollback strategy,
    /// or validate and reserve in a first pass, to avoid the temporary vector
    /// while preserving transactional behavior.
    #[inline]
    pub fn decode_into(
        &self,
        text: &str,
        output: &mut Vec<u8>,
    ) -> MiscCodecResult<()> {
        let decoded = self.decode_to_vec(text)?;
        output.reserve(decoded.len());
        output.extend(decoded);
        Ok(())
    }

    /// Decodes text into a new byte vector using a single transactional pass.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    ///
    /// # Returns
    /// Decoded bytes.
    ///
    /// # Errors
    /// Returns [`MiscCodecError::InvalidDigit`] for unsupported characters.
    #[inline]
    fn decode_to_vec(&self, text: &str) -> MiscCodecResult<Vec<u8>> {
        let start_index = self.consume_prefix(text)?;
        if let Some(separator) = self
            .config
            .separator
            .as_deref()
            .filter(|separator| !separator.is_empty())
        {
            return self.decode_separated(text, start_index, separator);
        }
        if let Some(byte_prefix) = self
            .config
            .byte_prefix
            .as_deref()
            .filter(|prefix| !prefix.is_empty())
        {
            return self.decode_byte_prefixed(text, byte_prefix, start_index);
        }
        self.decode_unprefixed(text, start_index)
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
    #[inline]
    fn consume_prefix(&self, text: &str) -> MiscCodecResult<usize> {
        let Some(prefix) = self
            .config
            .prefix
            .as_deref()
            .filter(|prefix| !prefix.is_empty())
        else {
            return Ok(0);
        };
        let index = if self.starts_with_prefix(text, prefix) {
            0
        } else {
            self.skip_ascii_whitespace(text, 0)
        };
        let rest = &text[index..];
        if self.starts_with_prefix(rest, prefix) {
            Ok(index + prefix.len())
        } else {
            Err(MiscCodecError::MissingPrefix {
                prefix: prefix.to_owned(),
            })
        }
    }

    /// Decodes separator-delimited input into bytes.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    /// - `index`: Byte index where parsing should start.
    /// - `separator`: Required separator between complete bytes.
    ///
    /// # Returns
    /// Decoded bytes.
    ///
    /// # Errors
    /// Returns [`MiscCodecError`] when a byte is malformed or the configured
    /// separator is missing between complete bytes.
    fn decode_separated(
        &self,
        text: &str,
        mut index: usize,
        separator: &str,
    ) -> MiscCodecResult<Vec<u8>> {
        let mut output = Vec::with_capacity(text.len() / 2);
        index = self.skip_ascii_whitespace(text, index);
        if index >= text.len() {
            return Ok(output);
        }
        loop {
            index = self.consume_byte_prefix(text, index)?;
            let (_, high_char, next_index) =
                read_required_hex_digit(text, index)?;
            let (_, low_char, next_index) =
                read_required_hex_digit(text, next_index)?;
            output.push(decode_hex_pair(high_char, low_char));
            index = next_index;

            let separator_index =
                self.next_separator_index(text, index, separator);
            if separator_index >= text.len() {
                return Ok(output);
            }
            let rest = &text[separator_index..];
            if !rest.starts_with(separator) {
                return Err(invalid_hex_input(&format!(
                    "missing separator '{separator}' between hex bytes"
                )));
            }
            index = self
                .skip_ascii_whitespace(text, separator_index + separator.len());
            if index >= text.len() {
                return Err(invalid_hex_input(
                    "separator must be followed by a hex byte",
                ));
            }
        }
    }

    /// Consumes the configured per-byte prefix.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    /// - `index`: Current byte index.
    ///
    /// # Returns
    /// Index after the per-byte prefix, or `index` when no non-empty per-byte
    /// prefix is configured.
    ///
    /// # Errors
    /// Returns [`MiscCodecError::MissingPrefix`] when the configured per-byte
    /// prefix is absent.
    #[inline]
    fn consume_byte_prefix(
        &self,
        text: &str,
        index: usize,
    ) -> MiscCodecResult<usize> {
        let Some(prefix) = self
            .config
            .byte_prefix
            .as_deref()
            .filter(|prefix| !prefix.is_empty())
        else {
            return Ok(index);
        };
        let index = if self.starts_with_prefix(&text[index..], prefix) {
            index
        } else {
            self.skip_ignored(text, index)
        };
        let rest = &text[index..];
        if self.starts_with_prefix(rest, prefix) {
            Ok(index + prefix.len())
        } else {
            Err(MiscCodecError::MissingPrefix {
                prefix: prefix.to_owned(),
            })
        }
    }

    /// Finds the position where the next separator must appear.
    ///
    /// # Parameters
    /// - `text`: Text being decoded.
    /// - `index`: Current byte index after a complete hex byte.
    /// - `separator`: Configured separator.
    ///
    /// # Returns
    /// Index where the separator must start, or `text.len()` when only ignored
    /// trailing whitespace remains.
    #[inline]
    fn next_separator_index(
        &self,
        text: &str,
        index: usize,
        separator: &str,
    ) -> usize {
        if !separator.chars().all(|ch| ch.is_ascii_whitespace())
            && text
                .get(index..)
                .is_some_and(|rest| rest.starts_with(separator))
        {
            return index;
        }
        let whitespace_end = self.skip_ascii_whitespace(text, index);
        if whitespace_end >= text.len() {
            return whitespace_end;
        }
        if separator.chars().all(|ch| ch.is_ascii_whitespace()) {
            index
        } else {
            whitespace_end
        }
    }

    /// Decodes unprefixed input characters into bytes.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    ///
    /// # Returns
    /// Decoded bytes.
    ///
    /// # Errors
    /// Returns [`MiscCodecError::InvalidDigit`] for unsupported characters.
    fn decode_unprefixed(
        &self,
        text: &str,
        mut index: usize,
    ) -> MiscCodecResult<Vec<u8>> {
        let mut output = Vec::with_capacity(text.len() / 2);
        let mut pending = None;
        let mut digit_count = 0usize;
        while index < text.len() {
            let rest = &text[index..];
            let ch = rest.chars().next().expect("index is below text length");
            if self.config.ignore_ascii_whitespace && ch.is_ascii_whitespace() {
                index += ch.len_utf8();
                continue;
            }
            if hex_value(ch).is_some() {
                digit_count += 1;
                if let Some(high_char) = pending.take() {
                    output.push(decode_hex_pair(high_char, ch));
                } else {
                    pending = Some(ch);
                }
                index += ch.len_utf8();
                continue;
            }
            return Err(invalid_hex_digit(index, ch));
        }
        if !digit_count.is_multiple_of(2) {
            return Err(invalid_hex_length(digit_count));
        }
        Ok(output)
    }

    /// Decodes byte-prefixed input characters into bytes.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    /// - `prefix`: Required prefix before each byte.
    /// - `index`: Byte index where parsing should start.
    ///
    /// # Returns
    /// Decoded bytes.
    ///
    /// # Errors
    /// Returns [`MiscCodecError::MissingPrefix`] when a byte prefix is missing,
    /// or [`MiscCodecError::InvalidDigit`] for unsupported characters.
    fn decode_byte_prefixed(
        &self,
        text: &str,
        prefix: &str,
        mut index: usize,
    ) -> MiscCodecResult<Vec<u8>> {
        let mut output = Vec::with_capacity(text.len() / 2);
        let mut digit_count = 0usize;
        while index < text.len() {
            index = self.skip_ignored(text, index);
            if index >= text.len() {
                break;
            }
            let rest = &text[index..];
            if !self.starts_with_prefix(rest, prefix) {
                return Err(MiscCodecError::MissingPrefix {
                    prefix: prefix.to_owned(),
                });
            }
            index += prefix.len();

            let mut pair_count = 0;
            let mut pair = [(0usize, '\0'); 2];
            while pair_count < 2 && index < text.len() {
                let rest = &text[index..];
                let ch =
                    rest.chars().next().expect("index is below text length");
                if self.config.ignore_ascii_whitespace
                    && ch.is_ascii_whitespace()
                {
                    index += ch.len_utf8();
                    continue;
                }
                if hex_value(ch).is_some() {
                    pair[pair_count] = (index, ch);
                    index += ch.len_utf8();
                    pair_count += 1;
                    digit_count += 1;
                    continue;
                }
                return Err(invalid_hex_digit(index, ch));
            }
            if pair_count != 2 {
                return Err(invalid_hex_input(
                    "byte prefix must be followed by two hex digits",
                ));
            }
            output.push(decode_hex_pair(pair[0].1, pair[1].1));
        }
        if !digit_count.is_multiple_of(2) {
            return Err(invalid_hex_length(digit_count));
        }
        Ok(output)
    }

    /// Skips ignored ASCII whitespace.
    ///
    /// # Parameters
    /// - `text`: Text being decoded.
    /// - `index`: Current byte index.
    ///
    /// # Returns
    /// The next byte index that should be parsed.
    #[inline]
    fn skip_ignored(&self, text: &str, mut index: usize) -> usize {
        while index < text.len() {
            let byte = text.as_bytes()[index];
            if self.config.ignore_ascii_whitespace && byte.is_ascii_whitespace()
            {
                index += 1;
                continue;
            }
            return index;
        }
        index
    }

    /// Skips ignored leading ASCII whitespace.
    ///
    /// # Parameters
    /// - `text`: Text being decoded.
    /// - `index`: Current byte index.
    ///
    /// # Returns
    /// The next byte index after ignored ASCII whitespace.
    #[inline]
    fn skip_ascii_whitespace(&self, text: &str, mut index: usize) -> usize {
        while self.config.ignore_ascii_whitespace && index < text.len() {
            if !text.as_bytes()[index].is_ascii_whitespace() {
                return index;
            }
            index += 1;
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
    #[inline]
    fn starts_with_prefix(&self, text: &str, prefix: &str) -> bool {
        if !self.config.ignore_prefix_case {
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
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl ValueEncoder<[u8]> for HexCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Encodes bytes into hexadecimal text.
    #[inline]
    fn encode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(HexCodec::encode(self, input))
    }
}

impl ValueDecoder<str> for HexCodec {
    type Error = MiscCodecError;
    type Output = Vec<u8>;

    /// Decodes hexadecimal text into bytes.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
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
#[inline(always)]
pub(crate) fn hex_value(ch: char) -> Option<u8> {
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
pub(crate) fn invalid_hex_digit(
    index: usize,
    character: char,
) -> MiscCodecError {
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

/// Builds an invalid hexadecimal input error.
///
/// # Parameters
/// - `reason`: Human-readable reason the input was rejected.
///
/// # Returns
/// An invalid input error for the hexadecimal codec.
fn invalid_hex_input(reason: &str) -> MiscCodecError {
    MiscCodecError::InvalidInput {
        codec: "hex",
        reason: reason.to_owned(),
    }
}

/// Reads one required hexadecimal digit at a byte boundary.
///
/// # Parameters
/// - `text`: Text being decoded.
/// - `index`: Byte index where the digit is expected.
///
/// # Returns
/// Original digit index, digit character, and the next byte index.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidInput`] when input ends before the digit,
/// or [`MiscCodecError::InvalidDigit`] when the next character is not hex.
#[inline]
fn read_required_hex_digit(
    text: &str,
    index: usize,
) -> MiscCodecResult<(usize, char, usize)> {
    let rest = &text[index..];
    let Some(character) = rest.chars().next() else {
        return Err(invalid_hex_input("expected a hexadecimal digit"));
    };
    if hex_value(character).is_none() {
        return Err(invalid_hex_digit(index, character));
    }
    Ok((index, character, index + character.len_utf8()))
}

/// Decodes two hexadecimal characters into one byte.
///
/// # Parameters
/// - `high_index`: Original byte index of the high nibble.
/// - `high_char`: High-nibble character.
/// - `low_index`: Original byte index of the low nibble.
/// - `low_char`: Low-nibble character.
///
/// # Returns
/// The decoded byte.
///
/// The callers validate both characters before constructing the pair.
#[inline]
fn decode_hex_pair(high_char: char, low_char: char) -> u8 {
    let high = hex_value(high_char).expect("validated hexadecimal digit");
    let low = hex_value(low_char).expect("validated hexadecimal digit");
    (high << 4) | low
}

/// Appends one encoded byte to `output`.
///
/// # Parameters
/// - `byte`: Byte to encode.
/// - `uppercase`: Whether to use uppercase digits.
/// - `output`: Destination string.
#[inline(always)]
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
#[inline(always)]
pub(crate) fn hex_digit(value: u8, uppercase: bool) -> char {
    let digits = if uppercase {
        &UPPER_HEX_DIGITS
    } else {
        &LOWER_HEX_DIGITS
    };
    digits[(value & 0x0f) as usize]
}
