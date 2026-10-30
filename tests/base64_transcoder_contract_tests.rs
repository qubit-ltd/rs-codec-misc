// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use base64::DecodeError;
use qubit_codec::CapacityError;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeDecoder;
use qubit_codec::TranscodeEncoder;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec_misc::Base64Codec;
use qubit_codec_misc::Base64Decoder;
use qubit_codec_misc::Base64Encoder;
use qubit_codec_misc::Base64ErrorKind;
use qubit_codec_misc::MiscCodecError;

/// Checks at compile time that the encoder exposes the expected byte types.
fn require_encoder<T: TranscodeEncoder<Input = u8, Output = u8>>(_: &T) {}

/// Checks at compile time that the decoder exposes the expected byte types.
fn require_decoder<T: TranscodeDecoder<Input = u8, Output = u8>>(_: &T) {}

#[test]
fn test_base64_transcoder_bounds_and_direction_markers() {
    let encoder = Base64Encoder::standard();
    let decoder = Base64Decoder::standard();
    require_encoder(&encoder);
    require_decoder(&decoder);
    for n in 0..=12 {
        assert_eq!(encoder.max_transcode_output_len(n).unwrap(), n.div_ceil(3) * 4);
        assert_eq!(decoder.max_transcode_output_len(n).unwrap(), n.div_ceil(4) * 3);
    }
    assert_eq!(encoder.max_total_output_len(0).unwrap(), 4);
    assert_eq!(Base64Encoder::standard_no_pad().max_total_output_len(0).unwrap(), 3);
    assert_eq!(decoder.max_total_output_len(0).unwrap(), 0);
    assert_eq!(
        encoder.max_transcode_output_len(usize::MAX),
        Err(CapacityError::OutputLengthOverflow)
    );
    let n = usize::MAX;
    assert_eq!(decoder.max_transcode_output_len(n).unwrap(), (n / 4) * 3 + 3);
}

#[test]
fn test_base64_transcoder_encode_bound_distinguishes_main_and_total_overflow() {
    let padded = Base64Encoder::standard();
    let unpadded = Base64Encoder::standard_no_pad();
    let groups = usize::MAX / 4;
    let n = groups * 3;
    assert_eq!(padded.max_transcode_output_len(n).unwrap(), groups * 4);
    assert_eq!(padded.max_total_output_len(n), Err(CapacityError::OutputLengthOverflow));
    assert_eq!(unpadded.max_total_output_len(n).unwrap(), usize::MAX);
    assert_eq!(
        padded.max_transcode_output_len(n + 1),
        Err(CapacityError::OutputLengthOverflow)
    );
}

/// Drives one input segment until the decoder reaches an input or output
/// boundary.
fn drive_decode_segment(
    decoder: &mut Base64Decoder,
    input: &[u8],
    eof: bool,
    first_capacity: usize,
    decoded: &mut Vec<u8>,
) -> usize {
    let mut cursor = 0;
    let mut capacity = first_capacity;
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(
            steps <= input.len() * 2 + 8,
            "decoder must make progress or return a boundary"
        );
        let mut scratch = [0xa5; 3];
        let progress = if eof {
            decoder.transcode_eof(input, cursor, &mut scratch[..capacity], 0)
        } else {
            decoder.transcode(input, cursor, &mut scratch[..capacity], 0)
        }
        .expect("valid encoded input");
        progress
            .validate(cursor, input.len() - cursor, 0, capacity)
            .expect("valid progress");
        cursor += progress.read();
        decoded.extend_from_slice(&scratch[..progress.written()]);
        match progress.status() {
            TranscodeStatus::Complete => {
                assert_eq!(cursor, input.len());
                return cursor;
            }
            TranscodeStatus::NeedInput { .. } => {
                assert!(!eof, "EOF must resolve or reject tail");
                return cursor;
            }
            TranscodeStatus::NeedOutput { required } => {
                capacity = required.get();
                assert!((1..=3).contains(&capacity));
            }
        }
    }
}

