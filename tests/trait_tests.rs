// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for lightweight encoder and decoder traits.

use qubit_codec::Codec;
use qubit_codec::CodecTranscodeDecoder;
use qubit_codec::CodecTranscodeEncoder;
use qubit_codec::CodecValueEncoder;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeDecoder;
use qubit_codec::TranscodeEncodeError;
use qubit_codec::TranscodeEncoder;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::engine::DecodeContext;
use qubit_codec::engine::DecodeInvalidAction;
use qubit_codec::engine::TranscodeDecodeEngine;
use qubit_codec::engine::TranscodeDecodeHooks;
use qubit_codec_misc::CStringLiteralCodec;
use qubit_codec_misc::FormUrlencodedCodec;
use qubit_codec_misc::HexByteCodec;
use qubit_codec_misc::HexCodec;
use qubit_codec_misc::MiscCodecError;
use qubit_codec_misc::PercentCodec;

#[derive(Debug)]
struct RecoveryHooks {
    replace: bool,
    spans: Vec<(usize, usize, usize)>,
    errors: Vec<RecoveryDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RecoveryDiagnostic {
    kind: &'static str,
    sample_index: Option<usize>,
}

#[derive(Debug, Eq, PartialEq)]
struct RecoveryResult {
    output: Vec<u8>,
    spans: Vec<(usize, usize, usize)>,
    errors: Vec<RecoveryDiagnostic>,
    failed: bool,
    need_input: usize,
    need_output: usize,
}

impl<C> TranscodeDecodeHooks<C> for RecoveryHooks
where
    C: Codec<Value = u8, Unit = u8, DecodeError = MiscCodecError>,
{
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut C,
        error: &MiscCodecError,
        consumed: Option<core::num::NonZeroUsize>,
        context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, TranscodeDecodeError<MiscCodecError>> {
        let consumed = consumed.expect("investigated codecs report known invalid spans");
        self.spans
            .push((context.input_index(), consumed.get(), context.available()));
        let (kind, index) = match error {
            MiscCodecError::InvalidDigit { index, .. } => ("InvalidDigit", Some(*index)),
            MiscCodecError::InvalidEscape { index, .. } => ("InvalidEscape", Some(*index)),
            MiscCodecError::InvalidCharacter { index, .. } => ("InvalidCharacter", Some(*index)),
            MiscCodecError::InvalidLength { .. } => ("InvalidLength", None),
            MiscCodecError::InvalidInput { .. } => ("InvalidInput", None),
            MiscCodecError::InvalidBase64 { input_index, .. } => ("InvalidBase64", *input_index),
            MiscCodecError::MissingPrefix { .. } => ("MissingPrefix", None),
            MiscCodecError::InvalidUtf8 { .. } => ("InvalidUtf8", None),
        };
        self.errors.push(RecoveryDiagnostic {
            kind,
            sample_index: index.map(|index| index.saturating_sub(1)),
        });
        Ok(if self.replace {
            DecodeInvalidAction::Emit { value: b'?', consumed }
        } else {
            DecodeInvalidAction::Skip { consumed }
        })
    }
}

fn run_recovery_chunks<C>(codec: C, input: &[u8], chunk_ends: &[usize], replace: bool) -> RecoveryResult
where
    C: Codec<Value = u8, Unit = u8, DecodeError = MiscCodecError>,
{
    let hooks = RecoveryHooks {
        replace,
        spans: Vec::new(),
        errors: Vec::new(),
    };
    let mut engine = TranscodeDecodeEngine::new(codec, hooks);
    let mut output = [0u8; 1024];
    let mut output_index = 1;
    output[0] = 0xa5;
    engine.reset(&mut output[..2], 1).expect("decoder reset");
    let mut input_index = 1;
    let mut failed = false;
    let mut need_input = 0;
    let mut need_output = 0;
    let mut iterations = 0;

    for end in chunk_ends {
        let visible_end = *end;
        loop {
            iterations += 1;
            assert!(iterations <= 1024, "recovery driver made no progress");
            let output_end = (output_index + 1).min(output.len());
            let progress = match engine.transcode(
                &input[..visible_end],
                input_index,
                &mut output[..output_end],
                output_index,
            ) {
                Ok(progress) => progress,
                Err(_) => {
                    failed = true;
                    break;
                }
            };
            input_index += progress.read();
            output_index += progress.written();
            match progress.status() {
                TranscodeStatus::NeedInput { .. } => {
                    need_input += 1;
                    break;
                }
                TranscodeStatus::NeedOutput { .. } => {
                    need_output += 1;
                    continue;
                }
                TranscodeStatus::Complete if input_index < visible_end => continue,
                TranscodeStatus::Complete => break,
            }
        }
        if failed {
            break;
        }
    }

    if !failed {
        loop {
            iterations += 1;
            assert!(iterations <= 1024, "EOF recovery driver made no progress");
            let output_end = (output_index + 1).min(output.len());
            let progress = match engine.transcode_eof(input, input_index, &mut output[..output_end], output_index) {
                Ok(progress) => progress,
                Err(_) => {
                    failed = true;
                    break;
                }
            };
            input_index += progress.read();
            output_index += progress.written();
            match progress.status() {
                TranscodeStatus::NeedInput { .. } => {
                    need_input += 1;
                    failed = true;
                    break;
                }
                TranscodeStatus::NeedOutput { .. } => {
                    need_output += 1;
                    continue;
                }
                TranscodeStatus::Complete if input_index < input.len() => continue,
                TranscodeStatus::Complete => break,
            }
        }
    }
    if !failed {
        engine.finish(&mut output, output_index).expect("decoder finish");
    }
    RecoveryResult {
        output: output[1..output_index].to_vec(),
        spans: engine.hooks().spans.clone(),
        errors: engine.hooks().errors.clone(),
        failed,
        need_input,
        need_output,
    }
}

fn strict_adapter_rejects_chunks<C>(codec: C, input: &[u8], chunk_ends: &[usize]) -> bool
where
    C: Codec<Value = u8, Unit = u8, DecodeError = MiscCodecError>,
{
    let mut decoder = CodecTranscodeDecoder::new(codec);
    let mut output = [0xa5; 32];
    decoder.reset(&mut [], 0).expect("strict decoder reset");
    let mut input_index = 1;
    let mut output_index = 1;
    let mut iterations = 0;

    for end in chunk_ends {
        loop {
            iterations += 1;
            assert!(iterations <= 1024, "strict chunk driver made no progress");
            match decoder.transcode(&input[..*end], input_index, &mut output, output_index) {
                Err(error) => return error.domain_ref().is_some(),
                Ok(progress) => {
                    input_index += progress.read();
                    output_index += progress.written();
                    match progress.status() {
                        TranscodeStatus::NeedInput { .. } => break,
                        TranscodeStatus::NeedOutput { .. } => continue,
                        TranscodeStatus::Complete if input_index < *end => continue,
                        TranscodeStatus::Complete => break,
                    }
                }
            }
        }
    }

    loop {
        iterations += 1;
        assert!(iterations <= 1024, "strict EOF driver made no progress");
        match decoder.transcode_eof(input, input_index, &mut output, output_index) {
            Err(error) => return error.domain_ref().is_some(),
            Ok(progress) => {
                input_index += progress.read();
                output_index += progress.written();
                match progress.status() {
                    TranscodeStatus::NeedInput { .. } => return false,
                    TranscodeStatus::NeedOutput { .. } => continue,
                    TranscodeStatus::Complete if input_index < input.len() => continue,
                    TranscodeStatus::Complete => return false,
                }
            }
        }
    }
}

fn assert_malformed_recovery_result(
    sample: &[u8],
    input: &[u8],
    ends: &[usize],
    replace: bool,
    outcome: &RecoveryResult,
) {
    let (trigger, limit, kind, sample_index) = match sample {
        b"%zA" => (2, 3, "InvalidEscape", 0),
        b"%AzB" => (3, 3, "InvalidEscape", 0),
        b"%A" => (3, 3, "InvalidEscape", 0),
        b"%" => (2, 3, "InvalidEscape", 0),
        br"\uZABC" => (3, 6, "InvalidDigit", 2),
        br"\u00ZGQ" => (5, 6, "InvalidDigit", 4),
        br"\U000000ZGQ" => (9, 10, "InvalidDigit", 8),
        br"\u12ABQ" => (3, 6, "InvalidEscape", 0),
        br"\xGQ" => (3, 2, "InvalidEscape", 0),
        _ => panic!("no explicit recovery expectation for {sample:?}"),
    };
    let available = ends
        .iter()
        .map(|end| end - 1)
        .find(|available| *available >= trigger)
        .expect("layout must expose the malformed fragment");
    let width = available.min(limit);
    let mut expected_output = Vec::new();
    if replace {
        expected_output.push(b'?');
    }
    expected_output.extend_from_slice(&input[1 + width..]);
    let context = format!("sample={sample:?} ends={ends:?} replace={replace}");
    assert!(!outcome.failed, "recovery failed: {context}");
    assert_eq!(vec![(1, width, available)], outcome.spans, "{context}");
    assert_eq!(
        vec![RecoveryDiagnostic { kind, sample_index: Some(sample_index) }],
        outcome.errors,
        "{context}"
    );
    assert_eq!(expected_output, outcome.output, "{context}");
}

fn check_recovery_splits<C>(factory: fn() -> C, samples: &[&[u8]])
where
    C: Codec<Value = u8, Unit = u8, DecodeError = MiscCodecError>,
{
    for sample in samples {
        let mut input = vec![b'!'];
        input.extend_from_slice(sample);
        input.push(b'!');
        let first = 1;
        let last = input.len();
        let mut layouts = vec![vec![last]];
        layouts.push(((first + 1)..=last).collect());
        for split in (first + 1)..last {
            layouts.push(vec![split, last]);
        }

        for replace in [false, true] {
            let mut observations = Vec::new();
            for ends in &layouts {
                if !matches!(*sample, br"\xA" | br"\7") {
                    assert!(
                        strict_adapter_rejects_chunks(factory(), &input, ends),
                        "strict adapter must reject malformed input for chunk layout {ends:?}"
                    );
                }
                let outcome = run_recovery_chunks(factory(), &input, ends, replace);
                if !matches!(*sample, br"\xA" | br"\7") {
                    assert_malformed_recovery_result(sample, &input, ends, replace, &outcome);
                }
                assert!(
                    outcome
                        .spans
                        .iter()
                        .all(|(index, width, available)| { index + width <= input.len() && width <= available })
                );
                observations.push(outcome);
            }
            let distinct = observations.iter().skip(1).any(|item| {
                item.output != observations[0].output
                    || item.spans != observations[0].spans
                    || item.errors != observations[0].errors
                    || item.failed != observations[0].failed
            });
            let need_input = observations.iter().map(|item| item.need_input).sum::<usize>();
            let need_output = observations.iter().map(|item| item.need_output).sum::<usize>();
            if *sample == b"%zA" {
                assert!(
                    distinct,
                    "malformed percent input should demonstrate chunk-dependent recovery"
                );
                assert!(need_input > 0, "chunked decoding must exercise NeedInput");
                assert!(need_output > 0, "one-slot output must exercise NeedOutput");
                assert!(observations.iter().all(|item| !item.failed && !item.errors.is_empty()));
            }
            if matches!(*sample, b"\\xA" | b"\\7") {
                let expected = if *sample == b"\\xA" { 0x0a } else { 0x07 };
                assert!(
                    observations.iter().all(|item| {
                        !item.failed
                            && item.errors.is_empty()
                            && item.spans.is_empty()
                            && item.output == [expected, b'!']
                    }),
                    "valid short C escape should decode consistently across chunk layouts"
                );
            }
            eprintln!(
                "recovery sample={:?} policy={} input_splits={} chunk_dependent={distinct} need_input={need_input} need_output={need_output} outcome={:?}",
                sample,
                if replace { "emit" } else { "skip" },
                layouts.len(),
                observations[0]
            );
        }
    }
}

#[test]
fn test_recovery_hints_are_bounded_and_exercised_across_chunks() {
    let percent_samples: &[&[u8]] = &[b"%zA", b"%AzB", b"%A", b"%"];
    check_recovery_splits(PercentCodec::new, percent_samples);
    check_recovery_splits(FormUrlencodedCodec::new, percent_samples);

    let c_string_samples: &[&[u8]] = &[
        br"\uZABC",
        br"\u00ZGQ",
        br"\U000000ZGQ",
        br"\u12ABQ",
        br"\xGQ",
        br"\xA",
        br"\7",
    ];
    check_recovery_splits(CStringLiteralCodec::new, c_string_samples);

    for sample in [b"%".as_slice(), b"%A".as_slice()] {
        let mut input = vec![b'!'];
        input.extend_from_slice(sample);
        let chunk_ends: Vec<usize> = (2..=input.len()).collect();
        for replace in [false, true] {
            let percent = run_recovery_chunks(PercentCodec::new(), &input, &chunk_ends, replace);
            let form = run_recovery_chunks(FormUrlencodedCodec::new(), &input, &chunk_ends, replace);
            for outcome in [&percent, &form] {
                assert!(!outcome.failed, "EOF recovery should complete");
                assert_eq!(1, outcome.errors.len());
                assert_eq!("InvalidEscape", outcome.errors[0].kind);
                assert_eq!(1, outcome.spans.len());
                assert_eq!((1, sample.len()), (outcome.spans[0].0, outcome.spans[0].1));
                assert!(outcome.need_input > 0);
                assert_eq!(if replace { vec![b'?'] } else { vec![] }, outcome.output);
            }
            eprintln!(
                "recovery EOF sample={sample:?} policy={} percent={percent:?} form={form:?}",
                if replace { "emit" } else { "skip" }
            );
        }
    }
}

#[test]
fn test_codec_types_can_be_used_through_traits() {
    let mut codec = HexCodec::new();
    let encoded = ValueEncoder::<[u8]>::encode(&mut codec, b"abc").expect("hex encode should succeed");
    let decoded = ValueDecoder::<str>::decode(&mut codec, &encoded).expect("hex decode should succeed");

    assert_eq!("616263", encoded);
    assert_eq!(b"abc".to_vec(), decoded);
}

#[test]
fn test_core_codec_adapter_types_can_wrap_misc_codecs() {
    fn assert_codec_value_encoder<
        T: ValueEncoder<u8, Output = Vec<u8>, Error = TranscodeEncodeError<MiscCodecError, u8>>,
    >() {
    }
    fn assert_codec_transcode_decoder<T: TranscodeDecoder<Input = u8, Output = u8>>() {}
    fn assert_codec_transcode_encoder<T: TranscodeEncoder<Input = u8, Output = u8>>() {}

    assert_codec_value_encoder::<CodecValueEncoder<HexByteCodec>>();
    assert_codec_transcode_decoder::<CodecTranscodeDecoder<HexByteCodec>>();
    assert_codec_transcode_encoder::<CodecTranscodeEncoder<HexByteCodec>>();

    let encode_error = TranscodeEncodeError::<core::convert::Infallible, u8>::unencodable(2, 0xff);
    assert!(matches!(encode_error, TranscodeEncodeError::Unencodable { .. }));
    let decode_error: TranscodeDecodeError<core::convert::Infallible> =
        TranscodeFailure::incomplete_input(2, 3, 1).into();
    assert!(matches!(
        decode_error,
        TranscodeDecodeError::Failure(TranscodeFailure::IncompleteInput { .. })
    ));
    let transcode_error: TranscodeDecodeError<core::convert::Infallible> =
        TranscodeFailure::invalid_input_index(2, 1).into();
    assert!(matches!(
        transcode_error,
        TranscodeDecodeError::Failure(TranscodeFailure::InvalidInputIndex { .. })
    ));
}

#[test]
fn test_hex_adapter_encoder_retries_small_output() {
    let mut encoder = CodecTranscodeEncoder::new(HexByteCodec::new());
    let mut output = [0xa5; 4];
    assert_eq!(0, encoder.reset(&mut [], 0).unwrap());
    let short = encoder.transcode(&[0, 0x1f], 1, &mut output[..2], 1).unwrap();
    assert_eq!(0, short.read());
    assert_eq!(0, short.written());
    assert!(matches!(short.status(), TranscodeStatus::NeedOutput { required } if required.get() == 2));
    assert_eq!([0xa5; 4], output);
    let done = encoder.transcode(&[0, 0x1f], 1, &mut output[..3], 1).unwrap();
    assert_eq!(1, done.read());
    assert_eq!(2, done.written());
    assert_eq!(TranscodeStatus::Complete, done.status());
    assert_eq!([0xa5, b'1', b'f', 0xa5], output);
    assert_eq!(0, encoder.finish(&mut output, 3).unwrap());
}

#[test]
fn test_hex_adapter_decoder_preserves_incomplete_tail_and_eof() {
    let mut decoder = CodecTranscodeDecoder::new(HexByteCodec::new());
    let mut output = [0xa5; 3];
    assert_eq!(0, decoder.reset(&mut [], 0).unwrap());
    let short = decoder.transcode(b"!1", 1, &mut output, 1).unwrap();
    assert_eq!(0, short.read());
    assert_eq!(0, short.written());
    assert!(matches!(short.status(), TranscodeStatus::NeedInput { required } if required.get() == 2));
    assert_eq!([0xa5; 3], output);
    let done = decoder.transcode_eof(b"!1f", 1, &mut output, 1).unwrap();
    assert_eq!(2, done.read());
    assert_eq!(1, done.written());
    assert_eq!(TranscodeStatus::Complete, done.status());
    assert_eq!([0xa5, 0x1f, 0xa5], output);
    assert_eq!(0, decoder.finish(&mut output, 2).unwrap());

    decoder.reset(&mut [], 0).unwrap();
    let error = decoder.transcode_eof(b"!1", 1, &mut output, 1).unwrap_err();
    assert!(matches!(
        error,
        TranscodeDecodeError::Failure(TranscodeFailure::IncompleteInput {
            input_index: 1,
            required: 2,
            available: 1
        })
    ));
    assert_eq!([0xa5, 0x1f, 0xa5], output);
}

fn assert_percent_adapter_rejects_impossible_prefix<C>(codec: C)
where
    C: Codec<Value = u8, Unit = u8, DecodeError = MiscCodecError>,
{
    let mut decoder = CodecTranscodeDecoder::new(codec);
    let mut output = [0xa5; 3];
    decoder.reset(&mut [], 0).unwrap();
    let error = decoder.transcode(b"!%z", 1, &mut output, 1).unwrap_err();
    assert!(matches!(error.domain_ref(), Some(MiscCodecError::InvalidEscape { .. })));
    assert_eq!([0xa5; 3], output);

    decoder.reset(&mut [], 0).unwrap();
    let progress = decoder.transcode(b"!%A", 1, &mut output, 1).unwrap();
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
    assert!(matches!(progress.status(), TranscodeStatus::NeedInput { required } if required.get() == 3));

    let progress = decoder.transcode_eof(b"!%A0", 1, &mut output, 1).unwrap();
    assert_eq!(3, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([0xa5, 0xa0, 0xa5], output);
}

#[test]
fn test_percent_adapters_reject_impossible_prefix_without_refill() {
    assert_percent_adapter_rejects_impossible_prefix(PercentCodec::new());
    assert_percent_adapter_rejects_impossible_prefix(FormUrlencodedCodec::new());
}

#[test]
fn test_c_universal_adapter_rejects_invalid_short_prefix() {
    let mut decoder = CodecTranscodeDecoder::new(CStringLiteralCodec::new());
    let mut output = [0xa5; 3];
    decoder.reset(&mut [], 0).unwrap();

    let error = decoder.transcode(br"\uZ", 0, &mut output, 0).unwrap_err();

    assert!(matches!(
        error.domain_ref(),
        Some(MiscCodecError::InvalidDigit {
            radix: 16,
            index: 2,
            character: 'Z'
        })
    ));
    assert_eq!([0xa5; 3], output);
}

#[test]
fn test_c_universal_adapter_rejects_impossible_numeric_prefix() {
    for input in [br"\u12".as_slice(), br"\U000001".as_slice()] {
        let mut decoder = CodecTranscodeDecoder::new(CStringLiteralCodec::new());
        let mut output = [0xa5; 2];
        decoder.reset(&mut output, 0).unwrap();
        let error = decoder.transcode(input, 0, &mut output, 0).unwrap_err();
        assert!(matches!(
            error.domain_ref(),
            Some(MiscCodecError::InvalidEscape { index: 0, .. })
        ));
        assert_eq!([0xa5; 2], output);
    }
}

#[test]
fn test_safe_hex_adapter_readme() {
    let mut decoder = CodecTranscodeDecoder::new(HexByteCodec::new());
    let chunks: [&[u8]; 3] = [b"1", b"f8", b"b"];
    let mut pending = Vec::new();
    let mut output = [0_u8; 2];
    let mut written = 0;

    decoder.reset(&mut output, 0).expect("reset decoder");
    for (index, chunk) in chunks.iter().enumerate() {
        pending.extend_from_slice(chunk);
        let progress = decoder
            .transcode(&pending, 0, &mut output, written)
            .expect("decode available input");
        let read = progress.read();
        written += progress.written();
        pending.drain(..read);
        assert!(!matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
        if index < 2 {
            assert!(matches!(progress.status(), TranscodeStatus::NeedInput { .. }));
        } else {
            assert_eq!(TranscodeStatus::Complete, progress.status());
        }
    }

    // Complete says this call consumed its visible input; only the source tells us
    // it has ended.
    assert!(pending.is_empty());
    let eof = decoder
        .transcode_eof(&pending, 0, &mut output, written)
        .expect("finish decoding at confirmed EOF");
    assert_eq!(TranscodeStatus::Complete, eof.status());
    written += eof.written();
    assert_eq!([0x1f, 0x8b], output);
    assert_eq!(2, written);
    assert_eq!(0, decoder.finish(&mut output, written).expect("finish adapter"));

    output = [0xa5; 2];
    decoder.reset(&mut output, 0).expect("reset for short input");
    let error = decoder.transcode_eof(b"1", 0, &mut output, 0).unwrap_err();
    assert!(matches!(
        error,
        TranscodeDecodeError::Failure(TranscodeFailure::IncompleteInput {
            input_index: 0,
            required: 2,
            available: 1
        })
    ));
    assert_eq!([0xa5; 2], output);

    decoder.reset(&mut output, 0).expect("reset for invalid input");
    let error = decoder.transcode(b"1g", 0, &mut output, 0).unwrap_err();
    assert!(matches!(
        error.domain_ref(),
        Some(qubit_codec_misc::MiscCodecError::InvalidDigit { index: 1, .. })
    ));
    assert_eq!([0xa5; 2], output);
}

#[test]
fn test_value_traits_accept_text_codecs() {
    fn roundtrip<C>(mut codec: C, text: &str) -> String
    where
        C: ValueEncoder<str, Output = String, Error = MiscCodecError>
            + ValueDecoder<str, Output = String, Error = MiscCodecError>,
    {
        let encoded = ValueEncoder::<str>::encode(&mut codec, text).expect("text should encode");
        ValueDecoder::<str>::decode(&mut codec, &encoded).expect("text should decode")
    }

    assert_eq!("a b", roundtrip(PercentCodec::new(), "a b"));
    assert_eq!("a b", roundtrip(FormUrlencodedCodec::new(), "a b"));
}
