// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the decoder trait contract.

use qubit_codec_misc::{
    Base64Codec,
    HexCodec,
    MiscCodecError,
    ValueDecoder,
};

#[test]
fn test_decoder_trait_dispatches_to_concrete_hex_decoder() {
    let mut codec = HexCodec::new().with_prefix("0x");

    let decoded = ValueDecoder::<str>::decode(&mut codec, "0x616263")
        .expect("hex should decode");

    assert_eq!(b"abc".to_vec(), decoded);
}

#[test]
fn test_decoder_trait_preserves_concrete_error_type() {
    let error =
        ValueDecoder::<str>::decode(&mut Base64Codec::standard(), "@@@")
            .expect_err("invalid base64 should fail");

    assert!(matches!(
        error,
        MiscCodecError::InvalidInput {
            codec: "base64",
            ..
        }
    ));
}
