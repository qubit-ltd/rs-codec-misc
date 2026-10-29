// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared static configuration for Base64 codecs and transcoders.

use base64::engine::GeneralPurpose;
use base64::engine::general_purpose::STANDARD;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::engine::general_purpose::URL_SAFE;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use qubit_codec::CapacityError;

static STANDARD_ENGINE: GeneralPurpose = STANDARD;
static STANDARD_NO_PAD_ENGINE: GeneralPurpose = STANDARD_NO_PAD;
static URL_SAFE_ENGINE: GeneralPurpose = URL_SAFE;
static URL_SAFE_NO_PAD_ENGINE: GeneralPurpose = URL_SAFE_NO_PAD;

/// The supported strict Base64 alphabets and padding policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Base64Config {
    /// Standard alphabet with required padding for partial groups.
    Standard,
    /// Standard alphabet without output padding.
    StandardNoPad,
    /// URL-safe alphabet with required padding for partial groups.
    UrlSafe,
    /// URL-safe alphabet without output padding.
    UrlSafeNoPad,
}

impl Base64Config {
    /// Returns the static engine for this configuration.
    pub(crate) fn engine(self) -> &'static GeneralPurpose {
        match self {
            Self::Standard => &STANDARD_ENGINE,
            Self::StandardNoPad => &STANDARD_NO_PAD_ENGINE,
            Self::UrlSafe => &URL_SAFE_ENGINE,
            Self::UrlSafeNoPad => &URL_SAFE_NO_PAD_ENGINE,
        }
    }

    /// Returns whether this configuration requires padding for partial groups.
    pub(crate) fn padded(self) -> bool {
        matches!(self, Self::Standard | Self::UrlSafe)
    }
}

/// Returns a checked upper bound for encoding `n` bytes.
pub(crate) fn base64_encode_bound(n: usize) -> Result<usize, CapacityError> {
    (n / 3)
        .checked_add(usize::from(!n.is_multiple_of(3)))
        .and_then(|groups| groups.checked_mul(4))
        .ok_or(CapacityError::OutputLengthOverflow)
}

/// Returns a checked upper bound for decoding `n` Base64 bytes.
pub(crate) fn base64_decode_bound(n: usize) -> Result<usize, CapacityError> {
    (n / 4)
        .checked_add(usize::from(!n.is_multiple_of(4)))
        .and_then(|groups| groups.checked_mul(3))
        .ok_or(CapacityError::OutputLengthOverflow)
}
