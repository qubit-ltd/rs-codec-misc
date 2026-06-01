/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for shared misc codec errors.

use std::error::Error;

use qubit_codec::{
    DecodeErrorInfo,
    DecodeFailure,
};
use qubit_codec_misc::{
    MiscCodecError,
    MiscCodecResult,
};

#[test]
fn test_misc_misc_codec_error_display_messages_include_context() {
    let cases = [
        (
            MiscCodecError::MissingPrefix {
                prefix: "0x".to_owned(),
            },
            "missing required prefix '0x'",
        ),
        (
            MiscCodecError::InvalidDigit {
                radix: 16,
                index: 3,
                character: 'g',
            },
            "invalid radix-16 digit 'g' at index 3",
        ),
        (
            MiscCodecError::InvalidLength {
                context: "hex digits",
                expected: "even number".to_owned(),
                actual: 3,
            },
            "invalid length for hex digits: expected even number, got 3",
        ),
        (
            MiscCodecError::InvalidEscape {
                index: 1,
                escape: "%z".to_owned(),
                reason: "expected two hex digits".to_owned(),
            },
            "invalid escape \"%z\" at index 1: expected two hex digits",
        ),
        (
            MiscCodecError::InvalidCharacter {
                index: 5,
                character: ' ',
                reason: "space is not allowed".to_owned(),
            },
            "invalid character ' ' at index 5: space is not allowed",
        ),
        (
            MiscCodecError::InvalidInput {
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
fn test_misc_misc_codec_error_wraps_utf8_source_error() {
    let error = String::from_utf8(vec![0xff]).expect_err("invalid utf-8 should fail");
    let error = MiscCodecError::from(error);

    assert_eq!(
        "decoded bytes are not valid UTF-8: invalid utf-8 sequence of 1 bytes from index 0",
        error.to_string()
    );
    assert!(error.source().is_some());
}

#[test]
fn test_misc_codec_result_alias_uses_misc_codec_error() {
    fn decode_stub() -> MiscCodecResult<()> {
        Err(MiscCodecError::MissingPrefix { prefix: "#".to_owned() })
    }

    let error = decode_stub().expect_err("stub should return misc codec error");

    assert!(matches!(error, MiscCodecError::MissingPrefix { prefix } if prefix == "#"));
}

#[test]
fn test_misc_codec_error_reports_incomplete_decode_failure() {
    let error = MiscCodecError::Incomplete {
        required: 8,
        available: 3,
    };

    assert_eq!(
        DecodeFailure::Incomplete {
            required_total: 8,
            available: 3,
        },
        error.failure()
    );
}

#[test]
fn test_misc_codec_error_reports_escape_decode_failure_length() {
    let error = MiscCodecError::InvalidEscape {
        index: 1,
        escape: "%7G".to_owned(),
        reason: "expected two hex digits".to_owned(),
    };
    let empty_escape = MiscCodecError::InvalidEscape {
        index: 1,
        escape: String::new(),
        reason: "missing escape sequence".to_owned(),
    };

    assert_eq!(DecodeFailure::Invalid { consumed: 3 }, error.failure());
    assert_eq!(DecodeFailure::Invalid { consumed: 1 }, empty_escape.failure());
}

#[test]
fn test_misc_codec_error_reports_single_unit_invalid_decode_failure() {
    let utf8_error = String::from_utf8(vec![0xff]).expect_err("invalid utf-8 should fail");
    let cases = [
        MiscCodecError::MissingPrefix {
            prefix: "0x".to_owned(),
        },
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 3,
            character: 'g',
        },
        MiscCodecError::InvalidLength {
            context: "hex digits",
            expected: "even number".to_owned(),
            actual: 3,
        },
        MiscCodecError::InvalidCharacter {
            index: 5,
            character: ' ',
            reason: "space is not allowed".to_owned(),
        },
        MiscCodecError::InvalidInput {
            codec: "base64",
            reason: "invalid symbol".to_owned(),
        },
        MiscCodecError::InvalidUtf8 { source: utf8_error },
    ];

    for error in cases {
        assert_eq!(DecodeFailure::Invalid { consumed: 1 }, error.failure());
    }
}
