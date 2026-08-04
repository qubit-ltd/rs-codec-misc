// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the bidirectional codec trait.

use qubit_codec::{
    Codec,
    ValueDecoder,
    ValueEncoder,
};
use qubit_codec_misc::{
    CStringLiteralCodec,
    FormUrlencodedCodec,
    HexCodec,
    MiscCodecError,
    PercentCodec,
};

#[test]
fn test_codec_trait_decodes_and_encodes_percent_byte() {
    let mut codec = PercentCodec::new();
    let mut escaped = [0u8; 3];
    let mut raw = [0u8; 1];

    let (decoded_escape, escape_units) = unsafe {
        Codec::decode(&mut codec, b"%E4", 0)
            .expect("percent escape should decode")
    };
    let (decoded_raw, raw_units) = unsafe {
        Codec::decode(&mut codec, b"~", 0)
            .expect("unreserved byte should decode")
    };
    let escaped_units = unsafe {
        Codec::encode(&mut codec, &0xe4, &mut escaped, 0)
            .expect("escaped byte should encode")
    };
    let unreserved_units = unsafe {
        Codec::encode(&mut codec, &b'~', &mut raw, 0)
            .expect("unreserved byte should encode")
    };

    assert_eq!(0xe4, decoded_escape);
    assert_eq!(3, escape_units.get());
    assert_eq!(b'~', decoded_raw);
    assert_eq!(1, raw_units.get());
    assert_eq!(3, escaped_units);
    assert_eq!(b"%E4", &escaped);
    assert_eq!(1, unreserved_units);
    assert_eq!(b"~", &raw);
    assert_eq!(1, <PercentCodec as Codec>::MIN_UNITS_PER_VALUE);
    assert_eq!(3, <PercentCodec as Codec>::MAX_ENCODE_UNITS_PER_VALUE);
    assert_eq!(3, <PercentCodec as Codec>::MAX_DECODE_UNITS_PER_VALUE);
}

#[test]
fn test_codec_trait_decodes_available_percent_byte() {
    let mut codec = PercentCodec::new();

    let raw = unsafe { Codec::decode(&mut codec, b"A", 0) }
        .expect("raw byte should decode");
    let (decoded, consumed) = raw;
    assert_eq!(b'A', decoded);
    assert_eq!(1, consumed.get());

    let incomplete = unsafe { Codec::decode(&mut codec, b"%E", 0) }
        .expect_err("partial percent escape should be incomplete");
    assert_eq!(3, super::incomplete_required(incomplete));

    let malformed = unsafe { Codec::decode(&mut codec, b"%Ez", 0) }
        .expect_err("malformed percent escape should fail");
    assert_eq!(Some(3), super::invalid_consumed(malformed));
    let malformed = unsafe { Codec::decode(&mut codec, b"%Ez", 0) }
        .expect_err("malformed percent escape should fail");
    let malformed = super::invalid_source(malformed);
    assert!(matches!(
        malformed,
        MiscCodecError::InvalidEscape { index: 0, .. }
    ));
}

#[test]
fn test_codec_trait_uses_exact_percent_widths_and_eof_rules() {
    let mut percent = PercentCodec::new();
    assert_eq!(1, Codec::encode_len(&percent, &b'A'));
    assert_eq!(3, Codec::encode_len(&percent, &0xe4));

    let (decoded, consumed) = unsafe {
        Codec::decode_eof(&mut percent, b"A", 0)
            .expect("EOF raw percent byte should decode")
    };
    assert_eq!(b'A', decoded);
    assert_eq!(1, consumed.get());

    let invalid = unsafe { Codec::decode_eof(&mut percent, b"%A", 0) }
        .expect_err("EOF truncated percent escape should fail");
    assert!(matches!(
        super::invalid_source(invalid),
        MiscCodecError::InvalidEscape { index: 0, .. }
    ));
    let invalid = unsafe { Codec::decode_eof(&mut percent, b"%", 0) }
        .expect_err("EOF percent marker should fail");
    assert_eq!(Some(1), super::invalid_consumed(invalid));
    let invalid = unsafe { Codec::decode_eof(&mut percent, b"%A", 0) }
        .expect_err("EOF partial percent escape should fail");
    assert_eq!(Some(2), super::invalid_consumed(invalid));
}

