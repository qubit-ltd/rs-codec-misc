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

use qubit_codec_misc::{
    MiscCodecError,
    PercentCodec,
    ValueDecoder,
    ValueEncoder,
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
        MiscCodecError::Incomplete {
            required: 3,
            available: 1
        }
    ));

    let bad_hex = PercentCodec::new()
        .decode("%zz")
        .expect_err("bad hex escape should fail");
    assert!(matches!(
        bad_hex,
        MiscCodecError::InvalidEscape {
            index: 0,
            escape: _,
            reason: _
        }
    ));

    let bad_low_hex = PercentCodec::new()
        .decode("%0z")
        .expect_err("bad low hex digit should fail");
    assert!(matches!(
        bad_low_hex,
        MiscCodecError::InvalidEscape {
            index: 0,
            escape: _,
            reason: _
        }
    ));

    let bad_utf8 = PercentCodec::new()
        .decode("%FF")
        .expect_err("invalid utf-8 should fail");
    assert!(matches!(bad_utf8, MiscCodecError::InvalidUtf8 { .. }));
}

#[test]
fn test_percent_codec_default_and_trait_methods() {
    let codec = PercentCodec;
    let encoded = ValueEncoder::<str>::encode(&codec, "a b").expect("percent encode should succeed");
    let decoded = ValueDecoder::<str>::decode(&codec, &encoded).expect("percent decode should succeed");

    assert_eq!("a%20b", encoded);
    assert_eq!("a b", decoded);
}
