/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for Base64 encoding variants.

use qubit_codec::{
    Base64Codec,
    CodecError,
    Decoder,
    Encoder,
};

#[test]
fn test_standard_base64_roundtrip_with_padding() {
    let codec = Base64Codec::standard();
    let encoded = codec.encode(b"hello world");

    assert_eq!("aGVsbG8gd29ybGQ=", encoded);
    assert_eq!(
        b"hello world".to_vec(),
        codec.decode(&encoded).expect("base64 should decode")
    );
}

#[test]
fn test_url_safe_base64_roundtrip_without_padding() {
    let codec = Base64Codec::url_safe_no_pad();
    let bytes = [0xfb, 0xff, 0xee, 0x00];
    let encoded = codec.encode(&bytes);

    assert_eq!("-__uAA", encoded);
    assert_eq!(
        bytes.to_vec(),
        codec
            .decode(&encoded)
            .expect("url-safe base64 should decode")
    );
}

#[test]
fn test_base64_constructors_cover_padding_and_alphabet_variants() {
    assert_eq!("aGk", Base64Codec::standard_no_pad().encode(b"hi"));
    assert_eq!("++8=", Base64Codec::standard().encode(&[0xfb, 0xef]));
    assert_eq!("--8=", Base64Codec::url_safe().encode(&[0xfb, 0xef]));
    assert_eq!(
        b"hi".to_vec(),
        Base64Codec::standard_no_pad()
            .decode("aGk")
            .expect("standard no-pad base64 should decode")
    );
    assert_eq!(
        vec![0xfb, 0xef],
        Base64Codec::url_safe()
            .decode("--8=")
            .expect("url-safe padded base64 should decode")
    );
    assert_eq!(
        b"hi".to_vec(),
        Base64Codec::default()
            .decode("aGk=")
            .expect("default base64 should decode")
    );
}

#[test]
fn test_decode_rejects_invalid_base64() {
    let error = Base64Codec::standard()
        .decode("not base64!")
        .expect_err("invalid base64 should fail");

    assert!(matches!(error, CodecError::InvalidBase64 { .. }));
}

#[test]
fn test_base64_codec_can_be_used_through_traits() {
    let codec = Base64Codec::standard();
    let encoded = Encoder::<[u8]>::encode(&codec, b"abc").expect("base64 encode should succeed");
    let decoded = Decoder::<str>::decode(&codec, &encoded).expect("base64 decode should succeed");

    assert_eq!("YWJj", encoded);
    assert_eq!(b"abc".to_vec(), decoded);
}
