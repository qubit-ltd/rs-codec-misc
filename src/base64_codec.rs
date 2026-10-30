// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Base64 byte codec.

use ::base64::Engine;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;

use crate::Base64Decoder;
use crate::Base64Encoder;
use crate::MiscCodecError;
use crate::MiscCodecResult;
use crate::internal::Base64Config;

/// Encodes and decodes Base64 byte strings.
///
/// This facade intentionally remains a whole-value codec backed by the
/// `base64` crate. Final partial quantum handling and optional `=` padding are
/// facade/transcoder responsibilities, not part of the low-level quantum codec.
/// Decoding is strict: each constructor accepts only its configured alphabet
/// and padding policy, rejects whitespace and non-canonical trailing bits, and
/// does not permit padded and unpadded partial groups to be interchanged.
/// Complete three-byte groups such as `TWFu` need no `=` and are accepted by
/// both padding policies.
///
/// # Examples
///
/// ```
/// use qubit_codec_misc::Base64Codec;
///
/// let codec = Base64Codec::standard();
/// assert_eq!(codec.encode(b"hello"), "aGVsbG8=");
/// ```
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct Base64Codec {
    /// Configuration that defines this codec's alphabet and padding policy.
    config: Base64Config,
}

impl PartialEq for Base64Codec {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
    }
}

impl Eq for Base64Codec {}

impl Base64Codec {
    /// Creates a standard Base64 codec that requires canonical padding for a
    /// partial final group. Complete groups need no `=`.
    ///
    /// # Returns
    /// Standard Base64 codec.
    #[inline]
    pub fn standard() -> Self {
        Self {
            config: Base64Config::Standard,
        }
    }

    /// Creates a standard Base64 codec that rejects padding and accepts
    /// unpadded partial final groups.
    ///
    /// # Returns
    /// Standard no-padding Base64 codec.
    #[inline]
    pub fn standard_no_pad() -> Self {
        Self {
            config: Base64Config::StandardNoPad,
        }
    }

    /// Creates a URL-safe Base64 codec that requires canonical padding for a
    /// partial final group. Complete groups need no `=`.
    ///
    /// # Returns
    /// URL-safe Base64 codec.
    #[inline]
    pub fn url_safe() -> Self {
        Self {
            config: Base64Config::UrlSafe,
        }
    }

    /// Creates a URL-safe Base64 codec that rejects padding and accepts
    /// unpadded partial final groups.
    ///
    /// # Returns
    /// URL-safe no-padding Base64 codec.
    #[inline]
    pub fn url_safe_no_pad() -> Self {
        Self {
            config: Base64Config::UrlSafeNoPad,
        }
    }

    /// Encodes bytes into Base64 text.
    ///
    /// # Parameters
    /// - `bytes`: Bytes to encode.
    ///
    /// # Returns
    /// Encoded Base64 text.
    #[inline]
    #[must_use]
    pub fn encode(&self, bytes: &[u8]) -> String {
        self.config.engine().encode(bytes)
    }

    /// Decodes Base64 text into bytes.
    ///
    /// # Parameters
    /// - `text`: Base64 text.
    ///
    /// # Returns
    /// Decoded bytes.
    ///
    /// # Errors
    /// Returns [`MiscCodecError::InvalidBase64`] when `text` uses another
    /// alphabet, violates this codec's padding policy, contains whitespace, or
    /// has non-canonical trailing bits.
    #[inline]
    pub fn decode(&self, text: &str) -> MiscCodecResult<Vec<u8>> {
        self.config.engine().decode(text).map_err(MiscCodecError::from)
    }

    /// Creates an independent encoder with this codec's configuration.
    ///
    /// The returned encoder does not borrow this codec, uses fixed-size state,
    /// and must be reset before streaming.
    ///
    /// # Returns
    /// An independent streaming encoder with the same alphabet and padding
    /// policy.
    #[inline]
    pub fn encoder(&self) -> Base64Encoder {
        Base64Encoder::from_config(self.config)
    }

    /// Creates an independent decoder with this codec's configuration.
    ///
    /// The returned decoder does not borrow this codec, uses fixed-size state,
    /// and must be reset before streaming.
    ///
    /// # Returns
    /// An independent streaming decoder with the same alphabet and padding
    /// policy.
    #[inline]
    pub fn decoder(&self) -> Base64Decoder {
        Base64Decoder::from_config(self.config)
    }
}

impl Default for Base64Codec {
    /// Creates a standard Base64 codec with padding.
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

impl ValueEncoder<[u8]> for Base64Codec {
    type Error = MiscCodecError;
    type Output = String;

    /// Encodes bytes into Base64 text without a fallible operation.
    ///
    /// # Parameters
    /// - `input`: Bytes to encode.
    ///
    /// # Returns
    /// Encoded Base64 text.
    ///
    /// # Errors
    /// This implementation does not return an error.
    #[inline]
    fn encode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(Base64Codec::encode(self, input))
    }
}

impl ValueDecoder<str> for Base64Codec {
    type Error = MiscCodecError;
    type Output = Vec<u8>;

    /// Decodes Base64 text into bytes using this codec's alphabet and padding
    /// policy.
    ///
    /// # Parameters
    /// - `input`: Base64 text to decode.
    ///
    /// # Returns
    /// Decoded bytes.
    ///
    /// # Errors
    /// Returns [`MiscCodecError::InvalidBase64`] when the input is malformed
    /// or violates this codec's alphabet or padding policy.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Base64Codec::decode(self, input)
    }
}
