// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Hexadecimal byte codec.
// qubit-style: allow multiple-public-types

use crate::{
    Codec,
    MiscCodecError,
    MiscCodecResult,
    ValueDecoder,
    ValueEncoder,
    misc_codec_error::map_misc_decode_failure,
};

const LOWER_HEX_DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e',
    'f',
];

const UPPER_HEX_DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E',
    'F',
];

/// Encodes and decodes one byte as two ASCII hexadecimal units.
///
/// `HexByteCodec` is the low-level [`Codec`] implementation for streaming or
/// generic codec call sites. It does not understand whole-string prefixes,
/// per-byte prefixes, separators, or whitespace. Use [`HexCodec`] for owned
/// byte-slice helpers with those formatting options.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HexByteCodec {
    uppercase: bool,
}

impl HexByteCodec {
    /// Creates a lowercase single-byte hexadecimal codec.
    ///
    /// # Returns
    ///
    /// A byte codec using lowercase digits.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self { uppercase: false }
    }

    /// Creates an uppercase single-byte hexadecimal codec.
    ///
    /// # Returns
    ///
    /// A byte codec using uppercase digits.
    #[must_use]
    #[inline]
    pub const fn upper() -> Self {
        Self { uppercase: true }
    }

    /// Sets whether encoded digits should be uppercase.
    ///
    /// # Parameters
    /// - `uppercase`: Whether to use uppercase hexadecimal digits.
    ///
    /// # Returns
    /// The updated byte codec.
    #[must_use]
    #[inline]
    pub const fn with_uppercase(mut self, uppercase: bool) -> Self {
        self.uppercase = uppercase;
        self
    }

    /// Returns whether this byte codec emits uppercase hexadecimal digits.
    ///
    /// # Returns
    /// `true` when uppercase digits are selected.
    #[must_use]
    #[inline]
    pub const fn is_uppercase(self) -> bool {
        self.uppercase
    }
}

