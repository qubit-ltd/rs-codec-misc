// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec_misc::Base64Codec;
use qubit_codec_misc::Base64Decoder;
use qubit_codec_misc::Base64ErrorKind;
use qubit_codec_misc::MiscCodecError;

#[test]
fn test_base64_decoder_constructors_are_independent_of_codec_lifetime() {
    let decoder = {
        let codec = Base64Codec::url_safe_no_pad();
        codec.decoder()
    };
    let constructed = [
        Base64Decoder::standard(),
        Base64Decoder::standard_no_pad(),
        Base64Decoder::url_safe(),
        Base64Decoder::url_safe_no_pad(),
        Base64Decoder::default(),
        decoder,
    ];
    assert_eq!(constructed.len(), 6);
}

#[test]
fn test_base64_decoder_preserves_short_tail_until_eof() {
    let mut decoder = Base64Decoder::standard_no_pad();
    decoder.reset(&mut [], 0).expect("reset");
    let mut out = [0xa5; 3];
    let p = decoder.transcode(b"TQ", 0, &mut out, 0).expect("open stream");
    assert_eq!((p.read(), p.written()), (0, 0));
    assert!(matches!(
        p.status(),
        TranscodeStatus::NeedInput { required } if required.get() == 4
    ));
    assert_eq!(out, [0xa5; 3]);
    let p = decoder.transcode_eof(b"TQ", 0, &mut out, 0).expect("EOF resolves tail");
    assert_eq!((p.read(), p.written()), (2, 1));
    assert_eq!(p.status(), TranscodeStatus::Complete);
    assert_eq!(out[0], b'M');
    assert_eq!(decoder.finish(&mut [], 0).expect("finish"), 0);
}

#[test]
fn test_base64_decoder_preserves_split_padding_prefix_until_eof() {
    let mut decoder = Base64Decoder::standard();
    decoder.reset(&mut [], 0).expect("reset");
    let mut output = [0; 1];
    let progress = decoder
        .transcode(b"TQ=", 0, &mut output, 0)
        .expect("retain an incomplete padding prefix");
    assert_eq!((progress.read(), progress.written()), (0, 0));
    assert!(matches!(
        progress.status(),
        TranscodeStatus::NeedInput { required } if required.get() == 4
    ));

    let progress = decoder
        .transcode_eof(b"TQ==", 0, &mut output, 0)
        .expect("caller resubmits the preserved tail at EOF");
    assert_eq!((progress.read(), progress.written()), (4, 1));
    assert_eq!(progress.status(), TranscodeStatus::Complete);
    assert_eq!(output, [b'M']);
    assert_eq!(decoder.finish(&mut [], 0).expect("finish"), 0);
}

#[test]
fn test_base64_decoder_padding_accepts_exact_output_and_rejects_next_call() {
    let mut decoder = Base64Decoder::standard();
    decoder.reset(&mut [], 0).expect("reset");
    let mut one = [0; 1];
    let p = decoder.transcode(b"TQ==", 0, &mut one, 0).expect("one-byte tail");
    assert_eq!((p.read(), p.written()), (4, 1));
    assert_eq!(one, [b'M']);
    let error = decoder
        .transcode(b"TWFu", 0, &mut [0; 3], 0)
        .expect_err("data after padding");
    assert!(error.is_domain());
    let error = decoder.finish(&mut [], 0).expect_err("poisoned after domain error");
    assert!(matches!(error.failure_ref(), Some(TranscodeFailure::LifecyclePoisoned)));
    decoder.reset(&mut [], 0).expect("reset clears terminal flag");
    assert_eq!(decoder.transcode_complete_into(b"TWFu", &mut [0; 3]).expect("reuse"), 3);
}

#[test]
fn test_base64_decoder_does_not_commit_padding_when_output_is_full() {
    let mut decoder = Base64Decoder::standard();
    decoder.reset(&mut [], 0).expect("reset");
    let progress = decoder
        .transcode(b"TQ==", 0, &mut [], 0)
        .expect("report output pressure");
    assert_eq!((progress.read(), progress.written()), (0, 0));
    assert!(matches!(
        progress.status(),
        TranscodeStatus::NeedOutput { required } if required.get() == 1
    ));
    let mut output = [0; 1];
    let progress = decoder
        .transcode(b"TQ==", 0, &mut output, 0)
        .expect("resume padded quantum");
    assert_eq!((progress.read(), progress.written()), (4, 1));
    assert_eq!(output, [b'M']);
}

