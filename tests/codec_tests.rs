/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the bidirectional codec trait.

use qubit_codec_misc::{
    Base64QuantumCodec,
    CStringLiteralCodec,
    Codec,
    FormUrlencodedCodec,
    HexCodec,
    MiscCodecError,
    PercentCodec,
    ValueDecoder,
    ValueEncoder,
};

#[test]
fn test_codec_trait_decodes_and_encodes_single_hex_byte() {
    let codec = HexCodec::upper().with_byte_prefix("0x").with_separator(":");
    let mut output = [0u8; 2];

    let (decoded, consumed) =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"Af", 0).expect("single hex byte should decode") };
    let written = unsafe {
        Codec::<u8, u8>::encode_unchecked(&codec, &0xaf, &mut output, 0).expect("single hex byte should encode")
    };

    assert_eq!(0xaf, decoded);
    assert_eq!(2, consumed.get());
    assert_eq!(2, written);
    assert_eq!(b"AF", &output);
    assert_eq!(2, Codec::<u8, u8>::min_units_per_value(&codec).get());
    assert_eq!(2, Codec::<u8, u8>::max_units_per_value(&codec).get());
}

#[test]
fn test_codec_trait_reports_single_hex_byte_errors() {
    let codec = HexCodec::new();

    let high =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"xf", 0) }.expect_err("invalid high hex digit should fail");
    let low =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"fx", 0) }.expect_err("invalid low hex digit should fail");

    assert!(matches!(
        high,
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 0,
            character: 'x'
        }
    ));
    assert!(matches!(
        low,
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 1,
            character: 'x'
        }
    ));
}

#[test]
fn test_codec_trait_decodes_and_encodes_percent_byte() {
    let codec = PercentCodec::new();
    let mut escaped = [0u8; 3];
    let mut raw = [0u8; 1];

    let (decoded_escape, escape_units) =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"%E4", 0).expect("percent escape should decode") };
    let (decoded_raw, raw_units) =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"~", 0).expect("unreserved byte should decode") };
    let escaped_units = unsafe {
        Codec::<u8, u8>::encode_unchecked(&codec, &0xe4, &mut escaped, 0).expect("escaped byte should encode")
    };
    let unreserved_units = unsafe {
        Codec::<u8, u8>::encode_unchecked(&codec, &b'~', &mut raw, 0).expect("unreserved byte should encode")
    };

    assert_eq!(0xe4, decoded_escape);
    assert_eq!(3, escape_units.get());
    assert_eq!(b'~', decoded_raw);
    assert_eq!(1, raw_units.get());
    assert_eq!(3, escaped_units);
    assert_eq!(b"%E4", &escaped);
    assert_eq!(1, unreserved_units);
    assert_eq!(b"~", &raw);
    assert_eq!(1, Codec::<u8, u8>::min_units_per_value(&codec).get());
    assert_eq!(3, Codec::<u8, u8>::max_units_per_value(&codec).get());
}

#[test]
fn test_codec_trait_decodes_available_percent_byte() {
    let codec = PercentCodec::new();

    let raw = unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"A", 0) }.expect("raw byte should decode");
    let (decoded, consumed) = raw;
    assert_eq!(b'A', decoded);
    assert_eq!(1, consumed.get());

    let incomplete = unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"%E", 0) }
        .expect_err("partial percent escape should be incomplete");
    assert!(matches!(
        incomplete,
        MiscCodecError::Incomplete {
            required: 3,
            available: 2
        }
    ));

    let malformed = unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"%Ez", 0) }
        .expect_err("malformed percent escape should fail");
    assert!(matches!(malformed, MiscCodecError::InvalidEscape { index: 0, .. }));
}

