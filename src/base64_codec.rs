// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Base64 byte codec.

use ::base64::Engine;
use ::base64::engine::GeneralPurpose;
use ::base64::engine::general_purpose::STANDARD;
use ::base64::engine::general_purpose::STANDARD_NO_PAD;
use ::base64::engine::general_purpose::URL_SAFE;
use ::base64::engine::general_purpose::URL_SAFE_NO_PAD;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;

use crate::MiscCodecError;
use crate::MiscCodecResult;

/// Stable identity for the standard padded Base64 configuration.
static STANDARD_ENGINE: GeneralPurpose = STANDARD;
/// Stable identity for the standard unpadded Base64 configuration.
static STANDARD_NO_PAD_ENGINE: GeneralPurpose = STANDARD_NO_PAD;
/// Stable identity for the URL-safe padded Base64 configuration.
static URL_SAFE_ENGINE: GeneralPurpose = URL_SAFE;
/// Stable identity for the URL-safe unpadded Base64 configuration.
static URL_SAFE_NO_PAD_ENGINE: GeneralPurpose = URL_SAFE_NO_PAD;

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
    /// Static engine that defines this codec's alphabet and padding policy.
    engine: &'static GeneralPurpose,
}

impl PartialEq for Base64Codec {
    // Every constructor refers to one of this module's four explicit static
    // engines, so engine address identity represents the fixed configuration.
    // Do not construct this facade from references to dependency const values.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.engine, other.engine)
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
            engine: &STANDARD_ENGINE,
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
            engine: &STANDARD_NO_PAD_ENGINE,
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
            engine: &URL_SAFE_ENGINE,
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
            engine: &URL_SAFE_NO_PAD_ENGINE,
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
        self.engine.encode(bytes)
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
        self.engine.decode(text).map_err(MiscCodecError::from)
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

    /// Encodes bytes into Base64 text.
    #[inline]
    fn encode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(Base64Codec::encode(self, input))
    }
}

impl ValueDecoder<str> for Base64Codec {
    type Error = MiscCodecError;
    type Output = Vec<u8>;

    /// Decodes Base64 text into bytes.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Base64Codec::decode(self, input)
    }
}
