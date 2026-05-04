/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for percent encoding.

use qubit_codec::{
    CodecError,
    PercentCodec,
};

#[test]
fn test_percent_codec_encodes_utf8_and_leaves_unreserved_ascii() {
    let codec = PercentCodec::new();

    assert_eq!("a-z_A.Z~0%201%2F%E4%B8%AD", codec.encode("a-z_A.Z~0 1/中"));
    assert_eq!("%01%09%1C", codec.encode("\u{0001}\t\u{001c}"));
}

#[test]
fn test_percent_codec_decodes_utf8_text() {
    let decoded = PercentCodec::new()
        .decode("a%20b%2Fc%E4%B8%AD")
        .expect("percent text should decode");

    assert_eq!("a b/c中", decoded);
}

#[test]
fn test_percent_codec_reports_bad_escape_and_utf8() {
    let short = PercentCodec::new()
        .decode("abc%")
        .expect_err("truncated escape should fail");
    assert!(matches!(
        short,
        CodecError::InvalidPercentEscape { index: 3 }
    ));

    let bad_hex = PercentCodec::new()
        .decode("%zz")
        .expect_err("bad hex escape should fail");
    assert!(matches!(
        bad_hex,
        CodecError::InvalidPercentEscape { index: 0 }
    ));

    let bad_utf8 = PercentCodec::new()
        .decode("%FF")
        .expect_err("invalid utf-8 should fail");
    assert!(matches!(bad_utf8, CodecError::InvalidUtf8 { .. }));
}
