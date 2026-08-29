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
