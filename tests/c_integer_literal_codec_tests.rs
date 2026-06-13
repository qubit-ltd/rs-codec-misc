// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for C integer literal decoding.

use qubit_codec_misc::{CIntegerLiteralCodec, MiscCodecError, ValueDecoder};

#[test]
fn test_decode_decimal_octal_and_hex_literals() {
    let codec = CIntegerLiteralCodec::new();

    assert_eq!(0, codec.decode("0").expect("zero should decode"));
    assert_eq!(123, codec.decode("123").expect("decimal should decode"));
    assert_eq!(83, codec.decode("0123").expect("octal should decode"));
    assert_eq!(
        0xbeef_c0de,
        codec.decode("0xBEEFC0DE").expect("hex should decode")
    );
    assert_eq!(
        0xbeef_c0de,
        codec
            .decode("0Xbeefc0de")
            .expect("uppercase prefix should decode")
    );
}

#[test]
fn test_decode_trims_surrounding_ascii_and_unicode_whitespace() {
    let codec = CIntegerLiteralCodec::new();

    assert_eq!(
        42,
        codec
            .decode(" \t42\n")
            .expect("ASCII whitespace should trim")
    );
    assert_eq!(
        42,
        codec
            .decode("\u{2003}42\u{2003}")
            .expect("Unicode whitespace should trim")
    );
}

#[test]
fn test_decode_reports_invalid_digits_with_original_index() {
    let octal = CIntegerLiteralCodec::new()
        .decode(" 09")
        .expect_err("invalid octal digit should fail");
    assert!(matches!(
        octal,
        MiscCodecError::InvalidDigit {
            radix: 8,
            index: 2,
            character: '9'
        }
    ));

    let hex = CIntegerLiteralCodec::new()
        .decode(" 0xg")
        .expect_err("invalid hex digit should fail");
    assert!(matches!(
        hex,
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 3,
            character: 'g'
        }
    ));

    let decimal = CIntegerLiteralCodec::new()
        .decode("+1")
        .expect_err("signed literals are not supported");
    assert!(matches!(
        decimal,
        MiscCodecError::InvalidDigit {
            radix: 10,
            index: 0,
            character: '+'
        }
    ));
}

#[test]
fn test_decode_reports_empty_missing_digits_and_overflow() {
    let empty = CIntegerLiteralCodec::new()
        .decode(" \t\n")
        .expect_err("empty input should fail");
    assert!(matches!(
        empty,
        MiscCodecError::InvalidInput {
            codec: "c-integer-literal",
            ..
        }
    ));

    let missing_hex = CIntegerLiteralCodec::new()
        .decode("0x")
        .expect_err("hex prefix without digits should fail");
    assert!(matches!(
        missing_hex,
        MiscCodecError::InvalidInput {
            codec: "c-integer-literal",
            ..
        }
    ));

    let overflow = CIntegerLiteralCodec::new()
        .decode("18446744073709551616")
        .expect_err("overflow should fail");
    assert!(matches!(
        overflow,
        MiscCodecError::InvalidInput {
            codec: "c-integer-literal",
            ..
        }
    ));
}

#[test]
fn test_c_integer_literal_codec_can_be_used_through_decoder_trait() {
    let mut codec = CIntegerLiteralCodec::new();
    let decoded = ValueDecoder::<str>::decode(&mut codec, "0x2a")
        .expect("C integer literal should decode through trait");

    assert_eq!(42, decoded);
}
