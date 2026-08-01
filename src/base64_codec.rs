// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Base64 byte codec.

use ::base64::Engine;
use ::base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};

use crate::{MiscCodecError, MiscCodecResult};
use qubit_codec::{ValueDecoder, ValueEncoder};

/// Encodes and decodes Base64 byte strings.
///
/// This facade intentionally remains a whole-value codec backed by the
/// `base64` crate. Final partial quantum handling and optional `=` padding are
/// facade/transcoder responsibilities, not part of the low-level quantum codec.
#[derive(Debug, Clone, Copy)]
pub struct Base64Codec {
    engine: &'static ::base64::engine::GeneralPurpose,
}

impl PartialEq for Base64Codec {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.engine, other.engine)
    }
}

impl Eq for Base64Codec {}

impl Base64Codec {
    /// Creates a standard Base64 codec with padding.
    ///
    /// # Returns
    /// Standard Base64 codec.
    #[inline]
    pub fn standard() -> Self {
        Self { engine: &STANDARD }
    }

    /// Creates a standard Base64 codec without padding.
    ///
    /// # Returns
    /// Standard no-padding Base64 codec.
    #[inline]
    pub fn standard_no_pad() -> Self {
        Self {
            engine: &STANDARD_NO_PAD,
        }
    }

    /// Creates a URL-safe Base64 codec with padding.
    ///
    /// # Returns
    /// URL-safe Base64 codec.
    #[inline]
    pub fn url_safe() -> Self {
        Self { engine: &URL_SAFE }
    }

    /// Creates a URL-safe Base64 codec without padding.
    ///
    /// # Returns
    /// URL-safe no-padding Base64 codec.
    #[inline]
    pub fn url_safe_no_pad() -> Self {
        Self {
            engine: &URL_SAFE_NO_PAD,
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
    /// Returns [`MiscCodecError::InvalidInput`] when `text` is malformed.
    #[inline]
    pub fn decode(&self, text: &str) -> MiscCodecResult<Vec<u8>> {
        self.engine
            .decode(text)
            .map_err(|source| MiscCodecError::InvalidInput {
                codec: "base64",
                reason: source.to_string(),
            })
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
    #[inline(always)]
    fn encode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(Base64Codec::encode(self, input))
    }
}

impl ValueDecoder<str> for Base64Codec {
    type Error = MiscCodecError;
    type Output = Vec<u8>;

    /// Decodes Base64 text into bytes.
    #[inline(always)]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Base64Codec::decode(self, input)
    }
}
