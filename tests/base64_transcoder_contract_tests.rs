// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_codec::CapacityError;
use qubit_codec::TranscodeDecoder;
use qubit_codec::TranscodeEncoder;
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec_misc::Base64Codec;
use qubit_codec_misc::Base64Decoder;
use qubit_codec_misc::Base64Encoder;
use qubit_codec_misc::Base64ErrorKind;
use qubit_codec_misc::MiscCodecError;

fn require_encoder<T: TranscodeEncoder<Input = u8, Output = u8>>(_: &T) {}
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
        Some(qubit_codec::TranscodeFailure::InsufficientOutput {
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
        Some(qubit_codec::TranscodeFailure::InsufficientOutput { .. })
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
        source.downcast_ref::<base64::DecodeError>(),
        Some(&base64::DecodeError::InvalidByte(0, b'@'))
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
        source.downcast_ref::<base64::DecodeError>(),
        Some(&base64::DecodeError::InvalidByte(4, b'@'))
    );

    let mut invalid_length = Base64Decoder::standard();
    invalid_length.reset(&mut [], 0).unwrap();
    let error = invalid_length.transcode_eof(b"T", 0, &mut [0; 3], 0).unwrap_err();
    assert_eq!(error.domain_error_ref().and_then(|e| e.input_index()), Some(0));
}
