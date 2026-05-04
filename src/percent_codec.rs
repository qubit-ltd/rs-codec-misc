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

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

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
            output.push(HEX_DIGITS[(byte >> 4) as usize] as char);
            output.push(HEX_DIGITS[(byte & 0x0f) as usize] as char);
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
/// Returns [`CodecError::InvalidPercentEscape`] for malformed escapes.
pub(crate) fn percent_decode_bytes(text: &str, plus_as_space: bool) -> CodecResult<Vec<u8>> {
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(CodecError::InvalidPercentEscape { index });
                }
                let high = percent_hex_value(bytes[index + 1])
                    .ok_or(CodecError::InvalidPercentEscape { index })?;
                let low = percent_hex_value(bytes[index + 2])
                    .ok_or(CodecError::InvalidPercentEscape { index })?;
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
