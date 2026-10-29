use qubit_codec_misc::Base64Codec;
use qubit_codec_misc::Base64Encoder;
use qubit_codec::{TranscodeFailure, TranscodeStatus, Transcoder};

#[test]
fn test_base64_encoder_constructors_are_independent_of_codec_lifetime() {
    let encoder = {
        let codec = Base64Codec::url_safe_no_pad();
        codec.encoder()
    };
    let constructed = [
        Base64Encoder::standard(),
        Base64Encoder::standard_no_pad(),
        Base64Encoder::url_safe(),
        Base64Encoder::url_safe_no_pad(),
        Base64Encoder::default(),
        encoder,
    ];
    assert_eq!(constructed.len(), 6);
}

#[test]
fn test_base64_encoder_retains_two_bytes_then_reports_output_pressure() {
    let mut encoder = Base64Encoder::standard();
    encoder.reset(&mut [], 0).expect("reset");
    let progress = encoder.transcode(b"Man", 0, &mut [], 0).expect("backpressure");
    assert_eq!(progress.read(), 2);
    assert_eq!(progress.written(), 0);
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { required } if required.get() == 4));
    let mut out = [0; 4];
    let progress = encoder.transcode(b"n", 0, &mut out, 0).expect("resume");
    assert_eq!(progress.read(), 1);
    assert_eq!(progress.written(), 4);
    assert_eq!(progress.status(), TranscodeStatus::Complete);
    assert_eq!(&out, b"TWFu");
    assert_eq!(encoder.finish(&mut [], 0).expect("finish"), 0);
}

#[test]
fn test_base64_encoder_consumes_one_byte_per_call() {
    let mut encoder = Base64Encoder::standard();
    encoder.reset(&mut [], 0).expect("reset");
    let mut out = [0; 4];
    for (input, expected_read) in [(b"M".as_slice(), 1), (b"a".as_slice(), 1)] {
        let progress = encoder.transcode(input, 0, &mut out, 0).expect("retain partial quantum");
        assert_eq!(progress.status(), TranscodeStatus::Complete);
        assert_eq!(progress.read(), expected_read);
        assert_eq!(progress.written(), 0);
    }
    let progress = encoder.transcode(b"n", 0, &mut out, 0).expect("complete quantum");
    assert_eq!(progress.status(), TranscodeStatus::Complete);
    assert_eq!(progress.read(), 1);
    assert_eq!(progress.written(), 4);
    assert_eq!(&out, b"TWFu");
}

#[test]
fn test_base64_encoder_output_pressure_at_nonzero_output_index() {
    let mut encoder = Base64Encoder::standard();
    encoder.reset(&mut [], 0).expect("reset");
    let mut out = [0xa5; 6];
    let progress = encoder.transcode(b"Man", 0, &mut out[..5], 2).expect("backpressure");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { required } if required.get() == 4));
    assert_eq!(progress.read(), 2);
    assert_eq!(progress.written(), 0);
    assert_eq!(out, [0xa5; 6]);

    let progress = encoder.transcode(b"n", 0, &mut out, 2).expect("resume at same output index");
    assert_eq!(progress.status(), TranscodeStatus::Complete);
    assert_eq!(progress.read(), 1);
    assert_eq!(progress.written(), 4);
    assert_eq!(&out[2..6], b"TWFu");
    assert_eq!(&out[..2], &[0xa5; 2]);
}

#[test]
fn test_base64_encoder_finish_capacity_failure_preserves_tail() {
    let mut encoder = Base64Encoder::standard();
    encoder.reset(&mut [], 0).expect("reset");
    let p = encoder.transcode(b"M", 0, &mut [], 0).expect("retain tail");
    assert_eq!(p.read(), 1);
    assert_eq!(p.status(), TranscodeStatus::Complete);
    let mut short = [0xa5; 3];
    let error = encoder.finish(&mut short, 0).expect_err("short output");
    assert!(matches!(error.failure_ref(), Some(TranscodeFailure::InsufficientOutput { required: 4, available: 3, .. })));
    assert_eq!(short, [0xa5; 3]);
    let mut out = [0; 4];
    assert_eq!(encoder.finish(&mut out, 0).expect("retry finish"), 4);
    assert_eq!(&out, b"TQ==");
}

#[test]
fn test_base64_encoder_all_modes_and_every_input_split() {
    type Mode = (fn() -> Base64Codec, fn() -> Base64Encoder);
    let modes: [Mode; 4] = [
        (Base64Codec::standard, Base64Encoder::standard),
        (Base64Codec::standard_no_pad, Base64Encoder::standard_no_pad),
        (Base64Codec::url_safe, Base64Encoder::url_safe),
        (Base64Codec::url_safe_no_pad, Base64Encoder::url_safe_no_pad),
    ];
    let cases: &[&[u8]] = &[b"", b"M", b"Ma", b"Man", b"Mann", &[0xfb, 0xff, 0xee, 0]];
    for (make_codec, make_encoder) in modes {
        let codec = make_codec();
        for payload in cases {
            let expected = codec.encode(payload);
            for mut encoder in [make_encoder(), codec.encoder()] {
                for split in 0..=payload.len() {
                    let mut output = [0xa5; 32];
                    encoder.reset(&mut [], 0).expect("reset");
                    let first = encoder.transcode(&payload[..split], 0, &mut output, 0).expect("first segment");
                    assert_eq!(first.status(), TranscodeStatus::Complete);
                    assert_eq!(first.read(), split);
                    let second = encoder.transcode_eof(&payload[split..], 0, &mut output, first.written()).expect("final segment");
                    assert_eq!(second.status(), TranscodeStatus::Complete);
                    assert_eq!(second.read(), payload.len() - split);
                    let before_finish = first.written() + second.written();
                    let last = encoder.finish(&mut output, before_finish).expect("finish");
                    assert_eq!(&output[..before_finish + last], expected.as_bytes());
                }
            }
        }
    }
}

#[test]
fn test_base64_encoder_lifecycle_indices_and_reset() {
    let mut encoder = Base64Encoder::standard();
    assert!(matches!(encoder.finish(&mut [], 0).unwrap_err().failure_ref(), Some(TranscodeFailure::FinishBeforeReset)));
    encoder.reset(&mut [], 0).expect("reset");
    let mut output = [0xa5; 8];
    let first = encoder.transcode(b"xM", 1, &mut output, 2).expect("nonzero indices");
    assert_eq!(first.status(), TranscodeStatus::Complete);
    assert_eq!(first.read(), 1);
    assert_eq!(encoder.finish(&mut output, 2).expect("finish"), 4);
    assert_eq!(&output[2..6], b"TQ==");
    assert!(matches!(encoder.finish(&mut output, 0).unwrap_err().failure_ref(), Some(TranscodeFailure::FinishAfterFinish)));
    assert!(matches!(encoder.transcode(b"", 0, &mut output, 0).unwrap_err().failure_ref(), Some(TranscodeFailure::TranscodeAfterFinish)));

    encoder.reset(&mut [], 0).expect("reset after finish");
    let p = encoder.transcode(b"M", 0, &mut output, 0).expect("retain tail");
    assert_eq!(p.read(), 1);
    encoder.reset(&mut [], 0).expect("discard old tail");
    let p = encoder.transcode(b"a", 0, &mut output, 0).expect("new stream");
    assert_eq!(p.read(), 1);
    assert_eq!(encoder.finish(&mut output, 0).expect("finish new stream"), 4);
    assert_eq!(&output[..4], b"YQ==");
}