#[test]
fn test_codec_trait_decodes_and_encodes_form_urlencoded_byte() {
    let codec = FormUrlencodedCodec::new();
    let mut plus_output = [0u8; 1];
    let mut raw_output = [0u8; 1];
    let mut escaped_output = [0u8; 3];

    let (decoded_plus, consumed) =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"+", 0).expect("form plus should decode as space") };
    let (decoded_escape, escape_consumed) =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"%E4", 0).expect("form escape should decode") };
    let (decoded_raw, raw_consumed) =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"~", 0).expect("form raw byte should decode") };
    let plus_written = unsafe {
        Codec::<u8, u8>::encode_unchecked(&codec, &b' ', &mut plus_output, 0).expect("space should encode as plus")
    };
    let raw_written = unsafe {
        Codec::<u8, u8>::encode_unchecked(&codec, &b'~', &mut raw_output, 0).expect("raw byte should encode")
    };
    let escaped_written = unsafe {
        Codec::<u8, u8>::encode_unchecked(&codec, &0xe4, &mut escaped_output, 0).expect("escaped byte should encode")
    };

    assert_eq!(b' ', decoded_plus);
    assert_eq!(1, consumed.get());
    assert_eq!(0xe4, decoded_escape);
    assert_eq!(3, escape_consumed.get());
    assert_eq!(b'~', decoded_raw);
    assert_eq!(1, raw_consumed.get());
    assert_eq!(1, plus_written);
    assert_eq!(b"+", &plus_output);
    assert_eq!(1, raw_written);
    assert_eq!(b"~", &raw_output);
    assert_eq!(3, escaped_written);
    assert_eq!(b"%E4", &escaped_output);
    assert_eq!(1, Codec::<u8, u8>::min_units_per_value(&codec).get());
    assert_eq!(3, Codec::<u8, u8>::max_units_per_value(&codec).get());
}

#[test]
fn test_codec_trait_decodes_available_form_urlencoded_byte() {
    let codec = FormUrlencodedCodec::new();

    let plus = unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"+", 0) }.expect("plus should decode to space");
    let (decoded, consumed) = plus;
    assert_eq!(b' ', decoded);
    assert_eq!(1, consumed.get());

    let incomplete = unsafe { Codec::<u8, u8>::decode_unchecked(&codec, b"%", 0) }
        .expect_err("partial form escape should be incomplete");
    assert!(matches!(
        incomplete,
        MiscCodecError::Incomplete {
            required: 3,
            available: 1
        }
    ));
}

#[test]
fn test_codec_trait_decodes_and_encodes_c_string_literal_byte() {
    let codec = CStringLiteralCodec::new();
    let mut escaped = [0u8; 4];
    let mut simple = [0u8; 2];

    let (decoded_hex, hex_units) =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\xD0", 0).expect("hex byte escape should decode") };
    let (decoded_newline, newline_units) =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\n", 0).expect("simple escape should decode") };
    let escaped_units = unsafe {
        Codec::<u8, u8>::encode_unchecked(&codec, &0xd0, &mut escaped, 0).expect("non-printable byte should encode")
    };
    let simple_units =
        unsafe { Codec::<u8, u8>::encode_unchecked(&codec, &b'\n', &mut simple, 0).expect("newline should encode") };

    assert_eq!(0xd0, decoded_hex);
    assert_eq!(4, hex_units.get());
    assert_eq!(b'\n', decoded_newline);
    assert_eq!(2, newline_units.get());
    assert_eq!(4, escaped_units);
    assert_eq!(br"\xD0", &escaped);
    assert_eq!(2, simple_units);
    assert_eq!(br"\n", &simple);
    assert_eq!(1, Codec::<u8, u8>::min_units_per_value(&codec).get());
    assert_eq!(10, Codec::<u8, u8>::max_units_per_value(&codec).get());
}

