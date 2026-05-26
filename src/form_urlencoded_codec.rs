/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! `application/x-www-form-urlencoded` text codec.

use crate::percent_codec::{
    percent_decode_bytes,
    percent_encode_bytes,
};
use crate::{
    Decoder,
    Encoder,
    MiscCodecError,
    MiscCodecResult,
};

/// Encodes and decodes `application/x-www-form-urlencoded` text fragments.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FormUrlencodedCodec;

impl FormUrlencodedCodec {
    /// Creates a form-url-encoded codec.
    ///
    /// # Returns
    /// Form URL encoded codec.
    pub fn new() -> Self {
        Self
    }

    /// Encodes text, using `+` for spaces.
    ///
    /// # Parameters
    /// - `text`: Text to encode.
    ///
    /// # Returns
    /// Form-url-encoded text.
    pub fn encode(&self, text: &str) -> String {
        percent_encode_bytes(text.as_bytes(), true)
    }

    /// Decodes text, treating `+` as space.
    ///
    /// # Parameters
    /// - `text`: Text to decode.
    ///
    /// # Returns
    /// Decoded UTF-8 text.
    ///
    /// # Errors
    /// Returns [`MiscCodecError`] when an escape is malformed or decoded bytes are
    /// not valid UTF-8.
    pub fn decode(&self, text: &str) -> MiscCodecResult<String> {
        String::from_utf8(percent_decode_bytes(text, true)?).map_err(MiscCodecError::from)
    }
}

impl Encoder<str> for FormUrlencodedCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Encodes text, using `+` for spaces.
    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(FormUrlencodedCodec::encode(self, input))
    }
}

impl Decoder<str> for FormUrlencodedCodec {
    type Error = MiscCodecError;
    type Output = String;

    /// Decodes form-url-encoded text.
    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        FormUrlencodedCodec::decode(self, input)
    }
}
