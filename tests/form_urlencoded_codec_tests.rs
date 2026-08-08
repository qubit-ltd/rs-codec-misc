// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for application/x-www-form-urlencoded text encoding.

use qubit_codec::Codec;
use qubit_codec_misc::FormUrlencodedCodec;
use qubit_codec_misc::MiscCodecError;

#[test]
fn test_form_urlencoded_codec_uses_plus_for_spaces() {
    let codec = FormUrlencodedCodec::new();

    assert_eq!(
        "name%3Da+b%2Bc%26city%3D%E4%B8%8A%E6%B5%B7",
        codec.encode("name=a b+c&city=上海")
    );
    assert_eq!(
        "name=a b+c&city=上海",
        codec
            .decode("name%3Da+b%2Bc%26city%3D%E4%B8%8A%E6%B5%B7")
            .expect("form text should decode")
    );
}

#[test]
fn test_form_urlencoded_codec_matches_whatwg_byte_serializer() {
    let codec = FormUrlencodedCodec::new();
    let text = "*~-._!\'() +/\u{4e2d}";
    let expected: String = form_urlencoded::byte_serialize(text.as_bytes()).collect();

    assert_eq!(expected, codec.encode(text));
    assert_eq!("*%7E-._%21%27%28%29+%2B%2F%E4%B8%AD", expected);
}

#[test]
fn test_form_urlencoded_codec_uses_form_unescaped_set_for_low_level_encode() {
    let mut codec = FormUrlencodedCodec::new();
    let mut output = [0u8; 3];

    let written = unsafe {
        Codec::encode(&mut codec, &b'*', &mut output, 0).expect("form '*' should be unescaped")
    };
    assert_eq!(1, written);
    assert_eq!(b'*', output[0]);

    let written = unsafe {
        Codec::encode(&mut codec, &b'~', &mut output, 0).expect("form '~' should be escaped")
    };
    assert_eq!(3, written);
    assert_eq!(b"%7E", &output);
}

#[test]
fn test_form_urlencoded_codec_reports_truncated_escape_at_eof() {
    let error = FormUrlencodedCodec::new()
        .decode("field=%A")
        .expect_err("truncated form escape should fail");

    assert!(matches!(
        error,
        MiscCodecError::InvalidEscape {
            index: 6,
            escape,
            reason: _
        } if escape == "%A"
    ));
}
