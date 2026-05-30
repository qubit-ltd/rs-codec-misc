/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for lightweight encoder and decoder traits.

use qubit_codec_misc::{
    BufferedDecoder,
    BufferedEncoder,
    CodecBufferedDecoder,
    CodecBufferedEncoder,
    CodecDecodeError,
    CodecEncodeError,
    CodecValueEncoder,
    DecodeErrorFactory,
    EncodeErrorFactory,
    EncodePlan,
    FormUrlencodedCodec,
    HexCodec,
    PercentCodec,
    ValueDecoder,
    ValueEncoder,
};

#[test]
fn test_codec_types_can_be_used_through_traits() {
    let codec = HexCodec::new();
    let encoded = ValueEncoder::<[u8]>::encode(&codec, b"abc").expect("hex encode should succeed");
    let decoded = ValueDecoder::<str>::decode(&codec, &encoded).expect("hex decode should succeed");

    assert_eq!("616263", encoded);
    assert_eq!(b"abc".to_vec(), decoded);
}

#[test]
fn test_codec_adapter_types_can_be_used_through_reexports() {
    fn assert_codec_value_encoder<T: ValueEncoder<u8, Output = Vec<u8>, Error = qubit_codec_misc::MiscCodecError>>() {}
    fn assert_codec_buffered_decoder<T: BufferedDecoder<u8, u8>>() {}
    fn assert_codec_buffered_encoder<T: BufferedEncoder<u8, u8>>() {}
    fn assert_buffered_decode_engine<T>() {}
    fn assert_buffered_encode_engine<T>() {}

    assert_codec_value_encoder::<CodecValueEncoder<HexCodec, u8, u8>>();
    assert_codec_buffered_decoder::<CodecBufferedDecoder<HexCodec, u8>>();
    assert_codec_buffered_encoder::<CodecBufferedEncoder<HexCodec>>();
    assert_buffered_decode_engine::<qubit_codec_misc::BufferedDecodeEngine<HexCodec, (), u8>>();
    assert_buffered_encode_engine::<qubit_codec_misc::BufferedEncodeEngine<HexCodec, ()>>();

    let plan = EncodePlan::new(1, ());
    assert_eq!(1, plan.max_output_units);
    let codec = HexCodec::new();
    let encode_error =
        <CodecEncodeError<core::convert::Infallible> as EncodeErrorFactory<HexCodec>>::invalid_input_index(
            &codec, 2, 1,
        );
    assert!(matches!(encode_error, CodecEncodeError::InvalidInputIndex { .. }));
    let decode_error =
        <CodecDecodeError<core::convert::Infallible> as DecodeErrorFactory<HexCodec>>::invalid_input_index(
            &codec, 2, 1,
        );
    assert!(matches!(decode_error, CodecDecodeError::InvalidInputIndex { .. }));
}

#[test]
fn test_value_traits_accept_text_codecs() {
    fn roundtrip<C>(codec: &C, text: &str) -> String
    where
        C: ValueEncoder<str, Output = String, Error = qubit_codec_misc::MiscCodecError>
            + ValueDecoder<str, Output = String, Error = qubit_codec_misc::MiscCodecError>,
    {
        let encoded = ValueEncoder::<str>::encode(codec, text).expect("text should encode");
        ValueDecoder::<str>::decode(codec, &encoded).expect("text should decode")
    }

    assert_eq!("a b", roundtrip(&PercentCodec::new(), "a b"));
    assert_eq!("a b", roundtrip(&FormUrlencodedCodec::new(), "a b"));
}