/// Drives one decoder segment while preserving domain errors for contract
/// checks.
fn drive_decode_part(
    decoder: &mut Base64Decoder,
    input: &[u8],
    eof: bool,
    first_capacity: usize,
    decoded: &mut Vec<u8>,
) -> Result<usize, TranscodeDecodeError<MiscCodecError>> {
    let mut cursor = 0;
    let mut capacity = first_capacity;
    for _ in 0..input.len() * 2 + 8 {
        let mut scratch = vec![0xa5; capacity + 2];
        let result = if eof {
            decoder.transcode_eof(input, cursor, &mut scratch[1..1 + capacity], 0)
        } else {
            decoder.transcode(input, cursor, &mut scratch[1..1 + capacity], 0)
        };
        assert_eq!(scratch[0], 0xa5);
        assert_eq!(scratch[capacity + 1], 0xa5);
        let progress = result?;
        progress.validate(cursor, input.len() - cursor, 0, capacity).unwrap();
        cursor += progress.read();
        decoded.extend_from_slice(&scratch[1..1 + progress.written()]);
        match progress.status() {
            TranscodeStatus::Complete => {
                assert_eq!(cursor, input.len());
                return Ok(cursor);
            }
            TranscodeStatus::NeedInput { .. } => {
                assert!(!eof, "EOF must resolve or reject the retained tail");
                return Ok(cursor);
            }
            TranscodeStatus::NeedOutput { required } => {
                capacity = required.get();
                assert!((1..=3).contains(&capacity));
            }
        }
    }
    panic!("decoder did not reach a boundary within the step limit");
}

#[test]
fn test_base64_transcoder_decode_every_split_and_small_output() {
    let factories: [fn() -> Base64Codec; 4] = [
        Base64Codec::standard,
        Base64Codec::standard_no_pad,
        Base64Codec::url_safe,
        Base64Codec::url_safe_no_pad,
    ];
    for make in factories {
        let codec = make();
        for len in 0..=24 {
            let payload: Vec<u8> = (0..len).map(|i| (i * 37 + 251) as u8).collect();
            let text = codec.encode(&payload);
            for split in 0..=text.len() {
                for first_capacity in 0..=3 {
                    let mut decoder = codec.decoder();
                    decoder.reset(&mut [], 0).unwrap();
                    let mut actual = Vec::new();
                    let consumed = drive_decode_segment(
                        &mut decoder,
                        &text.as_bytes()[..split],
                        false,
                        first_capacity,
                        &mut actual,
                    );
                    let mut final_input = text.as_bytes()[consumed..split].to_vec();
                    final_input.extend_from_slice(&text.as_bytes()[split..]);
                    drive_decode_segment(&mut decoder, &final_input, true, first_capacity, &mut actual);
                    assert_eq!(decoder.finish(&mut [], 0).unwrap(), 0);
                    assert_eq!(actual, payload, "len={len} split={split} capacity={first_capacity}");
                }
            }
        }
    }
}

/// Encodes a segment, retrying with the encoder-reported quantum capacity.
fn drive_encode_segment(encoder: &mut Base64Encoder, input: &[u8], first_capacity: usize, encoded: &mut Vec<u8>) {
    let mut cursor = 0;
    let mut capacity = first_capacity;
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps <= input.len() * 2 + 8, "encoder must make progress");
        let mut scratch = [0xa5; 4];
        let p = encoder.transcode(input, cursor, &mut scratch[..capacity], 0).unwrap();
        p.validate(cursor, input.len() - cursor, 0, capacity).unwrap();
        cursor += p.read();
        encoded.extend_from_slice(&scratch[..p.written()]);
        match p.status() {
            TranscodeStatus::Complete => {
                assert_eq!(cursor, input.len());
                return;
            }
            TranscodeStatus::NeedInput { .. } => panic!("TranscodeEncoder must never need input"),
            TranscodeStatus::NeedOutput { required } => {
                assert_eq!(required.get(), 4);
                capacity = 4;
            }
        }
    }
}

#[test]
fn test_base64_transcoder_encode_every_split_and_small_output() {
    let factories: [fn() -> Base64Codec; 4] = [
        Base64Codec::standard,
        Base64Codec::standard_no_pad,
        Base64Codec::url_safe,
        Base64Codec::url_safe_no_pad,
    ];
    for make in factories {
        let codec = make();
        for len in 0..=24 {
            let payload: Vec<u8> = (0..len).map(|i| (i * 37 + 251) as u8).collect();
            for split in 0..=payload.len() {
                for capacity in 0..=4 {
                    let mut encoder = codec.encoder();
                    encoder.reset(&mut [], 0).unwrap();
                    let mut actual = Vec::new();
                    drive_encode_segment(&mut encoder, &payload[..split], capacity, &mut actual);
                    drive_encode_segment(&mut encoder, &payload[split..], capacity, &mut actual);
                    let mut tail = [0; 4];
                    let n = encoder.finish(&mut tail, 0).unwrap();
                    actual.extend_from_slice(&tail[..n]);
                    assert_eq!(actual, codec.encode(&payload).as_bytes());
                }
            }
        }
    }
}