#[test]
fn test_codec_trait_decodes_and_encodes_form_urlencoded_byte() {
    let mut codec = FormUrlencodedCodec::new();
    let mut plus_output = [0u8; 1];
    let mut raw_output = [0u8; 3];
    let mut escaped_output = [0u8; 3];

    let (decoded_plus, consumed) = unsafe {
        Codec::decode(&mut codec, b"+", 0)
            .expect("form plus should decode as space")
    };
    let (decoded_escape, escape_consumed) = unsafe {
        Codec::decode(&mut codec, b"%E4", 0).expect("form escape should decode")
    };
    let (decoded_raw, raw_consumed) = unsafe {
        Codec::decode(&mut codec, b"~", 0).expect("form raw byte should decode")
    };
    let plus_written = unsafe {
        Codec::encode(&mut codec, &b' ', &mut plus_output, 0)
            .expect("space should encode as plus")
    };
    let raw_written = unsafe {
        Codec::encode(&mut codec, &b'~', &mut raw_output, 0)
            .expect("raw byte should encode")
    };
    let escaped_written = unsafe {
        Codec::encode(&mut codec, &0xe4, &mut escaped_output, 0)
            .expect("escaped byte should encode")
    };

    assert_eq!(b' ', decoded_plus);
    assert_eq!(1, consumed.get());
    assert_eq!(0xe4, decoded_escape);
    assert_eq!(3, escape_consumed.get());
    assert_eq!(b'~', decoded_raw);
    assert_eq!(1, raw_consumed.get());
    assert_eq!(1, plus_written);
    assert_eq!(b"+", &plus_output);
    assert_eq!(3, raw_written);
    assert_eq!(b"%7E", &raw_output);
    assert_eq!(3, escaped_written);
    assert_eq!(b"%E4", &escaped_output);
    assert_eq!(1, <FormUrlencodedCodec as Codec>::MIN_UNITS_PER_VALUE,);
    assert_eq!(
        3,
        <FormUrlencodedCodec as Codec>::MAX_ENCODE_UNITS_PER_VALUE,
    );
    assert_eq!(
        3,
        <FormUrlencodedCodec as Codec>::MAX_DECODE_UNITS_PER_VALUE,
    );
}

#[test]
fn test_codec_trait_decodes_available_form_urlencoded_byte() {
    let mut codec = FormUrlencodedCodec::new();

    let plus = unsafe { Codec::decode(&mut codec, b"+", 0) }
        .expect("plus should decode to space");
    let (decoded, consumed) = plus;
    assert_eq!(b' ', decoded);
    assert_eq!(1, consumed.get());

    let incomplete = unsafe { Codec::decode(&mut codec, b"%", 0) }
        .expect_err("partial form escape should be incomplete");
    assert_eq!(3, super::incomplete_required(incomplete));

    let eof = unsafe { Codec::decode_eof(&mut codec, b"%", 0) }
        .expect_err("EOF form escape should be invalid");
    assert_eq!(Some(1), super::invalid_consumed(eof));
    let eof = unsafe { Codec::decode_eof(&mut codec, b"%", 0) }
        .expect_err("EOF form escape should be invalid");
    assert!(matches!(
        super::invalid_source(eof),
        MiscCodecError::InvalidEscape { index: 0, .. }
    ));
    let eof = unsafe { Codec::decode_eof(&mut codec, b"%A", 0) }
        .expect_err("EOF partial form escape should be invalid");
    assert_eq!(Some(2), super::invalid_consumed(eof));
}

#[test]
fn test_form_codec_uses_exact_widths_and_eof_rules() {
    let mut codec = FormUrlencodedCodec::new();
    assert_eq!(1, Codec::encode_len(&codec, &b'*'));
    assert_eq!(3, Codec::encode_len(&codec, &b'~'));

    let (decoded, consumed) = unsafe {
        Codec::decode_eof(&mut codec, b"+", 0)
            .expect("EOF form plus should decode")
    };
    assert_eq!(b' ', decoded);
    assert_eq!(1, consumed.get());

    let invalid = unsafe { Codec::decode_eof(&mut codec, b"%z0", 0) }
        .expect_err("EOF malformed form escape should fail");
    assert_eq!(Some(3), super::invalid_consumed(invalid));
    let invalid = unsafe { Codec::decode_eof(&mut codec, b"%z0", 0) }
        .expect_err("EOF malformed form escape should fail");
    assert!(matches!(
        super::invalid_source(invalid),
        MiscCodecError::InvalidEscape { index: 0, .. }
    ));
}

