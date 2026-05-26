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
    Decoder,
    Encoder,
    FormUrlencodedCodec,
    HexCodec,
    PercentCodec,
};

#[test]
fn test_codec_types_can_be_used_through_traits() {
    let codec = HexCodec::new();
    let encoded = Encoder::<[u8]>::encode(&codec, b"abc").expect("hex encode should succeed");
    let decoded = Decoder::<str>::decode(&codec, &encoded).expect("hex decode should succeed");

    assert_eq!("616263", encoded);
    assert_eq!(b"abc".to_vec(), decoded);
}

#[test]
fn test_whole_value_traits_accept_text_codecs() {
    fn roundtrip<C>(codec: &C, text: &str) -> String
    where
        C: Encoder<str, Output = String, Error = qubit_codec_misc::MiscCodecError>
            + Decoder<str, Output = String, Error = qubit_codec_misc::MiscCodecError>,
    {
        let encoded = Encoder::<str>::encode(codec, text).expect("text should encode");
        Decoder::<str>::decode(codec, &encoded).expect("text should decode")
    }

    assert_eq!("a b", roundtrip(&PercentCodec::new(), "a b"));
    assert_eq!("a b", roundtrip(&FormUrlencodedCodec::new(), "a b"));
}