#[test]
fn test_base64_transcoder_complete_preflight_preserves_existing_stream() {
    let mut encoder = Base64Encoder::standard();
    encoder.reset(&mut [], 0).unwrap();
    let _ = encoder.transcode(b"M", 0, &mut [], 0).unwrap();
    let mut short = [0xa5; 7];
    let error = encoder.transcode_complete_into(b"a", &mut short).unwrap_err();
    assert!(matches!(
        error.failure_ref(),
        Some(TranscodeFailure::InsufficientOutput {
            required: 8,
            available: 7,
            ..
        })
    ));
    assert_eq!(short, [0xa5; 7]);
    let mut out = [0; 4];
    assert_eq!(encoder.finish(&mut out, 0).unwrap(), 4);
    assert_eq!(&out, b"TQ==");

    let mut decoder = Base64Decoder::standard();
    decoder.reset(&mut [], 0).unwrap();
    let _ = decoder.transcode(b"TQ==", 0, &mut [0; 1], 0).unwrap();
    let mut too_small = [0xa5; 2];
    let error = decoder.transcode_complete_into(b"M", &mut too_small).unwrap_err();
    assert!(matches!(
        error.failure_ref(),
        Some(TranscodeFailure::InsufficientOutput { .. })
    ));
    assert_eq!(too_small, [0xa5; 2]);
    let error = decoder.transcode(b"TWFu", 0, &mut [0; 3], 0).unwrap_err();
    assert!(error.is_domain(), "preflight must retain terminal padding state");
}

#[test]
fn test_base64_transcoder_invalid_indices_preserve_state_and_end_indices_are_valid() {
    let mut encoder = Base64Encoder::standard();
    encoder.reset(&mut [], 0).unwrap();
    let _ = encoder.transcode(b"M", 0, &mut [], 0).unwrap();
    assert!(encoder.transcode(b"a", 2, &mut [], 0).is_err());
    let mut out = [0; 4];
    assert_eq!(encoder.finish(&mut out, 0).unwrap(), 4);
    assert_eq!(&out, b"TQ==");

    let mut decoder = Base64Decoder::standard();
    decoder.reset(&mut [], 0).unwrap();
    assert_eq!(
        decoder.transcode(b"", 0, &mut [], 0).unwrap().status(),
        TranscodeStatus::Complete
    );
    let _ = decoder.transcode(b"TQ==", 0, &mut [0; 1], 0).unwrap();
    assert!(decoder.transcode(b"x", 2, &mut [0; 1], 0).is_err());
    assert!(decoder.transcode(b"TWFu", 0, &mut [0; 3], 0).unwrap_err().is_domain());
}

#[test]
fn test_base64_transcoder_structured_errors_report_positions_and_source() {
    let input = b"__TWFu@AAA";
    let mut decoder = Base64Decoder::url_safe();
    decoder.reset(&mut [], 0).unwrap();
    let mut storage = [0xa5; 14];
    let error = decoder.transcode(input, 2, &mut storage[1..4], 0).unwrap_err();
    assert_eq!(storage[0], 0xa5);
    assert_eq!(&storage[1..4], b"Man");
    assert_eq!(&storage[4..], &[0xa5; 10]);
    let domain = error.domain_ref().unwrap();
    assert!(matches!(
        domain,
        MiscCodecError::InvalidBase64 {
            kind: Base64ErrorKind::InvalidByte,
            input_index: Some(0),
            symbol: Some(b'@'),
            ..
        }
    ));
    assert_eq!(error.domain_error_ref().and_then(|e| e.input_index()), Some(6));
    let source = std::error::Error::source(domain).unwrap();
    assert_eq!(
        source.downcast_ref::<DecodeError>(),
        Some(&DecodeError::InvalidByte(0, b'@'))
    );
    assert!(
        decoder
            .transcode(&[], 0, &mut [], 0)
            .unwrap_err()
            .failure_ref()
            .is_some()
    );

    let mut eof_decoder = Base64Decoder::url_safe();
    eof_decoder.reset(&mut [], 0).unwrap();
    let mut output = [0xa5; 12];
    let error = eof_decoder.transcode_eof(input, 2, &mut output[1..7], 0).unwrap_err();
    assert_eq!(output[0], 0xa5);
    assert_eq!(&output[7..], &[0xa5; 5]);
    let domain = error.domain_ref().unwrap();
    assert!(matches!(
        domain,
        MiscCodecError::InvalidBase64 {
            kind: Base64ErrorKind::InvalidByte,
            input_index: Some(4),
            symbol: Some(b'@'),
            ..
        }
    ));
    assert_eq!(error.domain_error_ref().and_then(|e| e.input_index()), Some(6));
    let source = std::error::Error::source(domain).unwrap();
    assert_eq!(
        source.downcast_ref::<DecodeError>(),
        Some(&DecodeError::InvalidByte(4, b'@'))
    );

    let mut invalid_length = Base64Decoder::standard();
    invalid_length.reset(&mut [], 0).unwrap();
    let error = invalid_length.transcode_eof(b"T", 0, &mut [0; 3], 0).unwrap_err();
    assert_eq!(error.domain_error_ref().and_then(|e| e.input_index()), Some(0));
}

