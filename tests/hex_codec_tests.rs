// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for hexadecimal byte encoding.

use qubit_codec_misc::HexCodec;
use qubit_codec_misc::HexCodecConfig;
use qubit_codec_misc::MiscCodecError;

#[test]
fn test_hex_codec_reuses_named_configuration() {
    let config = HexCodecConfig::new()
        .with_uppercase(true)
        .with_prefix("0x")
        .with_separator(" ")
        .with_ignored_ascii_whitespace(true)
        .with_ignore_prefix_case(true);
    let codec = HexCodec::from_config(config.clone());

    assert_eq!(&config, codec.config());
    assert!(config.is_uppercase());
    assert_eq!(Some("0x"), config.prefix());
    assert_eq!(Some(" "), config.separator());
    assert!(config.ignores_ascii_whitespace());
    assert!(config.ignores_prefix_case());
    assert_eq!("0x1F 8B", codec.encode(&[0x1f, 0x8b]));
    assert_eq!(
        vec![0x1f, 0x8b],
        codec.decode(" 0X1F 8B ").expect("configured hex should decode")
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

    assert_eq!("0x1F 0x8B 0x00 0xFF", codec.encode(&[0x1f, 0x8b, 0x00, 0xff]));
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
fn test_decode_into_is_atomic_when_later_input_is_invalid() {
    let codec = HexCodec::new();
    let mut bytes = vec![0x7f];

    let error = codec
        .decode_into("abcd0g", &mut bytes)
        .expect_err("invalid trailing digit should fail");

    assert!(matches!(
        error,
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 5,
            character: 'g'
        }
    ));
    assert_eq!(vec![0x7f], bytes);
}

#[test]
fn test_decode_into_appends_successes_and_preserves_output_on_errors() {
    let plain = HexCodec::new();
    let mut output = vec![0xa5];
    plain
        .decode_into("1f8b", &mut output)
        .expect("valid input should append");
    assert_eq!(vec![0xa5, 0x1f, 0x8b], output);

    plain
        .decode_into("", &mut output)
        .expect("empty input should append no bytes");
    assert_eq!(vec![0xa5, 0x1f, 0x8b], output);

    let before = output.clone();
    let error = plain
        .decode_into("abcd0g", &mut output)
        .expect_err("invalid digit should fail");
    assert!(matches!(
        error,
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 5,
            character: 'g'
        }
    ));
    assert_eq!(before, output, "invalid digit must roll back appended bytes");

    let error = plain
        .decode_into("abc", &mut output)
        .expect_err("odd digit count should fail");
    assert!(matches!(
        error,
        MiscCodecError::InvalidLength {
            context: "hex digits",
            actual: 3,
            ..
        }
    ));
    assert_eq!(before, output, "odd digit count must roll back appended bytes");

    let byte_prefixed = HexCodec::new().with_byte_prefix("0x");
    let cases = [("0x1f8b", "missing second byte prefix")];
    for (text, case) in cases {
        let before = output.clone();
        let error = byte_prefixed.decode_into(text, &mut output).expect_err(case);
        assert!(matches!(error, MiscCodecError::MissingPrefix { .. }), "{case}");
        assert_eq!(before, output, "{case} must roll back appended bytes");
    }

    let separated = HexCodec::new().with_separator(":");
    let error = separated
        .decode_into("1f:8g", &mut output)
        .expect_err("invalid digit after a complete byte should fail");
    assert!(matches!(
        error,
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 4,
            character: 'g'
        }
    ));
    assert_eq!(before, output, "invalid digit must roll back appended bytes");

    for (text, reason) in [("1f8b", "missing separator"), ("1f:", "trailing separator")] {
        let error = separated.decode_into(text, &mut output).expect_err(reason);
        assert!(matches!(error, MiscCodecError::InvalidInput { codec: "hex", .. }));
        assert_eq!(before, output, "{reason} must roll back appended bytes");
    }

    let prefixed = HexCodec::new().with_prefix("#");
    let before = output.clone();
    let error = prefixed
        .decode_into("1f", &mut output)
        .expect_err("missing whole prefix should fail");
    assert!(matches!(error, MiscCodecError::MissingPrefix { .. }));
    assert_eq!(before, output);
}