#[test]
fn test_codec_trait_decodes_and_encodes_c_string_literal_byte() {
    let mut codec = CStringLiteralCodec::new();
    let mut escaped = [0u8; 4];
    let mut simple = [0u8; 2];

    let (decoded_hex, hex_units) = unsafe {
        Codec::decode(&mut codec, br"\xD0", 0)
            .expect("hex byte escape should decode")
    };
    let (decoded_newline, newline_units) = unsafe {
        Codec::decode(&mut codec, br"\n", 0)
            .expect("simple escape should decode")
    };
    let escaped_units = unsafe {
        Codec::encode(&mut codec, &0xd0, &mut escaped, 0)
            .expect("non-printable byte should encode")
    };
    let simple_units = unsafe {
        Codec::encode(&mut codec, &b'\n', &mut simple, 0)
            .expect("newline should encode")
    };

    assert_eq!(0xd0, decoded_hex);
    assert_eq!(4, hex_units.get());
    assert_eq!(b'\n', decoded_newline);
    assert_eq!(2, newline_units.get());
    assert_eq!(4, escaped_units);
    assert_eq!(br"\xD0", &escaped);
    assert_eq!(2, simple_units);
    assert_eq!(br"\n", &simple);
    assert_eq!(1, <CStringLiteralCodec as Codec>::MIN_UNITS_PER_VALUE,);
    assert_eq!(
        4,
        <CStringLiteralCodec as Codec>::MAX_ENCODE_UNITS_PER_VALUE,
    );
    assert_eq!(
        10,
        <CStringLiteralCodec as Codec>::MAX_DECODE_UNITS_PER_VALUE,
    );
}

#[test]
fn test_codec_trait_decodes_available_c_string_literal_byte() {
    let mut codec = CStringLiteralCodec::new();

    let raw = unsafe { Codec::decode(&mut codec, br"A", 0) }
        .expect("raw C string byte should decode");
    let (decoded, consumed) = raw;
    assert_eq!(b'A', decoded);
    assert_eq!(1, consumed.get());

    let simple = unsafe { Codec::decode(&mut codec, br"\n", 0) }
        .expect("simple C escape should decode");
    let (decoded, consumed) = simple;
    assert_eq!(b'\n', decoded);
    assert_eq!(2, consumed.get());

    let eof_hex = unsafe { Codec::decode_eof(&mut codec, br"\xA", 0) }
        .expect("EOF-closed hex escape should decode");
    let (decoded, consumed) = eof_hex;
    assert_eq!(0x0a, decoded);
    assert_eq!(3, consumed.get());

    let terminated_hex = unsafe { Codec::decode(&mut codec, br"\xAG", 0) }
        .expect("terminated hex escape should decode");
    let (decoded, consumed) = terminated_hex;
    assert_eq!(0x0a, decoded);
    assert_eq!(3, consumed.get());

    let eof_octal = unsafe { Codec::decode_eof(&mut codec, br"\12", 0) }
        .expect("EOF-closed octal escape should decode");
    let (decoded, consumed) = eof_octal;
    assert_eq!(0o12, decoded);
    assert_eq!(3, consumed.get());

    let terminated_octal = unsafe { Codec::decode(&mut codec, br"\12G", 0) }
        .expect("terminated octal escape should decode");
    let (decoded, consumed) = terminated_octal;
    assert_eq!(0o12, decoded);
    assert_eq!(3, consumed.get());

    let malformed = unsafe { Codec::decode(&mut codec, br"\z", 0) }
        .expect_err("unsupported C escape should fail");
    assert_eq!(Some(2), super::invalid_consumed(malformed));
    let malformed = unsafe { Codec::decode(&mut codec, br"\z", 0) }
        .expect_err("unsupported C escape should fail");
    let malformed = super::invalid_source(malformed);
    assert!(matches!(
        malformed,
        MiscCodecError::InvalidEscape { index: 0, .. }
    ));
}

#[test]
fn test_codec_trait_decodes_c_string_literal_escape_variants() {
    let mut codec = CStringLiteralCodec::new();
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
        (br"\x1Z", 0x01, 3),
        (br"\u0022", b'"', 6),
        (br"\U00000021", b'!', 10),
        (br"\377", 0xff, 4),
    ];

    for (input, expected, expected_units) in cases {
        let (decoded, consumed) = unsafe {
            Codec::decode(&mut codec, input, 0).expect("C escape should decode")
        };
        assert_eq!(*expected, decoded, "input {input:?}");
        assert_eq!(*expected_units, consumed.get(), "input {input:?}");
    }

    for (input, expected, expected_units) in
        [(br"\xA" as &[u8], 0x0a, 3), (br"\7" as &[u8], 0x07, 2)]
    {
        let (decoded, consumed) = unsafe {
            Codec::decode_eof(&mut codec, input, 0)
                .expect("EOF C escape should decode")
        };
        assert_eq!(expected, decoded, "input {input:?}");
        assert_eq!(expected_units, consumed.get(), "input {input:?}");
    }
}

