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
    BufferedEncoder,
    CodecBufferedEncoder,
    CodecValueEncoder,
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
    fn assert_codec_buffered_encoder<T: BufferedEncoder<u8, u8>>() {}

    assert_codec_value_encoder::<CodecValueEncoder<HexCodec, u8, u8>>();
    assert_codec_buffered_encoder::<CodecBufferedEncoder<HexCodec>>();
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
