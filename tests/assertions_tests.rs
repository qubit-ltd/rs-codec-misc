// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::DecodeFailure;
use qubit_codec_misc::MiscCodecError;

pub(crate) fn invalid_source(
    failure: DecodeFailure<MiscCodecError>,
) -> MiscCodecError {
    match failure {
        DecodeFailure::Invalid { source, .. } => source,
        other => {
            panic!("expected invalid misc codec decode failure: {other:?}")
        }
    }
}

/// Returns the invalid-input consumption hint from a decode failure.
pub(crate) fn invalid_consumed(
    failure: DecodeFailure<MiscCodecError>,
) -> Option<usize> {
    match failure {
        DecodeFailure::Invalid { consumed, .. } => {
            consumed.map(|width| width.get())
        }
        other => {
            panic!("expected invalid misc codec decode failure: {other:?}")
        }
    }
}

/// Extracts the retry size from a low-level incomplete decode failure.
pub(crate) fn incomplete_required(
    failure: DecodeFailure<MiscCodecError>,
) -> usize {
    match failure {
        DecodeFailure::Incomplete { required_total, .. } => {
            required_total.get()
        }
        other => {
            panic!("expected incomplete misc codec decode failure: {other:?}")
        }
    }
}
