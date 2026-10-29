// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Streaming Base64 decoder configuration and state.

use crate::internal::{Base64Config, Base64StreamPhase};

/// Decodes Base64 incrementally using a strict configuration.
///
/// The decoder uses fixed-size state and does not allocate. Call `reset`
/// before streaming. Input and output buffers remain caller-owned.
#[must_use]
#[derive(Debug)]
pub struct Base64Decoder {
    pub(crate) config: Base64Config,
    pub(crate) phase: Base64StreamPhase,
    pub(crate) terminal_padding: bool,
}

impl Base64Decoder {
    pub(crate) fn from_config(config: Base64Config) -> Self {
        Self {
            config,
            phase: Base64StreamPhase::Uninitialized,
            terminal_padding: false,
        }
    }

    /// Creates a strict standard Base64 decoder with padding.
    ///
    /// The decoder requires `reset` before streaming and does not allocate.
    pub fn standard() -> Self {
        Self::from_config(Base64Config::Standard)
    }

    /// Creates a strict standard Base64 decoder without padding.
    ///
    /// The decoder requires `reset` before streaming and does not allocate.
    pub fn standard_no_pad() -> Self {
        Self::from_config(Base64Config::StandardNoPad)
    }

    /// Creates a strict URL-safe Base64 decoder with padding.
    ///
    /// The decoder requires `reset` before streaming and does not allocate.
    pub fn url_safe() -> Self {
        Self::from_config(Base64Config::UrlSafe)
    }

    /// Creates a strict URL-safe Base64 decoder without padding.
    ///
    /// The decoder requires `reset` before streaming and does not allocate.
    pub fn url_safe_no_pad() -> Self {
        Self::from_config(Base64Config::UrlSafeNoPad)
    }
}

impl Default for Base64Decoder {
    fn default() -> Self {
        Self::standard()
    }
}