#[test]
fn test_decode_plain_prefixed_and_separated_hex() {
    assert_eq!(
        vec![0x1f, 0x8b, 0x00, 0xff],
        HexCodec::new().decode("1f8B00ff").expect("plain hex should decode")
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
        codec.decode("1f").expect("single byte should not require a separator")
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
    let colon_codec = HexCodec::new().with_separator(":").with_ignored_ascii_whitespace(true);
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
fn test_whitespace_separator_accepts_surrounding_ignored_whitespace() {
    for separator in ["\t", " \t"] {
        let codec = HexCodec::new()
            .with_separator(separator)
            .with_ignored_ascii_whitespace(true);
        for text in [
            format!("1f{separator}8b"),
            format!("1f {separator}8b"),
            format!("1f{separator} 8b"),
            format!("1f {separator} 8b\n"),
        ] {
            assert_eq!(vec![0x1f, 0x8b], codec.decode(&text).unwrap(), "{text:?}");
        }
        for text in ["1f8b", "1f 8b", "1 f\t8b", "1f\t8g"] {
            let mut output = vec![0xa5];
            assert!(codec.decode_into(text, &mut output).is_err(), "{text:?}");
            assert_eq!(vec![0xa5], output);
        }
        let strict = HexCodec::new().with_separator(separator);
        assert_eq!(vec![0x1f, 0x8b], strict.decode(&format!("1f{separator}8b")).unwrap());
        assert!(strict.decode(&format!("1f {separator}8b")).is_err());
        assert!(strict.decode(&format!("1f{separator} 8b")).is_err());
    }
}

#[test]
fn test_hex_roundtrip_preserves_whitespace_in_structural_tokens() {
    let prefixed = HexCodec::new().with_prefix(" 0x").with_ignored_ascii_whitespace(true);
    let prefixed_text = prefixed.encode(&[0x1f]);
    assert_eq!(" 0x1f", prefixed_text);
    assert_eq!(
        vec![0x1f],
        prefixed
            .decode(&prefixed_text)
            .expect("prefix whitespace should round-trip")
    );

    let separated = HexCodec::new()
        .with_separator(" : ")
        .with_ignored_ascii_whitespace(true);
    let separated_text = separated.encode(&[0x1f, 0x8b]);
    assert_eq!("1f : 8b", separated_text);
    assert_eq!(
        vec![0x1f, 0x8b],
        separated
            .decode(&separated_text)
            .expect("separator whitespace should round-trip")
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

    let invalid = HexCodec::new().decode("12xz").expect_err("invalid digit should fail");
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
    assert!(matches!(missing_prefix, MiscCodecError::MissingPrefix { .. }));

    let missing_second_prefix = HexCodec::new()
        .with_byte_prefix("0x")
        .decode("0x1f8b")
        .expect_err("each byte should require its own prefix");
    assert!(matches!(missing_second_prefix, MiscCodecError::MissingPrefix { .. }));

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
    assert!(matches!(too_short_prefix, MiscCodecError::MissingPrefix { .. }));
}

#[test]
fn test_decode_rejects_byte_prefix_without_a_hex_byte() {
    let codec = HexCodec::new().with_byte_prefix("0x");

    let error = codec
        .decode("0x")
        .expect_err("a byte prefix must be followed by two hex digits");

    assert!(matches!(error, MiscCodecError::InvalidInput { codec: "hex", .. }));
}

#[test]
fn test_byte_prefix_whitespace_roundtrips() {
    for prefix in [" 0x", "\t0x", " "] {
        for separator in [None, Some(":"), Some(" "), Some(" : ")] {
            for ignore in [false, true] {
                for bytes in [vec![], vec![0x1f], vec![0x1f, 0x8b]] {
                    let mut codec = HexCodec::new()
                        .with_prefix("#")
                        .with_byte_prefix(prefix)
                        .with_ignored_ascii_whitespace(ignore);
                    if let Some(separator) = separator {
                        codec = codec.with_separator(separator);
                    }
                    let encoded = codec.encode(&bytes);
                    assert_eq!(
                        bytes,
                        codec.decode(&encoded).unwrap(),
                        "prefix={prefix:?}, separator={separator:?}, ignore={ignore}"
                    );
                }
            }
        }
    }
}

#[test]
fn test_byte_prefix_preserves_structure_after_extra_whitespace() {
    let codec = HexCodec::new()
        .with_byte_prefix(" 0x")
        .with_separator(":")
        .with_ignored_ascii_whitespace(true)
        .with_ignore_prefix_case(true);
    assert_eq!(vec![0x1f, 0x8b], codec.decode("\t 0X1f:\t 0X8b\n").unwrap());
    let mut output = vec![0xa5];
    let error = codec.decode_into(" 0x1f: 0x8g", &mut output).unwrap_err();
    assert!(matches!(
        error,
        MiscCodecError::InvalidDigit {
            index: 10,
            character: 'g',
            ..
        }
    ));
    assert_eq!(vec![0xa5], output);
}

#[test]
fn test_matched_whitespace_prefix_requires_a_complete_byte() {
    let codec = HexCodec::new()
        .with_byte_prefix(" ")
        .with_ignored_ascii_whitespace(true);
    assert!(matches!(
        codec.decode(" "),
        Err(MiscCodecError::InvalidInput { codec: "hex", .. })
    ));
}

#[test]
fn test_extra_whitespace_preserves_whole_prefix_and_separator() {
    let prefixed = HexCodec::new().with_prefix(" 0x").with_ignored_ascii_whitespace(true);
    assert_eq!(vec![0x1f], prefixed.decode("\t 0x1f").unwrap());
    let separated = HexCodec::new()
        .with_separator(" : ")
        .with_ignored_ascii_whitespace(true);
    assert_eq!(vec![0x1f, 0x8b], separated.decode("1f\t : 8b").unwrap());
}

#[test]
fn test_structural_whitespace_policy_combinations() {
    for prefix in [" 0x", "\t0x"] {
        for separator in [" : ", "\t:\t"] {
            for ignore in [false, true] {
                for bytes in [vec![], vec![0x1f], vec![0x1f, 0x8b]] {
                    let codec = HexCodec::new()
                        .with_prefix(prefix)
                        .with_separator(separator)
                        .with_ignore_prefix_case(true)
                        .with_ignored_ascii_whitespace(ignore);
                    let exact = codec.encode(&bytes);
                    assert_eq!(bytes, codec.decode(&exact).unwrap());
                    let decorated =
                        format!("\n{}", exact.replace(separator, &format!("\n{separator}"))).replace("0x", "0X");
                    if ignore {
                        assert_eq!(bytes, codec.decode(&decorated).unwrap());
                    } else {
                        assert!(codec.decode(&decorated).is_err());
                    }
                }
            }
        }
    }
    let codec = HexCodec::new()
        .with_separator(" : ")
        .with_ignored_ascii_whitespace(true);
    for text in ["1f8b", "1f : ", "1 f : 8b", "1f :8b"] {
        let mut output = vec![0xa5];
        assert!(codec.decode_into(text, &mut output).is_err(), "{text:?}");
        assert_eq!(vec![0xa5], output);
    }
}