/// Encodes and decodes hexadecimal byte strings.
///
/// `HexCodec` is an owned facade for whole byte slices. Whole-string prefix,
/// per-byte prefix, separator, and whitespace handling are part of
/// [`encode`](Self::encode) and [`decode`](Self::decode). Use
/// [`HexByteCodec`] when a low-level [`Codec<Value = u8, Unit = u8>`] is
/// required.
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn encode(&self, bytes: &[u8]) -> String {
        let separator_len = self.separator.as_ref().map_or(0, String::len);
        let prefix_len = self.prefix.as_ref().map_or(0, String::len);
        let byte_prefix_len = self.byte_prefix.as_ref().map_or(0, String::len);
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
    /// Returns [`MiscCodecError`] when a configured whole or per-byte prefix is
    /// missing, when the normalized digit count is odd, or when a non-hex
    /// digit is found.
    #[inline]
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
    #[inline]
    pub fn decode_into(
        &self,
        text: &str,
        output: &mut Vec<u8>,
    ) -> MiscCodecResult<()> {
        let digits = self.normalized_digits(text)?;
        if digits.len() % 2 != 0 {
            return Err(invalid_hex_length(digits.len()));
        }
        output.reserve(digits.len() / 2);
        for pair in digits.chunks_exact(2) {
            let (high_index, high_char) = pair[0];
            let (low_index, low_char) = pair[1];
            let high = hex_value(high_char)
                .ok_or(invalid_hex_digit(high_index, high_char))?;
            let low = hex_value(low_char)
                .ok_or(invalid_hex_digit(low_index, low_char))?;
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
    #[inline]
    fn normalized_digits(
        &self,
        text: &str,
    ) -> MiscCodecResult<Vec<(usize, char)>> {
        let start_index = self.consume_prefix(text)?;
        if let Some(separator) = self
            .separator
            .as_deref()
            .filter(|separator| !separator.is_empty())
        {
            return self.normalized_separated_digits(
                text,
                start_index,
                separator,
            );
        }
        if let Some(byte_prefix) = self
            .byte_prefix
            .as_deref()
            .filter(|prefix| !prefix.is_empty())
        {
            return self.normalized_byte_prefixed_digits(
                text,
                byte_prefix,
                start_index,
            );
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
    #[inline]
    fn consume_prefix(&self, text: &str) -> MiscCodecResult<usize> {
        let Some(prefix) =
            self.prefix.as_deref().filter(|prefix| !prefix.is_empty())
        else {
            return Ok(0);
        };
        let index = self.skip_ascii_whitespace(text, 0);
        let rest = &text[index..];
        if self.starts_with_prefix(rest, prefix) {
            Ok(index + prefix.len())
        } else {
            Err(MiscCodecError::MissingPrefix {
                prefix: prefix.to_owned(),
            })
        }
    }

    /// Normalizes separator-delimited input into hex digits.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    /// - `index`: Byte index where parsing should start.
    /// - `separator`: Required separator between complete bytes.
    ///
    /// # Returns
    /// Hex digits paired with their original character indexes.
    ///
    /// # Errors
    /// Returns [`MiscCodecError`] when a byte is malformed or the configured
    /// separator is missing between complete bytes.
    fn normalized_separated_digits(
        &self,
        text: &str,
        mut index: usize,
        separator: &str,
    ) -> MiscCodecResult<Vec<(usize, char)>> {
        let mut digits = Vec::with_capacity(text.len());
        index = self.skip_ascii_whitespace(text, index);
        if index >= text.len() {
            return Ok(digits);
        }
        loop {
            index = self.consume_byte_prefix(text, index)?;
            let (high_index, high_char, next_index) =
                read_required_hex_digit(text, index)?;
            let (low_index, low_char, next_index) =
                read_required_hex_digit(text, next_index)?;
            digits.push((high_index, high_char));
            digits.push((low_index, low_char));
            index = next_index;

            let separator_index =
                self.next_separator_index(text, index, separator);
            if separator_index >= text.len() {
                return Ok(digits);
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
            .byte_prefix
            .as_deref()
            .filter(|prefix| !prefix.is_empty())
        else {
            return Ok(index);
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
    fn normalized_unprefixed_digits(
        &self,
        text: &str,
        mut index: usize,
    ) -> MiscCodecResult<Vec<(usize, char)>> {
        let mut digits = Vec::with_capacity(text.len());
        while index < text.len() {
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
    /// Returns [`MiscCodecError::MissingPrefix`] when a byte prefix is missing,
    /// or [`MiscCodecError::InvalidDigit`] for unsupported characters.
    fn normalized_byte_prefixed_digits(
        &self,
        text: &str,
        prefix: &str,
        mut index: usize,
    ) -> MiscCodecResult<Vec<(usize, char)>> {
        let mut digits = Vec::with_capacity(text.len());
        while index < text.len() {
            index = self.skip_ignored(text, index);
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
            if self.ignore_ascii_whitespace && byte.is_ascii_whitespace() {
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
        while self.ignore_ascii_whitespace && index < text.len() {
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

impl Codec for HexByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = MiscCodecError;
    type EncodeError = MiscCodecError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);

    /// Decodes one byte from two ASCII hexadecimal digits.
    #[inline]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(index + 2 <= input.len());

        let high_char = char::from(input[index]);
        let low_char = char::from(input[index + 1]);
        let high = hex_value(high_char)
            .ok_or_else(|| invalid_hex_digit(index, high_char))
            .map_err(map_misc_decode_failure)?;
        let low = hex_value(low_char)
            .ok_or_else(|| invalid_hex_digit(index + 1, low_char))
            .map_err(map_misc_decode_failure)?;
        Ok(((high << 4) | low, qubit_io::nz!(2)))
    }

    /// Encodes one byte as two ASCII hexadecimal digits.
    #[inline]
    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index + 2 <= output.len());

        output[index] = hex_digit(*value >> 4, self.uppercase) as u8;
        output[index + 1] = hex_digit(*value & 0x0f, self.uppercase) as u8;
        Ok(qubit_io::nz!(2))
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
    let Some(rest) = text.get(index..) else {
        return Err(invalid_hex_input(
            "expected a hexadecimal digit at a character boundary",
        ));
    };
    let Some(character) = rest.chars().next() else {
        return Err(invalid_hex_input("expected a hexadecimal digit"));
    };
    if hex_value(character).is_none() {
        return Err(invalid_hex_digit(index, character));
    }
    Ok((index, character, index + character.len_utf8()))
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
fn hex_digit(value: u8, uppercase: bool) -> char {
    let digits = if uppercase {
        &UPPER_HEX_DIGITS
    } else {
        &LOWER_HEX_DIGITS
    };
    digits[(value & 0x0f) as usize]
}
