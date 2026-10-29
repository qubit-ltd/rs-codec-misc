// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Streaming Base64 encoder configuration and state.

use crate::internal::{Base64Config, Base64StreamPhase};
use base64::Engine;
use core::convert::Infallible;
use core::num::NonZeroUsize;
use qubit_codec::{
    CapacityError, TranscodeEncodeError, TranscodeFailure, TranscodeProgress, TranscodeEncoder,
    Transcoder,
};

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

impl TranscodeEncoder for Base64Encoder {
    type EncodeError = Infallible;
}

impl Transcoder for Base64Encoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeEncodeError<Infallible, u8>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        crate::internal::base64_encode_bound(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(if self.config.padded() { 4 } else { 3 })
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        self.tail = [0; 2];
        self.tail_len = 0;
        self.phase = Base64StreamPhase::Active;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;
        self.phase.ensure_transcode()?;
        let mut i = input_index;
        let mut o = output_index;
        while i < input.len() {
            if self.tail_len > 0 {
                while self.tail_len < 2 && i < input.len() {
                    self.tail[self.tail_len] = input[i];
                    self.tail_len += 1;
                    i += 1;
                }
                if i == input.len() {
                    break;
                }
                if output.len() - o < 4 {
                    return Ok(TranscodeProgress::need_output(
                        NonZeroUsize::new(4).expect("four is nonzero"),
                        i - input_index,
                        o - output_index,
                    ));
                }
                let quantum = [self.tail[0], self.tail[1], input[i]];
                let written = self.config.engine().encode_slice(quantum, &mut output[o..o + 4])
                    .expect("four output bytes fit one full quantum");
                debug_assert_eq!(written, 4);
                self.tail_len = 0;
                i += 1;
                o += written;
                continue;
            }
            let groups = ((input.len() - i) / 3).min((output.len() - o) / 4);
            if groups > 0 {
                let consumed = groups * 3;
                let emitted = groups * 4;
                let written = self.config.engine().encode_slice(
                    &input[i..i + consumed],
                    &mut output[o..o + emitted],
                ).expect("preflight reserves four bytes per full quantum");
                debug_assert_eq!(written, emitted);
                i += consumed;
                o += written;
                continue;
            }
            let retained = (input.len() - i).min(2);
            self.tail[..retained].copy_from_slice(&input[i..i + retained]);
            self.tail_len = retained;
            i += retained;
        }
        Ok(TranscodeProgress::complete(i - input_index, o - output_index))
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        self.phase.ensure_finish()?;
        let needed = if self.tail_len == 0 {
            0
        } else if self.config.padded() {
            4
        } else {
            self.tail_len + 1
        };
        TranscodeFailure::ensure_output_capacity(output.len(), output_index, needed)?;
        let written = self.config.engine()
            .encode_slice(&self.tail[..self.tail_len], &mut output[output_index..output_index + needed])
            .expect("preflight reserves the exact final encoded length");
        self.tail = [0; 2];
        self.tail_len = 0;
        self.phase = Base64StreamPhase::Finished;
        Ok(written)
    }
}
