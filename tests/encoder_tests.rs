// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the encoder trait contract.

use qubit_codec::ValueEncoder;
use qubit_codec_misc::Base64Codec;
use qubit_codec_misc::HexCodec;
use qubit_codec_misc::MiscCodecError;
use qubit_codec_misc::PercentCodec;

#[test]
fn test_encoder_trait_dispatches_to_binary_codecs() {
    let bytes = [0xfb, 0xef];

    let hex = ValueEncoder::<[u8]>::encode(&mut HexCodec::upper(), &bytes)
        .expect("hex should encode");
    let base64 =
        ValueEncoder::<[u8]>::encode(&mut Base64Codec::url_safe(), &bytes)
            .expect("base64 should encode");

    assert_eq!("FBEF", hex);
    assert_eq!("--8=", base64);
}

#[test]
fn test_encoder_trait_dispatches_to_text_codecs() {
    let encoded =
        ValueEncoder::<str>::encode(&mut PercentCodec::new(), "a b/中")
            .expect("percent should encode");

    assert_eq!("a%20b%2F%E4%B8%AD", encoded);
}

#[test]
fn test_generic_value_encoder_example_accepts_mutable_codec() {
    fn encode_payload<C>(
        codec: &mut C,
        payload: &[u8],
    ) -> Result<String, MiscCodecError>
    where
        C: ValueEncoder<[u8], Output = String, Error = MiscCodecError>,
    {
        codec.encode(payload)
    }

    let mut codec = HexCodec::new();
    let encoded = encode_payload(&mut codec, &[0xab, 0xcd])
        .expect("generic hex encoding should succeed");

    assert_eq!("abcd", encoded);
}
