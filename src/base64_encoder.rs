// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Streaming Base64 encoder configuration and state.

use crate::internal::{Base64Config, Base64StreamPhase};

/// Encodes bytes incrementally using a strict Base64 configuration.
///
/// The encoder uses fixed-size state and does not allocate. Call `reset`
/// before streaming. Input and output buffers remain caller-owned.
#[must_use]
#[derive(Debug)]
pub struct Base64Encoder {
    pub(crate) config: Base64Config,
    pub(crate) phase: Base64StreamPhase,
    pub(crate) tail: [u8; 2],
    pub(crate) tail_len: usize,
}

impl Base64Encoder {
    pub(crate) fn from_config(config: Base64Config) -> Self {
        Self {
            config,
            phase: Base64StreamPhase::Uninitialized,
            tail: [0; 2],
            tail_len: 0,
        }
    }

    /// Creates a strict standard Base64 encoder with padding.
    ///
    /// The encoder requires `reset` before streaming and does not allocate.
    pub fn standard() -> Self {
        Self::from_config(Base64Config::Standard)
    }

    /// Creates a strict standard Base64 encoder without padding.
    ///
    /// The encoder requires `reset` before streaming and does not allocate.
    pub fn standard_no_pad() -> Self {
        Self::from_config(Base64Config::StandardNoPad)
    }

    /// Creates a strict URL-safe Base64 encoder with padding.
    ///
    /// The encoder requires `reset` before streaming and does not allocate.
    pub fn url_safe() -> Self {
        Self::from_config(Base64Config::UrlSafe)
    }

    /// Creates a strict URL-safe Base64 encoder without padding.
    ///
    /// The encoder requires `reset` before streaming and does not allocate.
    pub fn url_safe_no_pad() -> Self {
        Self::from_config(Base64Config::UrlSafeNoPad)
    }
}

impl Default for Base64Encoder {
    fn default() -> Self {
        Self::standard()
    }
}
