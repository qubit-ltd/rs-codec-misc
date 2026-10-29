// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Lifecycle state shared by Base64 stream transcoders.

use qubit_codec::TranscodeFailure;

/// Lifecycle phase for a Base64 stream transcoder.
#[derive(Debug)]
pub(crate) enum Base64StreamPhase {
    Uninitialized,
    Active,
    Finished,
    Poisoned,
}

impl Base64StreamPhase {
    /// Checks whether another transcode operation is permitted.
    pub(crate) fn ensure_transcode(&self) -> Result<(), TranscodeFailure> {
        match self {
            Self::Uninitialized => Err(TranscodeFailure::TranscodeBeforeReset),
            Self::Active => Ok(()),
            Self::Finished => Err(TranscodeFailure::TranscodeAfterFinish),
            Self::Poisoned => Err(TranscodeFailure::LifecyclePoisoned),
        }
    }

    /// Checks whether a finish operation is permitted.
    pub(crate) fn ensure_finish(&self) -> Result<(), TranscodeFailure> {
        match self {
            Self::Uninitialized => Err(TranscodeFailure::FinishBeforeReset),
            Self::Active => Ok(()),
            Self::Finished => Err(TranscodeFailure::FinishAfterFinish),
            Self::Poisoned => Err(TranscodeFailure::LifecyclePoisoned),
        }
    }
}
