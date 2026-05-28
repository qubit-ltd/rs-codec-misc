/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Base64 byte codec.

use ::base64::Engine;
use ::base64::engine::general_purpose::{
    STANDARD,
    STANDARD_NO_PAD,
    URL_SAFE,
    URL_SAFE_NO_PAD,
};

use crate::{
    MiscCodecError,
    MiscCodecResult,
    ValueDecoder,
    ValueEncoder,
};

/// Encodes and decodes Base64 byte strings.
///
/// This facade intentionally remains a whole-value codec backed by the
/// `base64` crate. Final partial quantum handling and optional `=` padding are
/// facade/transcoder responsibilities, not part of the low-level quantum codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Base64Codec {
    url_safe: bool,
    padding: bool,
}

impl Base64Codec {
    /// Creates a standard Base64 codec with padding.
    ///
    /// # Returns
    /// Standard Base64 codec.
    pub fn standard() -> Self {
        Self {
            url_safe: false,
            padding: true,
        }
    }

    /// Creates a standard Base64 codec without padding.
    ///
    /// # Returns
    /// Standard no-padding Base64 codec.
    pub fn standard_no_pad() -> Self {
        Self {
            url_safe: false,
            padding: false,
        }
    }

    /// Creates a URL-safe Base64 codec with padding.
    ///
    /// # Returns
    /// URL-safe Base64 codec.
    pub fn url_safe() -> Self {
        Self {
            url_safe: true,
            padding: true,
        }
    }

    /// Creates a URL-safe Base64 codec without padding.
    ///
    /// # Returns
    /// URL-safe no-padding Base64 codec.
    pub fn url_safe_no_pad() -> Self {
        Self {
            url_safe: true,
            padding: false,
        }
    }

    /// Encodes bytes into Base64 text.
    ///
    /// # Parameters
    /// - `bytes`: Bytes to encode.
    ///
    /// # Returns
    /// Encoded Base64 text.
    pub fn encode(&self, bytes: &[u8]) -> String {
        self.engine().encode(bytes)
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
    pub fn decode(&self, text: &str) -> MiscCodecResult<Vec<u8>> {
        self.engine()
            .decode(text)
            .map_err(|source| MiscCodecError::InvalidInput {
                codec: "base64",
                reason: source.to_string(),
            })
    }

    /// Selects the concrete Base64 engine.
    ///
    /// # Returns
    /// Base64 engine matching this codec's alphabet and padding settings.
    fn engine(&self) -> &'static ::base64::engine::GeneralPurpose {
        match (self.url_safe, self.padding) {
            (false, true) => &STANDARD,
            (false, false) => &STANDARD_NO_PAD,
            (true, true) => &URL_SAFE,
            (true, false) => &URL_SAFE_NO_PAD,
        }
    }
}

impl Default for Base64Codec {
    /// Creates a standard Base64 codec with padding.
    fn default() -> Self {
        Self::standard()
    }
}

impl ValueEncoder<[u8]> for Base64Codec {
    type Error = MiscCodecError;
    type Output = String;

    /// Encodes bytes into Base64 text.
    fn encode(&self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(Base64Codec::encode(self, input))
    }
}

impl ValueDecoder<str> for Base64Codec {
    type Error = MiscCodecError;
    type Output = Vec<u8>;

    /// Decodes Base64 text into bytes.
    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Base64Codec::decode(self, input)
    }
}
