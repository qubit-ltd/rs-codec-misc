// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for complete Base64 quantum encoding and decoding.

use qubit_codec::Codec;
use qubit_codec_misc::Base64QuantumCodec;
use qubit_codec_misc::MiscCodecError;

#[test]
fn test_base64_quantum_standard_roundtrip() {
    let mut codec = Base64QuantumCodec::standard();
    let mut output = [0u8; 4];

    let (decoded, consumed) = unsafe { Codec::decode(&mut codec, b"YWJj", 0).expect("base64 quantum should decode") };
    let written = unsafe { Codec::encode(&mut codec, b"abc", &mut output, 0).expect("base64 quantum should encode") };

    assert_eq!(*b"abc", decoded);
    assert_eq!(4, consumed.get());
    assert_eq!(4, written);
    assert_eq!(b"YWJj", &output);
}

#[test]
fn test_base64_quantum_url_safe_roundtrip() {
    let mut codec = Base64QuantumCodec::url_safe();
    let mut output = [0u8; 4];

    let (decoded, consumed) = unsafe { Codec::decode(&mut codec, b"-__u", 0).expect("URL-safe quantum should decode") };
    let written = unsafe {
        Codec::encode(&mut codec, &[0xfb, 0xff, 0xee], &mut output, 0).expect("URL-safe quantum should encode")
    };

    assert_eq!([0xfb, 0xff, 0xee], decoded);
    assert_eq!(4, consumed.get());
    assert_eq!(4, written);
    assert_eq!(b"-__u", &output);
}

#[test]
fn test_base64_quantum_alphabet_and_errors() {
    let mut standard = Base64QuantumCodec::default();
    let mut url_safe = Base64QuantumCodec::url_safe();

    let (decoded, _) = unsafe { Codec::decode(&mut standard, b"++//", 0) }.expect("standard symbols should decode");
    assert_eq!([0xfb, 0xef, 0xff], decoded);

    let (decoded, _) = unsafe { Codec::decode(&mut standard, b"0123", 0) }.expect("digit symbols should decode");
    assert_eq!([0xd3, 0x5d, 0xb7], decoded);
    let invalid = unsafe { Codec::decode(&mut url_safe, b"@@@@", 0) }.expect_err("invalid Base64 quantum should fail");
    assert_eq!(Some(4), super::invalid_consumed(invalid));
    let invalid = unsafe { Codec::decode(&mut url_safe, b"@@@@", 0) }.expect_err("invalid Base64 quantum should fail");
    assert!(matches!(
        super::invalid_source(invalid),
        MiscCodecError::InvalidInput {
            codec: "base64-quantum",
            ..
        }
    ));
}

#[test]
fn test_base64_quantum_reports_invalid_units_at_each_position() {
    let invalid_inputs = [b"A@AA".as_slice(), b"AA@A", b"AAA@"];

    for input in invalid_inputs {
        let invalid = unsafe { Codec::decode(&mut Base64QuantumCodec::standard(), input, 0) }
            .expect_err("invalid Base64 unit should fail");
        assert_eq!(Some(4), super::invalid_consumed(invalid));

        let invalid = unsafe { Codec::decode(&mut Base64QuantumCodec::standard(), input, 0) }
            .expect_err("invalid Base64 unit should fail");
        assert!(matches!(
            super::invalid_source(invalid),
            MiscCodecError::InvalidInput {
                codec: "base64-quantum",
                ..
            }
        ));
    }
}

#[test]
fn test_base64_quantum_rejects_foreign_alphabet_and_padding() {
    for input in [b"-AAA".as_slice(), b"_AAA"] {
        // SAFETY: each input contains the four units required by this codec.
        let failure = unsafe { Codec::decode(&mut Base64QuantumCodec::standard(), input, 0) }
            .expect_err("standard alphabet should reject URL-safe symbols");
        assert_eq!(Some(4), failure.consumed_units().map(|width| width.get()));
        assert!(matches!(
            failure.invalid_source(),
            Some(MiscCodecError::InvalidInput { .. })
        ));
    }
    for input in [b"+AAA".as_slice(), b"/AAA"] {
        // SAFETY: each input contains the four units required by this codec.
        let failure = unsafe { Codec::decode(&mut Base64QuantumCodec::url_safe(), input, 0) }
            .expect_err("URL-safe alphabet should reject standard symbols");
        assert_eq!(Some(4), failure.consumed_units().map(|width| width.get()));
        assert!(matches!(
            failure.invalid_source(),
            Some(MiscCodecError::InvalidInput { .. })
        ));
    }
    for input in [b"AA==".as_slice(), b"AAA="] {
        for mut codec in [Base64QuantumCodec::standard(), Base64QuantumCodec::url_safe()] {
            // SAFETY: each input contains the four units required by this codec.
            let failure =
                unsafe { Codec::decode(&mut codec, input, 0) }.expect_err("quantum codec should reject padding");
            assert_eq!(Some(4), failure.consumed_units().map(|width| width.get()));
            assert!(matches!(
                failure.invalid_source(),
                Some(MiscCodecError::InvalidInput { .. })
            ));
        }
    }
}

#[test]
fn test_base64_quantum_respects_nonzero_offsets() {
    let cases = [
        (Base64QuantumCodec::standard(), *b"abc", *b"YWJj"),
        (Base64QuantumCodec::url_safe(), [0xfb, 0xff, 0xee], *b"-__u"),
    ];
    for (mut codec, value, units) in cases {
        for offset in [1, 2] {
            let mut input = vec![b'@'; offset];
            input.extend_from_slice(&units);
            input.extend_from_slice(b"@@");
            // SAFETY: four readable units start at offset.
            let (decoded, consumed) =
                unsafe { Codec::decode(&mut codec, &input, offset) }.expect("quantum at nonzero offset must decode");
            assert_eq!(value, decoded);
            assert_eq!(4, consumed.get());

            let mut output = [0xa5; 8];
            // SAFETY: eight slots include four writable units at offset 1 or 2.
            let written = unsafe { Codec::encode(&mut codec, &value, &mut output, offset) }
                .expect("quantum at nonzero offset must encode");
            assert_eq!(4, written);
            assert_eq!(units.as_slice(), &output[offset..offset + 4]);
            assert!(output[..offset].iter().all(|byte| *byte == 0xa5));
            assert!(output[offset + 4..].iter().all(|byte| *byte == 0xa5));
        }
    }
}

#[test]
fn test_base64_quantum_reports_absolute_invalid_indices() {
    for mut codec in [Base64QuantumCodec::standard(), Base64QuantumCodec::url_safe()] {
        for offset in [1, 2] {
            for position in 0..4 {
                let mut input = vec![b'@'; offset];
                input.extend_from_slice(b"AAAA");
                input.extend_from_slice(b"@@");
                input[offset + position] = b'@';
                // SAFETY: a complete four-unit quantum is readable at offset.
                let failure = unsafe { Codec::decode(&mut codec, &input, offset) }
                    .expect_err("invalid quantum unit must be rejected");
                assert_eq!(Some(4), failure.consumed_units().map(|width| width.get()));
                match failure.invalid_source() {
                    Some(MiscCodecError::InvalidInput { codec, reason }) => {
                        assert_eq!("base64-quantum", *codec);
                        assert_eq!(
                            format!("invalid Base64 unit '@' at index {}", offset + position),
                            *reason
                        );
                    }
                    other => panic!("unexpected failure: {other:?}"),
                }
            }
        }
    }
}