#[test]
fn test_codec_trait_decodes_available_c_string_literal_byte() {
    let codec = CStringLiteralCodec::new();

    let raw = unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"A", 0) }.expect("raw C string byte should decode");
    let (decoded, consumed) = raw;
    assert_eq!(b'A', decoded);
    assert_eq!(1, consumed.get());

    let simple =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\n", 0) }.expect("simple C escape should decode");
    let (decoded, consumed) = simple;
    assert_eq!(b'\n', decoded);
    assert_eq!(2, consumed.get());

    let eof_hex =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\xA", 0) }.expect("EOF-closed hex escape should decode");
    let (decoded, consumed) = eof_hex;
    assert_eq!(0x0a, decoded);
    assert_eq!(3, consumed.get());

    let terminated_hex =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\xAG", 0) }.expect("terminated hex escape should decode");
    let (decoded, consumed) = terminated_hex;
    assert_eq!(0x0a, decoded);
    assert_eq!(3, consumed.get());

    let eof_octal = unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\12", 0) }
        .expect("EOF-closed octal escape should decode");
    let (decoded, consumed) = eof_octal;
    assert_eq!(0o12, decoded);
    assert_eq!(3, consumed.get());

    let terminated_octal = unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\12G", 0) }
        .expect("terminated octal escape should decode");
    let (decoded, consumed) = terminated_octal;
    assert_eq!(0o12, decoded);
    assert_eq!(3, consumed.get());

    let malformed =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\z", 0) }.expect_err("unsupported C escape should fail");
    assert!(matches!(malformed, MiscCodecError::InvalidEscape { index: 0, .. }));
}

#[test]
fn test_codec_trait_decodes_c_string_literal_escape_variants() {
    let codec = CStringLiteralCodec::new();
    let cases: &[(&[u8], u8, usize)] = &[
        (br"A", b'A', 1),
        (br"\ ", b' ', 2),
        (br"\'", b'\'', 2),
        (br#"\""#, b'"', 2),
        (br"\?", b'?', 2),
        (br"\\", b'\\', 2),
        (br"\a", 0x07, 2),
        (br"\b", 0x08, 2),
        (br"\f", 0x0c, 2),
        (br"\r", b'\r', 2),
        (br"\t", b'\t', 2),
        (br"\v", 0x0b, 2),
        (br"\xA", 0x0a, 3),
        (br"\x1Z", 0x01, 3),
        (br"\u0022", b'"', 6),
        (br"\U00000021", b'!', 10),
        (br"\377", 0xff, 4),
        (br"\7", 0x07, 2),
    ];

    for (input, expected, expected_units) in cases {
        let (decoded, consumed) =
            unsafe { Codec::<u8, u8>::decode_unchecked(&codec, input, 0).expect("C escape should decode") };
        assert_eq!(*expected, decoded, "input {input:?}");
        assert_eq!(*expected_units, consumed.get(), "input {input:?}");
    }
}

#[test]
fn test_codec_trait_reports_c_string_literal_byte_errors() {
    let codec = CStringLiteralCodec::new();
    let invalid_raw =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, &[0xff], 0) }.expect_err("invalid raw byte should fail");
    let unsupported =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\z", 0) }.expect_err("unsupported escape should fail");
    let missing_hex =
        unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\xz", 0) }.expect_err("missing hex digit should fail");
    let incomplete_universal = unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\u12", 0) }
        .expect_err("incomplete universal escape should fail");
    let invalid_universal_digit = unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\u00zz", 0) }
        .expect_err("invalid universal digit should fail");
    let oversized_universal = unsafe { Codec::<u8, u8>::decode_unchecked(&codec, br"\u0100", 0) }
        .expect_err("oversized universal escape should fail");

    assert!(matches!(invalid_raw, MiscCodecError::InvalidCharacter { index: 0, .. }));
    assert!(matches!(unsupported, MiscCodecError::InvalidEscape { index: 0, .. }));
    assert!(matches!(missing_hex, MiscCodecError::InvalidEscape { index: 0, .. }));
    assert!(matches!(
        incomplete_universal,
        MiscCodecError::Incomplete {
            required: 6,
            available: 4
        }
    ));
    assert!(matches!(
        invalid_universal_digit,
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 4,
            character: 'z'
        }
    ));
    assert!(matches!(
        oversized_universal,
        MiscCodecError::InvalidEscape { index: 0, .. }
    ));
}

