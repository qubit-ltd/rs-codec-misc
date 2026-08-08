// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Differentially tests complete Base64 quantum encoding and decoding.

#![no_main]

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use base64::engine::general_purpose::URL_SAFE;
use libfuzzer_sys::fuzz_target;
use qubit_codec::Codec;
use qubit_codec_misc::Base64QuantumCodec;

const MAX_INPUT_LEN: usize = 4 * 1024;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_LEN)];
    let (flags, payload) = data
        .split_first()
        .map_or((0_u8, &[][..]), |(flags, payload)| (*flags, payload));
    let (engine, mut codec) = if flags & 1 == 0 {
        (&STANDARD, Base64QuantumCodec::standard())
    } else {
        (&URL_SAFE, Base64QuantumCodec::url_safe())
    };

    for chunk in payload.chunks_exact(3) {
        let expected = engine.encode(chunk);
        let mut encoded = [0u8; 4];
        let written = unsafe {
            Codec::encode(
                &mut codec,
                chunk.try_into().expect("chunk width"),
                &mut encoded,
                0,
            )
            .expect("quantum encoding should succeed")
        };
        assert_eq!(4, written);
        assert_eq!(expected.as_bytes(), &encoded);

        let (decoded, consumed) = unsafe {
            Codec::decode(&mut codec, &encoded, 0)
                .expect("encoded quantum should decode")
        };
        assert_eq!(chunk, decoded);
        assert_eq!(4, consumed.get());
    }

    for window in payload.windows(4) {
        let failure = unsafe { Codec::decode(&mut codec, window, 0) };
        if let Err(failure) = failure {
            if failure.invalid_source().is_some() {
                assert_eq!(
                    Some(4),
                    failure.consumed_units().map(|width| width.get())
                );
            }
        }
    }
});