#[test]
fn test_base64_encoder_need_output_uses_advanced_output_position() {
    for start in [0, 2] {
        let mut encoder = Base64Encoder::standard();
        encoder.reset(&mut [], 0).expect("start stream");
        let mut output = vec![0xa5; start + 4];
        let progress = encoder
            .transcode(b"ManMan", 0, &mut output, start)
            .expect("encode first complete quantum");
        progress.validate(0, 6, start, 4).expect("valid progress");
        assert_eq!((progress.read(), progress.written()), (5, 4));
        assert_eq!(&output[start..start + 4], b"TWFu");
        let required = match progress.status() {
            TranscodeStatus::NeedOutput { required } => required.get(),
            status => panic!("expected NeedOutput, got {status:?}"),
        };
        assert_eq!(required, 4);

        let next_output = start + progress.written();
        let next_len = next_output.checked_add(required).expect("capacity fits");
        output.resize(next_len, 0xa5);
        let retry = encoder
            .transcode(b"ManMan", progress.read(), &mut output, next_output)
            .expect("retry unconsumed input");
        retry
            .validate(progress.read(), 6 - progress.read(), next_output, required)
            .expect("valid retry progress");
        assert_eq!((retry.read(), retry.written()), (1, 4));
        assert_eq!(&output[start..start + 8], b"TWFuTWFu");
        assert!(output[..start].iter().all(|byte| *byte == 0xa5));
        let finish_index = next_output + retry.written();
        assert_eq!(encoder.finish(&mut output, finish_index).unwrap(), 0);
    }
}

/// Decodes progressively revealed chunks, preserving decoder state between
/// them.
fn decode_long_chunks(
    decoder: &mut Base64Decoder,
    input: &[u8],
    chunk_size: usize,
    first_capacity: usize,
) -> Result<Vec<u8>, TranscodeDecodeError<MiscCodecError>> {
    decoder.reset(&mut [], 0).expect("start decoder");
    let mut output = Vec::new();
    let mut cursor = 0;
    let mut visible = chunk_size.min(input.len());
    let mut capacity = first_capacity;
    let limit = input.len().saturating_mul(4).saturating_add(64);

    for _ in 0..limit {
        let eof = visible == input.len();
        let mut scratch = vec![0xa5; capacity];
        let progress = if eof {
            decoder.transcode_eof(&input[..visible], cursor, &mut scratch, 0)?
        } else {
            decoder.transcode(&input[..visible], cursor, &mut scratch, 0)?
        };
        progress
            .validate(cursor, visible - cursor, 0, capacity)
            .expect("valid decoder progress");
        cursor += progress.read();
        output.extend_from_slice(&scratch[..progress.written()]);
        match progress.status() {
            TranscodeStatus::Complete if eof => {
                decoder.finish(&mut [], 0).expect("finish decoder");
                return Ok(output);
            }
            TranscodeStatus::Complete | TranscodeStatus::NeedInput { .. } => {
                assert!(!eof, "EOF must complete or reject input");
                visible = visible.saturating_add(chunk_size).min(input.len());
                capacity = first_capacity;
            }
            TranscodeStatus::NeedOutput { required } => {
                capacity = required.get();
                assert!((1..=3).contains(&capacity));
            }
        }
    }
    panic!("decoder failed to converge within the bounded step count");
}

