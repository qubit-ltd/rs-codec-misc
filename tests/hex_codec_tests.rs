/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for hexadecimal byte encoding.

use qubit_codec::{
    CodecError,
    HexCodec,
};

#[test]
fn test_encode_lowercase_contiguous_hex_by_default() {
    let codec = HexCodec::new();

    assert_eq!("1f8b00ff", codec.encode(&[0x1f, 0x8b, 0x00, 0xff]));
    assert_eq!("", codec.encode(&[]));
}

#[test]
fn test_encode_uppercase_with_prefix_and_separator() {
    let codec = HexCodec::upper().with_prefix("0x").with_separator(" ");

    assert_eq!("0x1F 8B 00 FF", codec.encode(&[0x1f, 0x8b, 0x00, 0xff]));
}

#[test]
fn test_encode_and_decode_into_existing_buffers() {
    let codec = HexCodec::default().with_uppercase(false);
    let mut text = String::from("prefix:");
    codec.encode_into(&[0xab, 0xcd], &mut text);
    assert_eq!("prefix:abcd", text);

    let mut bytes = vec![0x00];
    codec
        .decode_into("abcd", &mut bytes)
        .expect("hex should decode into existing buffer");
    assert_eq!(vec![0x00, 0xab, 0xcd], bytes);
}

#[test]
fn test_decode_plain_prefixed_and_separated_hex() {
    assert_eq!(
        vec![0x1f, 0x8b, 0x00, 0xff],
        HexCodec::new()
            .decode("1f8B00ff")
            .expect("plain hex should decode")
    );

    assert_eq!(
        vec![0x1f, 0x8b],
        HexCodec::new()
            .with_prefix("0x")
            .decode("0x1f8b")
            .expect("prefixed hex should decode")
    );

    assert_eq!(
        vec![0x1f, 0x8b, 0x00],
        HexCodec::new()
            .with_separator(":")
            .with_ignored_ascii_whitespace(true)
            .decode("1f: 8B:\n00")
            .expect("separated hex should decode")
    );
}

#[test]
fn test_decode_reports_precise_hex_errors() {
    let odd = HexCodec::new()
        .decode("abc")
        .expect_err("odd number of digits should fail");
    assert!(matches!(odd, CodecError::OddHexLength { digits: 3 }));

    let invalid = HexCodec::new()
        .decode("12xz")
        .expect_err("invalid digit should fail");
    assert!(matches!(
        invalid,
        CodecError::InvalidHexDigit {
            index: 2,
            character: 'x'
        }
    ));

    let missing_prefix = HexCodec::new()
        .with_prefix("0x")
        .decode("1f")
        .expect_err("missing prefix should fail");
    assert!(matches!(missing_prefix, CodecError::MissingPrefix { .. }));
}