#[test]
fn test_base64_decoder_padding_backpressure_uses_actual_quantum_output() {
    for (input, expected_required, expected_output) in [
        (b"TQ==".as_slice(), 1, b"M".as_slice()),
        (b"TWE=", 2, b"Ma"),
        (b"TWFu", 3, b"Man"),
    ] {
        let mut decoder = Base64Decoder::standard();
        decoder.reset(&mut [], 0).expect("reset");
        let progress = decoder
            .transcode(input, 0, &mut [], 0)
            .expect("report quantum capacity");
        assert_eq!((progress.read(), progress.written()), (0, 0));
        assert!(matches!(
            progress.status(),
            TranscodeStatus::NeedOutput { required } if required.get() == expected_required
        ));

        let mut output = [0; 3];
        let progress = decoder.transcode(input, 0, &mut output, 0).expect("resume quantum");
        assert_eq!((progress.read(), progress.written()), (input.len(), expected_required));
        assert_eq!(&output[..expected_required], expected_output);
    }
}

#[test]
fn test_base64_decoder_strict_configurations_match_complete_codec() {
    let configurations = [
        (Base64Codec::standard(), Base64Decoder::standard()),
        (Base64Codec::standard_no_pad(), Base64Decoder::standard_no_pad()),
        (Base64Codec::url_safe(), Base64Decoder::url_safe()),
        (Base64Codec::url_safe_no_pad(), Base64Decoder::url_safe_no_pad()),
    ];
    let malformed: &[&[u8]] = &[
        b"Zg===",
        b"Z=g=",
        b"Zg=",
        b"Z g==",
        b"Zg==\n",
        b"Zh==",
        b"Zm9=",
        b"Zh",
        b"Zm9",
        b"Y",
        b"@",
        &[0xff],
        b"TQ==TQ==",
    ];

    for (codec, mut decoder) in configurations {
        for input in malformed {
            let expected_ok = core::str::from_utf8(input)
                .ok()
                .is_some_and(|text| codec.decode(text).is_ok());
            decoder.reset(&mut [], 0).expect("reset before malformed case");
            let mut output = [0; 16];
            let result = decoder.transcode_complete_into(input, &mut output);
            assert_eq!(result.is_ok(), expected_ok, "input {input:?}");
            if let Err(error) = result {
                assert!(error.is_domain(), "input {input:?}: {error}");
            }
        }
    }
}

#[test]
fn test_base64_decoder_strict_alphabet_and_padding_variants() {
    let modes = [
        (
            Base64Codec::standard(),
            Base64Decoder::standard(),
            b"+/8=".as_slice(),
            b"-_8=".as_slice(),
        ),
        (
            Base64Codec::standard_no_pad(),
            Base64Decoder::standard_no_pad(),
            b"+/8",
            b"-_8",
        ),
        (
            Base64Codec::url_safe(),
            Base64Decoder::url_safe(),
            b"-_8=".as_slice(),
            b"+/8=".as_slice(),
        ),
        (
            Base64Codec::url_safe_no_pad(),
            Base64Decoder::url_safe_no_pad(),
            b"-_8",
            b"+/8",
        ),
    ];
    for (codec, mut decoder, accepted, rejected) in modes {
        let mut output = [0; 3];
        let written = decoder
            .transcode_complete_into(accepted, &mut output)
            .expect("configured alphabet and padding should decode");
        assert_eq!(written, 2);
        assert_eq!(&output[..written], &[0xfb, 0xff]);
        assert!(codec.decode(core::str::from_utf8(rejected).expect("ASCII")).is_err());
        assert!(
            decoder
                .transcode_complete_into(rejected, &mut output)
                .expect_err("foreign alphabet should fail")
                .is_domain()
        );

        let padding_mismatch = if accepted.ends_with(b"=") {
            &accepted[..accepted.len() - 1]
        } else {
            b"+/8=".as_slice()
        };
        assert!(
            decoder
                .transcode_complete_into(padding_mismatch, &mut output)
                .expect_err("padding mismatch should fail")
                .is_domain()
        );
    }
}

#[test]
fn test_base64_decoder_domain_error_preserves_source_and_segment_position() {
    let mut decoder = Base64Decoder::standard();
    decoder.reset(&mut [], 0).expect("reset");
    let mut output = [0; 3];
    let error = decoder
        .transcode(b"TWFu@AAA", 0, &mut output, 0)
        .expect_err("invalid byte after a decoded quantum");
    assert!(matches!(
        error.domain_ref(),
        Some(MiscCodecError::InvalidBase64 {
            kind: Base64ErrorKind::InvalidByte,
            input_index: Some(0),
            symbol: Some(b'@'),
            ..
        })
    ));
    assert_eq!(
        error.domain_error_ref().and_then(|domain| domain.input_index()),
        Some(4)
    );
    let domain_source = error.domain_ref().expect("domain error source");
    let source = std::error::Error::source(domain_source).expect("Base64 dependency source");
    assert_eq!(
        Some(&base64::DecodeError::InvalidByte(0, b'@')),
        source.downcast_ref::<base64::DecodeError>()
    );
}
