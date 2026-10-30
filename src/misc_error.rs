// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared codec error type.

use core::num::NonZeroUsize;

use qubit_codec::DecodeFailure;

mod misc_codec_error;

pub use self::misc_codec_error::MiscCodecError;

/// Result alias returned by codec operations.
///
/// # Type Parameters
///
/// - `T`: Value produced by a successful codec operation.
pub type MiscCodecResult<T> = Result<T, MiscCodecError>;

/// Maps a malformed value to a decode failure with a known input width.
///
/// # Parameters
///
/// - `error`: Codec-specific error describing the malformed value.
/// - `consumed`: Non-zero number of input units consumed by the malformed
///   value.
///
/// # Returns
///
/// Returns an invalid decode failure that retains `error` and records
/// `consumed` as the input width.
#[inline]
pub(crate) fn map_misc_decode_failure_with_consumed(
    error: MiscCodecError,
    consumed: NonZeroUsize,
) -> DecodeFailure<MiscCodecError> {
    DecodeFailure::invalid(error, consumed)
}
