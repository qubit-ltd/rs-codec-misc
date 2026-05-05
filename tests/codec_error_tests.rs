/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for shared codec errors.

use std::error::Error;

use qubit_codec::{
    CodecError,
    CodecResult,
};

#[test]
fn test_codec_error_display_messages_include_context() {
    let cases = [
        (
            CodecError::MissingPrefix {
                prefix: "0x".to_owned(),
            },
            "missing required prefix '0x'",
        ),
        (
            CodecError::InvalidDigit {
                radix: 16,
                index: 3,
                character: 'g',
            },
            "invalid radix-16 digit 'g' at index 3",
        ),
        (
            CodecError::InvalidLength {
                context: "hex digits",
                expected: "even number".to_owned(),
                actual: 3,
            },
            "invalid length for hex digits: expected even number, got 3",
        ),
        (
            CodecError::InvalidEscape {
                index: 1,
                escape: "%z".to_owned(),
                reason: "expected two hex digits".to_owned(),
            },
            "invalid escape \"%z\" at index 1: expected two hex digits",
        ),
        (
            CodecError::InvalidCharacter {
                index: 5,
                character: ' ',
                reason: "space is not allowed".to_owned(),
            },
            "invalid character ' ' at index 5: space is not allowed",
        ),
        (
            CodecError::InvalidInput {
                codec: "base64",
                reason: "invalid symbol".to_owned(),
            },
            "invalid base64 input: invalid symbol",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(expected, error.to_string());
    }
}

#[test]
fn test_codec_error_wraps_utf8_source_error() {
    let error = String::from_utf8(vec![0xff]).expect_err("invalid utf-8 should fail");
    let error = CodecError::from(error);

    assert_eq!(
        "decoded bytes are not valid UTF-8: invalid utf-8 sequence of 1 bytes from index 0",
        error.to_string()
    );
    assert!(error.source().is_some());
}

#[test]
fn test_codec_result_alias_uses_codec_error() {
    fn decode_stub() -> CodecResult<()> {
        Err(CodecError::MissingPrefix {
            prefix: "#".to_owned(),
        })
    }

    let error = decode_stub().expect_err("stub should return codec error");

    assert!(matches!(error, CodecError::MissingPrefix { prefix } if prefix == "#"));
}