#[test]
fn test_codec_trait_reports_c_string_literal_byte_errors() {
    let mut codec = CStringLiteralCodec::new();
    let invalid_raw = unsafe { Codec::decode(&mut codec, &[0xff], 0) }
        .expect_err("invalid raw byte should fail");
    let unsupported = unsafe { Codec::decode(&mut codec, br"\z", 0) }
        .expect_err("unsupported escape should fail");
    let trailing_escape = unsafe { Codec::decode(&mut codec, br"\", 0) }
        .expect_err("trailing escape should be incomplete");
    let short_hex_marker = unsafe { Codec::decode(&mut codec, br"\x", 0) }
        .expect_err("short hex marker should be incomplete");
    let missing_hex = unsafe { Codec::decode(&mut codec, br"\xz", 0) }
        .expect_err("missing hex digit should fail");
    let incomplete_universal =
        unsafe { Codec::decode(&mut codec, br"\u12", 0) }
            .expect_err("incomplete universal escape should fail");
    let incomplete_hex = unsafe { Codec::decode(&mut codec, br"\xA", 0) }
        .expect_err("extendable hex escape should be incomplete");
    let incomplete_octal = unsafe { Codec::decode(&mut codec, br"\7", 0) }
        .expect_err("extendable octal escape should be incomplete");
    let invalid_universal_digit =
        unsafe { Codec::decode(&mut codec, br"\u00zz", 0) }
            .expect_err("invalid universal digit should fail");
    let oversized_universal =
        unsafe { Codec::decode(&mut codec, br"\u0100", 0) }
            .expect_err("oversized universal escape should fail");
    let eof_invalid = unsafe { Codec::decode_eof(&mut codec, br"\z", 0) }
        .expect_err("EOF unsupported C escape should fail");

    let invalid_raw = super::invalid_source(invalid_raw);
    let unsupported = super::invalid_source(unsupported);
    let missing_hex = super::invalid_source(missing_hex);
    let invalid_universal_digit =
        super::invalid_source(invalid_universal_digit);
    let oversized_universal = super::invalid_source(oversized_universal);
    let eof_invalid = super::invalid_source(eof_invalid);
    assert!(matches!(
        invalid_raw,
        MiscCodecError::InvalidCharacter { index: 0, .. }
    ));
    assert!(matches!(
        unsupported,
        MiscCodecError::InvalidEscape { index: 0, .. }
    ));
    assert_eq!(2, super::incomplete_required(trailing_escape));
    assert_eq!(3, super::incomplete_required(short_hex_marker));
    assert!(matches!(
        missing_hex,
        MiscCodecError::InvalidEscape { index: 0, .. }
    ));
    assert_eq!(6, super::incomplete_required(incomplete_universal));
    assert_eq!(4, super::incomplete_required(incomplete_hex));
    assert_eq!(3, super::incomplete_required(incomplete_octal));
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
    assert!(matches!(
        eof_invalid,
        MiscCodecError::InvalidEscape { index: 0, .. }
    ));
}

#[test]
fn test_codec_trait_encodes_c_string_literal_escape_variants() {
    let mut codec = CStringLiteralCodec::new();
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
            Codec::encode(&mut codec, byte, &mut output, 0)
                .expect("C string literal byte should encode")
        };
        assert_eq!(expected.len(), codec.encode_len(byte));
        assert_eq!(*expected, &output[..written], "byte {byte:#04x}");
    }
}

#[test]
fn test_value_traits_remain_convenience_layer() {
    let mut codec =
        HexCodec::upper().with_byte_prefix("0x").with_separator(" ");

    let encoded = ValueEncoder::<[u8]>::encode(&mut codec, &[0x00, 0x7f, 0xff])
        .expect("hex value encode should work");
    let decoded = ValueDecoder::<str>::decode(&mut codec, &encoded)
        .expect("hex value decode should work");

    assert_eq!("0x00 0x7F 0xFF", encoded);
    assert_eq!(vec![0x00, 0x7f, 0xff], decoded);
}
