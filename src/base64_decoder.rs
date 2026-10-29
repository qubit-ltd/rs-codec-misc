// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Streaming Base64 decoder configuration and state.

use core::num::NonZeroUsize;

use base64::DecodeError;
use base64::DecodeSliceError;
use base64::Engine;
use qubit_codec::CapacityError;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeDecoder;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeProgress;
use qubit_codec::Transcoder;

use crate::MiscCodecError;
use crate::internal::Base64Config;
use crate::internal::Base64StreamPhase;
use crate::internal::base64_decode_bound;

/// Decodes Base64 incrementally using a strict configuration.
///
/// The decoder uses fixed-size state and does not allocate. Call `reset`
/// before streaming. Incomplete non-EOF input remains caller-owned and must be
/// supplied again with additional input or to `transcode_eof`. `Complete`
/// describes the currently visible input and does not establish source EOF;
/// call `transcode_eof` only after the source confirms EOF. A domain error may
/// occur after earlier bytes in the same operation were written, and poisons
/// the decoder; call `reset` before reusing it. If output capacity is
/// insufficient, use the returned progress to retry with unconsumed input and
/// a larger output buffer. `NeedOutput.required` is the minimum total capacity
/// from the advanced output position (`output_index + progress.written()`) for
/// the next step. When retaining the output buffer, its length must cover that
/// position plus `required`; when switching to an empty buffer at index zero,
/// provide at least `required` slots.
///
/// The outer transcoder error reports a position in the input slice passed to
/// the current call. Its `MiscCodecError` and `base64::DecodeError` source keep
/// the engine sub-slice position, which can vary with batching, EOF handling,
/// and output capacity. Errors without an exact engine position use the
/// failing segment's start in the outer error. These positions are not
/// cumulative stream offsets.
///
/// # Examples
///
/// ```
/// use qubit_codec::TranscodeStatus;
/// use qubit_codec::Transcoder;
/// use qubit_codec_misc::Base64Decoder;
///
/// let mut decoder = Base64Decoder::standard();
/// let mut output = [0; 3];
/// decoder.reset(&mut output, 0).unwrap();
/// let progress = decoder.transcode(b"TQ", 0, &mut output, 0).unwrap();
/// assert_eq!(progress.read(), 0);
/// assert!(matches!(progress.status(), TranscodeStatus::NeedInput { .. }));
/// // Keep the unconsumed TQ, append ==, and confirm EOF with the source.
/// let progress = decoder.transcode_eof(b"TQ==", 0, &mut output, 0).unwrap();
/// assert_eq!(progress.read(), 4);
/// assert_eq!(&output[..progress.written()], b"M");
/// assert_eq!(decoder.finish(&mut [], 0).unwrap(), 0);
/// ```
#[must_use]
#[derive(Debug)]
pub struct Base64Decoder {
    /// Strict alphabet and padding policy used for decoding.
    pub(crate) config: Base64Config,
    /// Lifecycle state; domain errors poison the stream until reset.
    pub(crate) phase: Base64StreamPhase,
    /// Whether a successfully committed quantum contained terminal padding.
    pub(crate) terminal_padding: bool,
}

impl Base64Decoder {
    /// Creates a decoder for `config` in the uninitialized phase.
    ///
    /// Callers must reset the returned decoder before streaming.
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

    /// Converts an engine error to the stream error, mapping its relative
    /// position to the current call's input slice in the outer error while
    /// preserving the engine sub-slice position in the domain error, then
    /// poisons this stream.
    ///
    /// Errors without a precise engine position use the start of the current
    /// segment, matching the existing decode error mapping.
    fn domain_error(&mut self, source: DecodeError, segment_start: usize) -> TranscodeDecodeError<MiscCodecError> {
        let relative = match &source {
            DecodeError::InvalidByte(index, _) => *index,
            DecodeError::InvalidLastSymbol { offset, .. } => *offset,
            DecodeError::InvalidLength(_) | DecodeError::InvalidPadding => 0,
        };
        self.phase = Base64StreamPhase::Poisoned;
        TranscodeDecodeError::domain_main(MiscCodecError::from(source), segment_start + relative)
    }

    /// Decodes one visible input segment, optionally treating its end as EOF.
    ///
    /// At EOF, a sufficiently large output buffer permits a whole-suffix
    /// engine call. Otherwise complete unpadded groups are decoded before a
    /// final quantum is handled separately. Errors can follow writes from
    /// earlier groups; a capacity failure on the final quantum leaves that
    /// quantum unconsumed and its decoder state uncommitted.
    fn decode_segment(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
        eof: bool,
    ) -> Result<TranscodeProgress, TranscodeDecodeError<MiscCodecError>> {
        TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;
        self.phase.ensure_transcode()?;
        let mut i = input_index;
        let mut o = output_index;

        if self.terminal_padding && i < input.len() {
            return Err(self.domain_error(DecodeError::InvalidByte(0, input[i]), i));
        }

        if eof
            && !self.terminal_padding
            && let Ok(bound) = base64_decode_bound(input.len() - i)
            && output.len() - o >= bound
        {
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
                let written = match self
                    .config
                    .engine()
                    .decode_slice(&input[i..i + take], &mut output[o..o + capacity])
                {
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

        Ok(TranscodeProgress::complete(i - input_index, o - output_index))
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
