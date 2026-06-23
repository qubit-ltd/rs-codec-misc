// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the encoder trait contract.

use qubit_codec_misc::{
    Base64Codec,
    HexCodec,
    PercentCodec,
    ValueEncoder,
};

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
