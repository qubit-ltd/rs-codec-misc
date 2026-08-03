// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
// =============================================================================
//! Private parser outcome used by streaming codecs.

use core::num::NonZeroUsize;

use crate::MiscCodecError;
use qubit_codec::DecodeFailure;

/// Distinguishes retryable stream tails from malformed codec input.
#[derive(Debug)]
pub(crate) enum ParseError {
    /// More input is required before a value can be decided.
    Incomplete {
        /// Total units required from the current value start.
        required: NonZeroUsize,
    },
    /// The input is malformed.
    Invalid(MiscCodecError),
}

impl From<MiscCodecError> for ParseError {
    #[inline]
    fn from(error: MiscCodecError) -> Self {
        Self::Invalid(error)
    }
}

impl ParseError {
    /// Converts a parser outcome into the low-level codec failure type.
    #[inline]
    pub(crate) fn into_decode_failure(self) -> DecodeFailure<MiscCodecError> {
        match self {
            Self::Incomplete { required, .. } => {
                DecodeFailure::incomplete(required)
            }
            Self::Invalid(error) => DecodeFailure::invalid_unknown(error),
        }
    }
}
