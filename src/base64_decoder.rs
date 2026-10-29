// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Streaming Base64 decoder configuration and state.

use base64::{DecodeError, DecodeSliceError, Engine};
use core::num::NonZeroUsize;
use qubit_codec::{
    CapacityError, TranscodeDecodeError, TranscodeDecoder, TranscodeFailure, TranscodeProgress,
    Transcoder,
};

use crate::internal::{base64_decode_bound, Base64Config, Base64StreamPhase};
use crate::MiscCodecError;

/// Decodes Base64 incrementally using a strict configuration.
///
/// The decoder uses fixed-size state and does not allocate. Call `reset`
/// before streaming. Incomplete non-EOF input remains caller-owned and must be
/// supplied again with additional input or to `transcode_eof`.
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

    fn domain_error(
        &mut self,
        source: DecodeError,
        segment_start: usize,
    ) -> TranscodeDecodeError<MiscCodecError> {
        let relative = match &source {
            DecodeError::InvalidByte(index, _) => *index,
            DecodeError::InvalidLastSymbol { offset, .. } => *offset,
            DecodeError::InvalidLength(_) | DecodeError::InvalidPadding => 0,
        };
        self.phase = Base64StreamPhase::Poisoned;
        TranscodeDecodeError::domain_main(MiscCodecError::from(source), segment_start + relative)
    }

    fn decode_segment(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
        eof: bool,
    ) -> Result<TranscodeProgress, TranscodeDecodeError<MiscCodecError>> {
        TranscodeFailure::ensure_transcode_indices(
            input.len(),
            input_index,
            output.len(),
            output_index,
        )?;
        self.phase.ensure_transcode()?;
        let mut i = input_index;
        let mut o = output_index;

        if self.terminal_padding && i < input.len() {
            return Err(self.domain_error(DecodeError::InvalidByte(0, input[i]), i));
        }

        if eof && !self.terminal_padding {
            if let Ok(bound) = base64_decode_bound(input.len() - i) {
                if output.len() - o >= bound {
                    let written = match self.config.engine().decode_slice(&input[i..], &mut output[o..]) {
                        Ok(written) => written,
                        Err(DecodeSliceError::DecodeError(source)) => {
                            return Err(self.domain_error(source, i));
                        }
                        // The checked bound covers the engine's conservative
                        // decode estimate for the full input suffix.
                        Err(DecodeSliceError::OutputSliceTooSmall) => {
                            unreachable!("decoded upper bound covers engine estimate")
                        }
                    };
                    self.terminal_padding = input[i..].contains(&b'=');
                    return Ok(TranscodeProgress::complete(input.len() - i, written));
                }
            }
        }

        while i < input.len() {
            if self.terminal_padding {
                return Err(self.domain_error(DecodeError::InvalidByte(0, input[i]), i));
            }

            let remaining = input.len() - i;
            if remaining < 4 && !eof {
                return Ok(TranscodeProgress::need_input(
                    NonZeroUsize::new(4).expect("four is nonzero"),
                    i - input_index,
                    o - output_index,
                ));
            }

            let groups_by_capacity = (remaining / 4).min((output.len() - o) / 3);
            let candidate_len = groups_by_capacity * 4;
            let groups_without_padding = input[i..i + candidate_len]
                .iter()
                .position(|byte| *byte == b'=')
                .map_or(groups_by_capacity, |at| at / 4);

            if groups_without_padding > 0 {
                let take = groups_without_padding * 4;
                let capacity = groups_without_padding * 3;
                let written = match self.config.engine().decode_slice(
                    &input[i..i + take],
                    &mut output[o..o + capacity],
                ) {
                    Ok(written) => written,
                    Err(DecodeSliceError::DecodeError(source)) => {
                        return Err(self.domain_error(source, i));
                    }
                    // Each complete unpadded quantum decodes to exactly three
                    // bytes, which the selected output range holds.
                    Err(DecodeSliceError::OutputSliceTooSmall) => {
                        unreachable!("three output bytes fit each full quantum")
                    }
                };
                debug_assert_eq!(written, capacity);
                i += take;
                o += written;
                continue;
            }

            let take = remaining.min(4);
            let segment = &input[i..i + take];
            let mut scratch = [0; 3];
            let written = match self.config.engine().decode_slice(segment, &mut scratch) {
                Ok(written) => written,
                Err(DecodeSliceError::DecodeError(source)) => {
                    return Err(self.domain_error(source, i));
                }
                // A Base64 quantum can produce at most three bytes.
                Err(DecodeSliceError::OutputSliceTooSmall) => {
                    unreachable!("three-byte scratch holds one Base64 quantum")
                }
            };

            if output.len() - o < written {
                return Ok(TranscodeProgress::need_output(
                    NonZeroUsize::new(written).expect("valid nonempty quantum produces bytes"),
                    i - input_index,
                    o - output_index,
                ));
            }
            output[o..o + written].copy_from_slice(&scratch[..written]);
            self.terminal_padding = segment.contains(&b'=');
            i += take;
            o += written;
        }

        Ok(TranscodeProgress::complete(
            i - input_index,
            o - output_index,
        ))
    }
}

impl Default for Base64Decoder {
    fn default() -> Self {
        Self::standard()
    }
}

impl TranscodeDecoder for Base64Decoder {
    type DecodeError = MiscCodecError;
}

impl Transcoder for Base64Decoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeDecodeError<MiscCodecError>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        base64_decode_bound(input_len)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        self.phase = Base64StreamPhase::Active;
        self.terminal_padding = false;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.decode_segment(input, input_index, output, output_index, false)
    }

    fn transcode_eof(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.decode_segment(input, input_index, output, output_index, true)
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        self.phase.ensure_finish()?;
        self.phase = Base64StreamPhase::Finished;
        Ok(0)
    }
}