#[test]
fn test_codec_trait_encodes_c_string_literal_escape_variants() {
    let codec = CStringLiteralCodec::new();
    let cases: &[(u8, &[u8])] = &[
        (b'A', b"A"),
        (b'\'', br"\'"),
        (b'"', br#"\""#),
        (b'?', br"\?"),
        (b'\\', br"\\"),
        (0x07, br"\a"),
        (0x08, br"\b"),
        (0x0c, br"\f"),
        (b'\r', br"\r"),
        (b'\t', br"\t"),
        (0x0b, br"\v"),
        (0xff, br"\xFF"),
    ];

    for (byte, expected) in cases {
        let mut output = [0u8; 4];
        let written = unsafe {
            Codec::<u8, u8>::encode_unchecked(&codec, byte, &mut output, 0)
                .expect("C string literal byte should encode")
        };
        assert_eq!(*expected, &output[..written], "byte {byte:#04x}");
    }
}

#[test]
fn test_codec_trait_decodes_and_encodes_base64_quantum() {
    let codec = Base64QuantumCodec::standard();
    let mut output = [0u8; 4];

    let (decoded, consumed) =
        unsafe { Codec::<[u8; 3], u8>::decode_unchecked(&codec, b"YWJj", 0).expect("base64 quantum should decode") };
    let written = unsafe {
        Codec::<[u8; 3], u8>::encode_unchecked(&codec, b"abc", &mut output, 0).expect("base64 quantum should encode")
    };

    assert_eq!(*b"abc", decoded);
    assert_eq!(4, consumed.get());
    assert_eq!(4, written);
    assert_eq!(b"YWJj", &output);
    assert_eq!(4, Codec::<[u8; 3], u8>::min_units_per_value(&codec).get());
    assert_eq!(4, Codec::<[u8; 3], u8>::max_units_per_value(&codec).get());
}

#[test]
fn test_codec_trait_decodes_and_encodes_url_safe_base64_quantum() {
    let codec = Base64QuantumCodec::url_safe();
    let mut output = [0u8; 4];

    let (decoded, consumed) =
        unsafe { Codec::<[u8; 3], u8>::decode_unchecked(&codec, b"-__u", 0).expect("URL-safe quantum should decode") };
    let written = unsafe {
        Codec::<[u8; 3], u8>::encode_unchecked(&codec, &[0xfb, 0xff, 0xee], &mut output, 0)
            .expect("URL-safe quantum should encode")
    };

    assert_eq!([0xfb, 0xff, 0xee], decoded);
    assert_eq!(4, consumed.get());
    assert_eq!(4, written);
    assert_eq!(b"-__u", &output);
}

#[test]
fn test_codec_trait_covers_base64_quantum_alphabet_and_errors() {
    let standard = Base64QuantumCodec::default();
    let url_safe = Base64QuantumCodec::url_safe();

    let (decoded, _) = unsafe { Codec::<[u8; 3], u8>::decode_unchecked(&standard, b"++//", 0) }
        .expect("standard symbols should decode");
    assert_eq!([0xfb, 0xef, 0xff], decoded);

    let (decoded, _) =
        unsafe { Codec::<[u8; 3], u8>::decode_unchecked(&standard, b"0123", 0) }.expect("digit symbols should decode");
    assert_eq!([0xd3, 0x5d, 0xb7], decoded);
    assert!(matches!(
        unsafe { Codec::<[u8; 3], u8>::decode_unchecked(&url_safe, b"@@@@", 0) }
            .expect_err("invalid Base64 quantum should fail"),
        MiscCodecError::InvalidInput {
            codec: "base64-quantum",
            ..
        }
    ));
}

#[test]
fn test_value_traits_remain_convenience_layer() {
    let codec = HexCodec::upper().with_byte_prefix("0x").with_separator(" ");

    let encoded = ValueEncoder::<[u8]>::encode(&codec, &[0x00, 0x7f, 0xff]).expect("hex value encode should work");
    let decoded = ValueDecoder::<str>::decode(&codec, &encoded).expect("hex value decode should work");

    assert_eq!("0x00 0x7F 0xFF", encoded);
    assert_eq!(vec![0x00, 0x7f, 0xff], decoded);
}