#[test]
fn test_base64_transcoder_long_inputs_and_recovery() {
    let factories: [fn() -> Base64Codec; 4] = [
        Base64Codec::standard,
        Base64Codec::standard_no_pad,
        Base64Codec::url_safe,
        Base64Codec::url_safe_no_pad,
    ];
    for make in factories {
        let codec = make();
        for len in [31, 32, 33, 63, 64, 65, 4095, 4096, 4097] {
            let payload: Vec<u8> = (0..len).map(|index| (index as u8).wrapping_mul(37)).collect();
            let text = codec.encode(&payload);
            let bound = codec.decoder().max_total_output_len(text.len()).expect("output bound");
            for chunk_size in [1, 7, 16, 4096] {
                for capacity in [0, 1, 2, 3, bound] {
                    let mut decoder = codec.decoder();
                    let actual = decode_long_chunks(&mut decoder, text.as_bytes(), chunk_size, capacity)
                        .expect("valid long input");
                    assert_eq!(actual, payload, "len={len}, chunk={chunk_size}, cap={capacity}");
                }
                let mut encoder = codec.encoder();
                encoder.reset(&mut [], 0).expect("start encoder");
                let mut encoded = Vec::new();
                for chunk in payload.chunks(chunk_size) {
                    drive_encode_segment(&mut encoder, chunk, 0, &mut encoded);
                }
                let mut tail = [0; 4];
                let written = encoder.finish(&mut tail, 0).expect("finish encoder");
                encoded.extend_from_slice(&tail[..written]);
                assert_eq!(encoded, text.as_bytes());
            }
        }

        let mut long_malformed = codec.encode(&vec![0x5a; 4096]).into_bytes();
        long_malformed.push(b'@');
        for malformed in [long_malformed, b"TQ==TWFu".to_vec()] {
            let mut decoder = codec.decoder();
            let error = decode_long_chunks(&mut decoder, &malformed, 7, 1).expect_err("malformed input must fail");
            assert!(error.is_domain());
            assert!(matches!(
                decoder.finish(&mut [], 0).unwrap_err().failure_ref(),
                Some(TranscodeFailure::LifecyclePoisoned)
            ));
            assert_eq!(
                decode_long_chunks(&mut decoder, b"TWFu", 1, 0).expect("reset recovery"),
                b"Man"
            );
        }
    }
}

#[test]
fn test_base64_transcoder_malformed_every_split_and_capacity() {
    let factories: [fn() -> Base64Codec; 4] = [
        Base64Codec::standard,
        Base64Codec::standard_no_pad,
        Base64Codec::url_safe,
        Base64Codec::url_safe_no_pad,
    ];
    let inputs = [
        "",
        "TWFu",
        "TQ==",
        "TQ",
        "+/8=",
        "-_8=",
        "TWFuZh==",
        "TWFuZm9=",
        "TWFuZg=",
        "Z=g=",
        "Zg===",
        "TQ==TQ==",
        "TQ==A",
        "TWFu@AAA",
        "TWFuZg==\n",
    ];
    for (mode, make) in factories.into_iter().enumerate() {
        let codec = make();
        for text in inputs {
            let expected = codec.decode(text);
            let input = text.as_bytes();
            let sufficient = codec.decoder().max_transcode_output_len(input.len()).unwrap().max(4);
            for split in 0..=input.len() {
                for capacity in [0, 1, 2, 3, sufficient] {
                    let case = format!("mode={mode} input={text:?} split={split} capacity={capacity}");
                    let mut decoder = codec.decoder();
                    decoder.reset(&mut [], 0).unwrap();
                    let mut actual = Vec::new();
                    let result = (|| -> Result<(), TranscodeDecodeError<MiscCodecError>> {
                        let read = drive_decode_part(&mut decoder, &input[..split], false, capacity, &mut actual)?;
                        // Retain the unconsumed prefix and append the second chunk.
                        drive_decode_part(&mut decoder, &input[read..], true, capacity, &mut actual)?;
                        assert_eq!(decoder.finish(&mut [], 0)?, 0);
                        Ok(())
                    })();
                    match (&expected, result) {
                        (Ok(bytes), Ok(())) => assert_eq!(&actual, bytes, "{case}"),
                        (Err(_), Err(error)) => {
                            assert!(error.is_domain(), "{case}: {error}");
                            let error = decoder.transcode(&[], 0, &mut [], 0).unwrap_err();
                            assert!(
                                matches!(error.failure_ref(), Some(TranscodeFailure::LifecyclePoisoned)),
                                "{case}"
                            );
                            let error = decoder.finish(&mut [], 0).unwrap_err();
                            assert!(
                                matches!(error.failure_ref(), Some(TranscodeFailure::LifecyclePoisoned)),
                                "{case}"
                            );
                            decoder.reset(&mut [], 0).unwrap();
                            let mut output = [0; 3];
                            let progress = decoder.transcode_eof(b"TWFu", 0, &mut output, 0).unwrap();
                            assert_eq!(progress.read(), 4, "{case}");
                            assert_eq!(progress.written(), 3, "{case}");
                            assert_eq!(progress.status(), TranscodeStatus::Complete, "{case}");
                            assert_eq!(&output, b"Man", "{case}");
                            assert_eq!(decoder.finish(&mut [], 0).unwrap(), 0);
                        }
                        (_, outcome) => {
                            panic!("acceptance mismatch: {case}, expected={expected:?}, actual={outcome:?}")
                        }
                    }
                }
            }
        }
    }
}
