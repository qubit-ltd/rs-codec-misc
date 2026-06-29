// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Percent text codec.

use crate::{
    Codec, MiscCodecError, MiscCodecResult, ValueDecoder, ValueEncoder,
    misc_codec_error::map_misc_decode_failure,
};

const UPPER_HEX_DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];

/// Encodes and decodes percent-encoded UTF-8 text.
///
/// Its low-level [`Codec<Value = u8, Unit = u8>`] implementation converts one
/// byte to either one unreserved ASCII unit or a `%XX` escape. UTF-8 validation
/// remains part of the owned [`decode`](Self::decode) helper.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PercentCodec;

impl PercentCodec {
    /// Creates a percent codec.
    ///
    /// # Returns
    /// Percent codec.
    #[inline]
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
    #[inline]
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
    /// Returns [`MiscCodecError`] when a percent escape is malformed or decoded
    /// bytes are not valid UTF-8.
    #[inline]
    pub fn decode(&self, text: &str) -> MiscCodecResult<String> {
        String::from_utf8(percent_decode_bytes(text, false)?).map_err(MiscCodecError::from)
    }
}

impl ValueEncoder<str> for PercentCodec {
    type Error = MiscCodecError;
    type DomainError = MiscCodecError;
    type Output = String;

    /// Maps percent-encoding domain errors to the public encoder error.
    #[inline(always)]
    fn map_error(&self, error: Self::DomainError) -> Self::Error {
        error
    }

    /// Encodes text using percent encoding.
    #[inline]
    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(PercentCodec::encode(self, input))
    }
}

impl ValueDecoder<str> for PercentCodec {
    type Error = MiscCodecError;
    type DomainError = MiscCodecError;
    type Output = String;

    /// Maps percent-encoding domain errors to the public decoder error.
    #[inline(always)]
    fn map_error(&self, error: Self::DomainError) -> Self::Error {
        error
    }

    /// Decodes percent-encoded text.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        PercentCodec::decode(self, input)
    }
}

impl Codec for PercentCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = MiscCodecError;
    type EncodeError = MiscCodecError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(1);
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(3);

    /// Returns the exact percent-encoded width for one byte.
    #[inline(always)]
    fn encode_len(&self, value: &u8) -> core::num::NonZeroUsize {
        if is_unreserved(*value) {
            qubit_io::nz!(1)
        } else {
            qubit_io::nz!(3)
        }
    }

    /// Decodes one raw byte or `%XX` escape.
    #[inline]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());

        let (value, consumed) =
            percent_decode_byte(input, input_index, false).map_err(map_misc_decode_failure)?;
        debug_assert!(consumed > 0);
        // SAFETY: `percent_decode_byte` returns a non-zero width for every
        // successful raw byte or escape.
        let consumed = qubit_io::nz!(consumed);
        Ok((value, consumed))
    }

    /// Encodes one byte using percent encoding.
    #[inline]
    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(output_index + if is_unreserved(*value) { 1 } else { 3 } <= output.len());

        let written = percent_encode_byte(*value, output, output_index, false);
        let required = <Self as Codec>::encode_len(self, value);
        debug_assert_eq!(written, required.get());
        Ok(required)
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
#[inline]
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
/// Returns [`MiscCodecError::InvalidEscape`] for malformed escapes.
#[inline]
pub(crate) fn percent_decode_bytes(text: &str, plus_as_space: bool) -> MiscCodecResult<Vec<u8>> {
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let (decoded, consumed) = percent_decode_byte(bytes, index, plus_as_space)?;
        output.push(decoded);
        index += consumed;
    }
    Ok(output)
}

/// Percent-encodes one byte into `output`.
///
/// # Parameters
/// - `byte`: Byte to encode.
/// - `output`: Destination unit buffer.
/// - `index`: Start index in `output`.
/// - `space_as_plus`: Whether spaces should be encoded as `+`.
///
/// # Returns
/// Number of units written.
#[inline]
pub(crate) fn percent_encode_byte(
    byte: u8,
    output: &mut [u8],
    index: usize,
    space_as_plus: bool,
) -> usize {
    if byte == b' ' && space_as_plus {
        output[index] = b'+';
        return 1;
    }
    if is_unreserved(byte) {
        output[index] = byte;
        return 1;
    }
    output[index] = b'%';
    output[index + 1] = percent_hex_digit(byte >> 4) as u8;
    output[index + 2] = percent_hex_digit(byte & 0x0f) as u8;
    3
}

/// Decodes one raw byte or `%XX` escape from `input`.
///
/// # Parameters
/// - `input`: Encoded byte units.
/// - `index`: Start index in `input`.
/// - `plus_as_space`: Whether `+` should decode to a space byte.
///
/// # Returns
/// Decoded byte and the number of consumed units.
///
/// # Errors
/// Returns [`MiscCodecError::InvalidEscape`] for malformed `%XX` escapes.
#[inline]
pub(crate) fn percent_decode_byte(
    input: &[u8],
    index: usize,
    plus_as_space: bool,
) -> MiscCodecResult<(u8, usize)> {
    let available = input.len().saturating_sub(index);
    if available == 0 {
        return Err(MiscCodecError::Incomplete {
            required: qubit_io::nz!(1),
            available,
        });
    }
    match input[index] {
        b'%' => {
            if available < 3 {
                return Err(MiscCodecError::Incomplete {
                    required: qubit_io::nz!(3),
                    available,
                });
            }
            let (Some(&high_byte), Some(&low_byte)) = (input.get(index + 1), input.get(index + 2))
            else {
                return Err(invalid_percent_escape(index));
            };
            let high = percent_hex_value(high_byte).ok_or_else(|| invalid_percent_escape(index))?;
            let low = percent_hex_value(low_byte).ok_or_else(|| invalid_percent_escape(index))?;
            Ok(((high << 4) | low, 3))
        }
        b'+' if plus_as_space => Ok((b' ', 1)),
        byte => Ok((byte, 1)),
    }
}

/// Builds a malformed percent escape error.
///
/// # Parameters
/// - `index`: Byte index of the `%` marker in the original input.
///
/// # Returns
/// An invalid escape error for a `%XX` sequence.
fn invalid_percent_escape(index: usize) -> MiscCodecError {
    MiscCodecError::InvalidEscape {
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
#[inline(always)]
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
#[inline(always)]
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
/// Uppercase hexadecimal digit. Values above `0x0f` are masked to their low
/// nibble.
#[inline(always)]
fn percent_hex_digit(value: u8) -> char {
    UPPER_HEX_DIGITS[(value & 0x0f) as usize]
}
