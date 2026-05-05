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

use qubit_codec::{
    Codec,
    CodecError,
    Decoder,
    Encoder,
    HexCodec,
    PercentCodec,
};

fn roundtrip_bytes<C>(codec: &C, bytes: &[u8]) -> Result<Vec<u8>, CodecError>
where
    C: Codec<[u8], str>
        + Encoder<[u8], Output = String, Error = CodecError>
        + Decoder<str, Output = Vec<u8>, Error = CodecError>,
{
    let encoded = Encoder::<[u8]>::encode(codec, bytes)?;
    Decoder::<str>::decode(codec, &encoded)
}

fn roundtrip_text<C>(codec: &C, text: &str) -> Result<String, CodecError>
where
    C: Codec<str, str>
        + Encoder<str, Output = String, Error = CodecError>
        + Decoder<str, Output = String, Error = CodecError>,
{
    let encoded = Encoder::<str>::encode(codec, text)?;
    Decoder::<str>::decode(codec, &encoded)
}

#[test]
fn test_codec_trait_supports_hex_byte_roundtrip() {
    let codec = HexCodec::upper().with_byte_prefix("0x").with_separator(" ");

    let decoded = roundtrip_bytes(&codec, &[0x00, 0x7f, 0xff]).expect("hex roundtrip should work");

    assert_eq!(vec![0x00, 0x7f, 0xff], decoded);
}

#[test]
fn test_codec_trait_supports_percent_text_roundtrip() {
    let codec = PercentCodec::new();

    let decoded = roundtrip_text(&codec, "a b/中").expect("percent roundtrip should work");

    assert_eq!("a b/中", decoded);
}
