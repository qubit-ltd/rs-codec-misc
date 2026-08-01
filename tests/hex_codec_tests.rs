// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for hexadecimal byte encoding.

use qubit_codec_misc::{HexCodec, MiscCodecError};

#[test]
fn test_production_code_does_not_use_panic_helpers() {
    let sources = [
        include_str!("../src/base64_codec.rs"),
        include_str!("../src/base64_quantum_codec.rs"),
        include_str!("../src/c_integer_literal_codec.rs"),
        include_str!("../src/c_string_literal_codec.rs"),
        include_str!("../src/form_urlencoded_codec.rs"),
        include_str!("../src/hex_codec.rs"),
        include_str!("../src/lib.rs"),
        include_str!("../src/misc_codec_error.rs"),
        include_str!("../src/percent_codec.rs"),
    ];
    let combined = sources.join("\n");

    assert!(
        !combined.contains(".expect("),
        "production code should return MiscCodecError instead of panicking"
    );
    assert!(
        !combined.contains(".unwrap("),
        "production code should return MiscCodecError instead of panicking"
    );
    assert!(
        !combined.contains("panic!"),
        "production code should return MiscCodecError instead of panicking"
    );
    assert!(
        !combined.contains("unreachable!"),
        "production code should return MiscCodecError instead of panicking"
    );
    assert!(
        !combined.contains("todo!"),
        "production code should return MiscCodecError instead of panicking"
    );
    assert!(
        !combined.contains("unimplemented!"),
        "production code should return MiscCodecError instead of panicking"
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
fn test_encode_uppercase_with_whole_prefix_and_separator() {
    let codec = HexCodec::upper().with_prefix("0x").with_separator(" ");

    assert_eq!("0x1F 8B 00 FF", codec.encode(&[0x1f, 0x8b, 0x00, 0xff]));
    assert_eq!(
        "0x01 23 45 67 89 AB CD EF",
        codec.encode(&[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef])
    );
}

#[test]
fn test_encode_uppercase_with_byte_prefix_and_separator() {
    let codec = HexCodec::upper().with_byte_prefix("0x").with_separator(" ");

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
fn test_encode_combines_whole_prefix_and_byte_prefix() {
    let codec = HexCodec::new().with_prefix("#").with_byte_prefix("\\x");

    assert_eq!("#\\x1f\\x8b", codec.encode(&[0x1f, 0x8b]));
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
            .expect("whole-prefixed contiguous hex should decode")
    );

    assert_eq!(
        vec![0x1f, 0x8b],
        HexCodec::new()
            .with_prefix("0x")
            .with_separator(" ")
            .decode("0x1f 8b")
            .expect("whole-prefixed separated hex should decode")
    );

    assert_eq!(
        vec![0x1f, 0x8b],
        HexCodec::new()
            .with_byte_prefix("0x")
            .with_separator(" ")
            .with_ignored_ascii_whitespace(true)
            .decode(" \t0x1F 0x8B ")
            .expect("byte-prefixed hex should tolerate configured whitespace")
    );

    assert_eq!(
        vec![0x1f, 0x8b, 0x00],
        HexCodec::new()
            .with_separator(":")
            .with_ignored_ascii_whitespace(true)
            .decode("1f: 8B:\n00")
            .expect("separated hex should decode")
    );

    assert_eq!(
        vec![0x1f, 0x8b],
        HexCodec::new()
            .with_prefix("#")
            .with_byte_prefix("\\x")
            .decode("#\\x1f\\x8b")
            .expect("whole prefix and byte prefix should decode together")
    );

    assert_eq!(
        vec![0x1f, 0x8b],
        HexCodec::new()
            .with_prefix("0x")
            .with_ignore_prefix_case(true)
            .decode("0X1f8b")
            .expect("whole prefix should optionally ignore ASCII case")
    );

    assert_eq!(
        vec![0x1f, 0x8b],
        HexCodec::new()
            .with_byte_prefix("0x")
            .with_ignore_prefix_case(true)
            .decode("0X1f0X8b")
            .expect("byte prefix should optionally ignore ASCII case")
    );

    assert_eq!(
        vec![0x1f],
        HexCodec::new()
            .with_prefix("0x")
            .with_ignored_ascii_whitespace(true)
            .decode(" \t0x1f")
            .expect("whole prefix should tolerate configured leading whitespace")
    );
}

#[test]
fn test_decode_requires_configured_separator_between_bytes() {
    let codec = HexCodec::new().with_separator(":");

    assert_eq!(
        vec![0x1f, 0x8b, 0x00],
        codec
            .decode("1f:8b:00")
            .expect("configured separator should decode between complete bytes")
    );
    assert_eq!(
        Vec::<u8>::new(),
        codec
            .decode("")
            .expect("empty separated hex input should decode to empty bytes")
    );
    assert_eq!(
        vec![0x1f],
        codec
            .decode("1f")
            .expect("single byte should not require a separator")
    );
    assert!(
        codec.decode("1f8b").is_err(),
        "configured separator should be required between bytes"
    );
    assert!(
        codec.decode("1f:8b00").is_err(),
        "configured separator should be required between every byte pair"
    );

    let missing_low_digit = codec
        .decode("1")
        .expect_err("separated hex byte should require two digits");
    assert!(matches!(
        missing_low_digit,
        MiscCodecError::InvalidInput { codec: "hex", .. }
    ));
}

#[test]
fn test_decode_rejects_separator_outside_byte_boundaries() {
    let codec = HexCodec::new().with_separator(":");

    for input in [":1f", "1f:", "1f::8b", "1:f"] {
        assert!(
            codec.decode(input).is_err(),
            "separator should be rejected outside byte boundaries: {input}"
        );
    }
}

#[test]
fn test_decode_keeps_ignored_whitespace_outside_hex_bytes() {
    let colon_codec = HexCodec::new()
        .with_separator(":")
        .with_ignored_ascii_whitespace(true);
    let space_codec = HexCodec::new()
        .with_byte_prefix("0x")
        .with_separator(" ")
        .with_ignored_ascii_whitespace(true);

    assert_eq!(
        vec![0x1f, 0x8b],
        colon_codec
            .decode(" \t1f :\n8b ")
            .expect("ignored whitespace may surround bytes and separators")
    );
    assert_eq!(
        vec![0x1f, 0x8b],
        space_codec
            .decode(" \t0x1F 0x8B ")
            .expect("space separator should still work with ignored edge whitespace")
    );
    assert!(
        colon_codec.decode("1 f:8b").is_err(),
        "ignored whitespace should not split a hex byte"
    );
    assert!(
        space_codec.decode("0x1 F 0x8B").is_err(),
        "ignored whitespace should not split a byte-prefixed hex byte"
    );
    assert!(
        space_codec.decode("0x1F0x8B").is_err(),
        "space separator should be required between byte-prefixed bytes"
    );
}

#[test]
fn test_decode_ignores_configured_whitespace_without_separator() {
    assert_eq!(
        vec![0x1f, 0x8b],
        HexCodec::new()
            .with_ignored_ascii_whitespace(true)
            .decode("1f \n8B")
            .expect("unprefixed hex should ignore configured ASCII whitespace")
    );

    assert_eq!(
        vec![0x1f],
        HexCodec::new()
            .with_byte_prefix("0x")
            .with_ignored_ascii_whitespace(true)
            .decode("0x1 f \t")
            .expect("byte-prefixed hex should ignore configured whitespace")
    );
}

#[test]
fn test_decode_reports_precise_hex_errors() {
    let odd = HexCodec::new()
        .decode("abc")
        .expect_err("odd number of digits should fail");
    assert!(matches!(
        odd,
        MiscCodecError::InvalidLength {
            context: "hex digits",
            actual: 3,
            ..
        }
    ));

    let invalid = HexCodec::new()
        .decode("12xz")
        .expect_err("invalid digit should fail");
    assert!(matches!(
        invalid,
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 2,
            character: 'x'
        }
    ));

    let missing_prefix = HexCodec::new()
        .with_prefix("0x")
        .decode("1f")
        .expect_err("missing prefix should fail");
    assert!(matches!(
        missing_prefix,
        MiscCodecError::MissingPrefix { .. }
    ));

    let missing_second_prefix = HexCodec::new()
        .with_byte_prefix("0x")
        .decode("0x1f8b")
        .expect_err("each byte should require its own prefix");
    assert!(matches!(
        missing_second_prefix,
        MiscCodecError::MissingPrefix { .. }
    ));

    let invalid_after_prefix = HexCodec::new()
        .with_prefix("0x")
        .decode("0x1g")
        .expect_err("invalid digit after prefix should fail");
    assert!(matches!(
        invalid_after_prefix,
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 3,
            character: 'g'
        }
    ));

    let invalid_after_byte_prefix = HexCodec::new()
        .with_byte_prefix("0x")
        .decode("0x1g")
        .expect_err("invalid digit after byte prefix should fail");
    assert!(matches!(
        invalid_after_byte_prefix,
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 3,
            character: 'g'
        }
    ));

    let missing_separated_byte_prefix = HexCodec::new()
        .with_byte_prefix("0x")
        .with_separator(":")
        .decode("0x1f:8b")
        .expect_err("separated bytes should each require the byte prefix");
    assert!(matches!(
        missing_separated_byte_prefix,
        MiscCodecError::MissingPrefix { .. }
    ));

    let too_short_prefix = HexCodec::new()
        .with_prefix("0x")
        .with_ignore_prefix_case(true)
        .decode("0")
        .expect_err("short input should not match prefix");
    assert!(matches!(
        too_short_prefix,
        MiscCodecError::MissingPrefix { .. }
    ));
}
