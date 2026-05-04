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
fn test_production_code_does_not_use_panic_helpers() {
    let sources = [
        include_str!("../src/base64_codec.rs"),
        include_str!("../src/codec.rs"),
        include_str!("../src/codec_error.rs"),
        include_str!("../src/decoder.rs"),
        include_str!("../src/encoder.rs"),
        include_str!("../src/form_urlencoded_codec.rs"),
        include_str!("../src/hex_codec.rs"),
        include_str!("../src/lib.rs"),
        include_str!("../src/percent_codec.rs"),
    ];
    let combined = sources.join("\n");

    assert!(
        !combined.contains(".expect("),
        "production code should return CodecError instead of panicking"
    );
    assert!(
        !combined.contains(".unwrap("),
        "production code should return CodecError instead of panicking"
    );
    assert!(
        !combined.contains("panic!"),
        "production code should return CodecError instead of panicking"
    );
    assert!(
        !combined.contains("unreachable!"),
        "production code should return CodecError instead of panicking"
    );
    assert!(
        !combined.contains("todo!"),
        "production code should return CodecError instead of panicking"
    );
    assert!(
        !combined.contains("unimplemented!"),
        "production code should return CodecError instead of panicking"
    );
}

#[test]
fn test_encode_lowercase_contiguous_hex_by_default() {
    let codec = HexCodec::new();

    assert_eq!("1f8b00ff", codec.encode(&[0x1f, 0x8b, 0x00, 0xff]));
    assert_eq!(
        "0123456789abcdef",
        codec.encode(&[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef])
    );
    assert_eq!("", codec.encode(&[]));
}

#[test]
fn test_encode_uppercase_with_prefix_and_separator() {
    let codec = HexCodec::upper().with_prefix("0x").with_separator(" ");

    assert_eq!(
        "0x1F 0x8B 0x00 0xFF",
        codec.encode(&[0x1f, 0x8b, 0x00, 0xff])
    );
    assert_eq!(
        "0x01 0x23 0x45 0x67 0x89 0xAB 0xCD 0xEF",
        codec.encode(&[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef])
    );
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
            .decode("0x1f0x8b")
            .expect("prefixed contiguous hex should decode")
    );

    assert_eq!(
        vec![0x1f, 0x8b],
        HexCodec::new()
            .with_prefix("0x")
            .with_separator(" ")
            .decode("0x1f 0x8b")
            .expect("prefixed separated hex should decode")
    );

    assert_eq!(
        vec![0x1f, 0x8b],
        HexCodec::new()
            .with_prefix("0x")
            .with_separator(" ")
            .with_ignored_ascii_whitespace(true)
            .decode(" \t0x1 \nF 0x8B ")
            .expect("prefixed hex should tolerate configured whitespace")
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

    let missing_second_prefix = HexCodec::new()
        .with_prefix("0x")
        .decode("0x1f8b")
        .expect_err("each byte should require its own prefix");
    assert!(matches!(
        missing_second_prefix,
        CodecError::MissingPrefix { .. }
    ));

    let invalid_after_prefix = HexCodec::new()
        .with_prefix("0x")
        .decode("0x1g")
        .expect_err("invalid digit after prefix should fail");
    assert!(matches!(
        invalid_after_prefix,
        CodecError::InvalidHexDigit {
            index: 3,
            character: 'g'
        }
    ));
}
