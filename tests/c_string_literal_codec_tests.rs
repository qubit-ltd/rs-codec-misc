// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for C string literal byte encoding.

use qubit_codec_misc::{
    CStringLiteralCodec,
    Codec,
    MiscCodecError,
    ValueDecoder,
    ValueEncoder,
};

#[test]
fn test_decode_plain_text_and_simple_escapes() {
    let codec = CStringLiteralCodec::new();

    assert_eq!(
        b"PK\x03\x04".to_vec(),
        codec
            .decode(r"PK\003\004")
            .expect("mixed text and octal escapes should decode")
    );
    assert_eq!(
        b"line\nquote\"slash\\tab\tbell\x07backspace\x08".to_vec(),
        codec
            .decode(r#"line\nquote\"slash\\tab\tbell\abackspace\b"#)
            .expect("simple C escapes should decode")
    );
    assert_eq!(
        b"?'\x0b\x0c\r".to_vec(),
        codec
            .decode(r"\?\'\v\f\r")
            .expect("remaining simple escapes should decode")
    );
    assert_eq!(
        b"<!DOCTYPE xbel".to_vec(),
        codec
            .decode(r"<!DOCTYPE\ xbel")
            .expect("escaped space should match Java CStringLiteral")
    );
    assert_eq!(
        b"\t\n\x0b\x0c".to_vec(),
        codec
            .decode("\t\n\u{0b}\u{0c}")
            .expect("allowed raw whitespace source characters should decode")
    );
}

#[test]
fn test_decode_hex_octal_and_universal_escapes() {
    let codec = CStringLiteralCodec::new();

    assert_eq!(
        vec![0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1],
        codec
            .decode(r"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1")
            .expect("hex byte escapes should decode")
    );
    assert_eq!(
        vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'],
        codec
            .decode(r"\211PNG\r\n\032\n")
            .expect("freedesktop magic escapes should decode")
    );
    assert_eq!(
        b"A\"!".to_vec(),
        codec
            .decode(r"\x41\u0022\U00000021")
            .expect("hex and universal byte escapes should decode")
    );
    assert_eq!(
        vec![0x01, b'Z'],
        codec
            .decode(r"\x1Z")
            .expect("hex escape should consume at most two hex digits")
    );
    assert_eq!(
        vec![0x0a],
        codec
            .decode(r"\xA")
            .expect("hex escape should allow one digit at end of input")
    );
    assert_eq!(
        vec![0x0b],
        codec
            .decode(r"\XB")
            .expect("uppercase hex escape marker should decode")
    );
    assert_eq!(
        vec![0x07],
        codec
            .decode(r"\7")
            .expect("short octal escape at end of input should decode")
    );
}

#[test]
fn test_decode_matches_java_c_string_literal_cases() {
    let codec = CStringLiteralCodec::new();

    assert_eq!(
        b"hello, world.".to_vec(),
        codec
            .decode("hello, world.")
            .expect("plain Java fixture should decode")
    );
    assert_eq!(
        b"hello, \"world\".".to_vec(),
        codec
            .decode(r#"hello, \"world\"."#)
            .expect("quoted Java fixture should decode")
    );
    assert_eq!(
        b"hello, \"world\".".to_vec(),
        codec
            .decode(r"hello, \x22world\x22.")
            .expect("hex Java fixture should decode")
    );
    assert_eq!(
        b"hello, \"world\"@123.".to_vec(),
        codec
            .decode(r"hello, \42world\42\100123.")
            .expect("octal Java fixture should decode")
    );
    assert_eq!(
        b"hello, \"world\".".to_vec(),
        codec
            .decode(r"hello, \u0022world\u0022.")
            .expect("short universal Java fixture should decode")
    );
    assert_eq!(
        b"hello, \"world\".".to_vec(),
        codec
            .decode(r"hello, \U00000022world\U00000022.")
            .expect("long universal Java fixture should decode")
    );
}

#[test]
fn test_decode_reports_invalid_escape_and_character_errors() {
    let trailing = CStringLiteralCodec::new()
        .decode(r"abc\")
        .expect_err("trailing escape marker should fail");
    assert!(matches!(
        trailing,
        MiscCodecError::InvalidEscape {
            index: 3,
            escape: _,
            reason: _
        }
    ));

    let invalid_escape = CStringLiteralCodec::new()
        .decode(r"\z")
        .expect_err("unsupported escape should fail");
    assert!(matches!(
        invalid_escape,
        MiscCodecError::InvalidEscape {
            index: 0,
            escape: _,
            reason: _
        }
    ));

    let missing_hex_digit = CStringLiteralCodec::new()
        .decode(r"\xz")
        .expect_err("hex escape without digits should fail");
    assert!(matches!(
        missing_hex_digit,
        MiscCodecError::InvalidEscape {
            index: 0,
            escape: _,
            reason: _
        }
    ));

    let incomplete_universal = CStringLiteralCodec::new()
        .decode(r"\u12")
        .expect_err("incomplete universal escape should fail");
    assert!(matches!(
        incomplete_universal,
        MiscCodecError::InvalidEscape {
            index: 0,
            escape: _,
            reason: _
        }
    ));

    let invalid_universal_digit = CStringLiteralCodec::new()
        .decode(r"\u00zz")
        .expect_err("invalid universal escape digit should fail");
    assert!(matches!(
        invalid_universal_digit,
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 4,
            character: 'z'
        }
    ));

    let unicode = CStringLiteralCodec::new()
        .decode("snowman: ☃")
        .expect_err("non-ASCII source character should fail");
    assert!(matches!(
        unicode,
        MiscCodecError::InvalidCharacter {
            index: 9,
            character: '☃',
            ..
        }
    ));

    let oversized = CStringLiteralCodec::new()
        .decode(r"\u0100")
        .expect_err("universal byte escape must fit in one byte");
    assert!(matches!(
        oversized,
        MiscCodecError::InvalidEscape {
            index: 0,
            escape: _,
            reason: _
        }
    ));
}

#[test]
fn test_encode_uses_simple_escapes_and_hex_bytes() {
    let codec = CStringLiteralCodec::new();

    assert_eq!(
        r#"quote\"apos\'question\?slash\\line\n"#,
        codec.encode(b"quote\"apos'question?slash\\line\n")
    );
    assert_eq!(
        r"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1",
        codec.encode(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1])
    );
    assert_eq!(
        r"\a\b\f\r\t\v",
        codec.encode(&[0x07, 0x08, 0x0c, b'\r', b'\t', 0x0b])
    );
    assert_eq!("", codec.encode(&[]));
    assert_eq!(
        r"\x02\x05\x06\x17\x18\x19",
        codec.encode(&[0x02, 0x05, 0x06, 0x17, 0x18, 0x19])
    );
}

#[test]
fn test_c_string_literal_codec_can_be_used_through_traits() {
    let mut codec = CStringLiteralCodec::new();
    let encoded = ValueEncoder::<[u8]>::encode(&mut codec, b"PK\x03\x04")
        .expect("C string literal encode should succeed");
    let decoded = ValueDecoder::<str>::decode(&mut codec, &encoded)
        .expect("C string literal decode should succeed");

    assert_eq!(r"PK\x03\x04", encoded);
    assert_eq!(b"PK\x03\x04".to_vec(), decoded);
}

#[test]
fn test_decode_matches_codec_trait_path_for_complete_fragments() {
    let mut codec = CStringLiteralCodec::new();
    let cases = [
        "",
        "plain text",
        r"PK\003\004",
        r#"quote\"apos\'question\?slash\\"#,
        r"\a\b\f\n\r\t\v",
        r"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1",
        r"\x41\u0022\U00000021",
        r"\x1Z\7\377",
        "<!DOCTYPE\\ xbel",
    ];

    for input in cases {
        let owned = codec
            .decode(input)
            .expect("owned C string literal decoder should accept fixture");
        let codec_trait =
            decode_complete_fragment_through_codec_trait(&mut codec, input)
                .expect("Codec trait path should accept fixture");

        assert_eq!(owned, codec_trait, "input {input:?}");
    }
}

#[test]
fn test_c_string_literal_codec_uses_shared_parser_core() {
    let source = include_str!("../src/c_string_literal_codec.rs");

    assert!(
        source.contains("fn decode_c_string_literal_unit("),
        "C string literal decoding should have one shared unit parser"
    );
    for removed_function in [
        "fn decode_escape(",
        "fn parse_variable_hex_escape(",
        "fn parse_fixed_hex_escape(",
        "fn parse_octal_escape(",
        "fn validate_source_character(",
    ] {
        assert!(
            !source.contains(removed_function),
            "owned decode should not keep the old char-oriented parser function {removed_function}"
        );
    }
}

fn decode_complete_fragment_through_codec_trait(
    codec: &mut CStringLiteralCodec,
    input: &str,
) -> Result<Vec<u8>, MiscCodecError> {
    let mut output = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut input_index = 0;
    while input_index < bytes.len() {
        let (decoded, consumed) =
            unsafe { Codec::decode(codec, bytes, input_index) }.map_err(
                |failure| match failure {
                    qubit_codec::DecodeFailure::Invalid { source, .. } => {
                        source
                    }
                    qubit_codec::DecodeFailure::Incomplete {
                        required_total,
                    } => MiscCodecError::Incomplete {
                        required: required_total,
                        available: bytes.len().saturating_sub(input_index),
                    },
                },
            )?;
        output.push(decoded);
        input_index += consumed.get();
    }
    Ok(output)
}
