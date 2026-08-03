// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzzes configurable hexadecimal encoding and decoding.

#![no_main]

use libfuzzer_sys::fuzz_target;
use qubit_codec_misc::HexCodec;

const MAX_INPUT_LEN: usize = 4 * 1024;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_LEN)];
    let (flags, bytes) = data
        .split_first()
        .map_or((0_u8, &[][..]), |(flags, bytes)| (*flags, bytes));
    let mut codec = HexCodec::new()
        .with_uppercase(flags & 1 != 0)
        .with_ignored_ascii_whitespace(flags & 16 != 0)
        .with_ignore_prefix_case(flags & 32 != 0);
    if flags & 2 != 0 {
        codec = codec.with_prefix("0x");
    }
    if flags & 4 != 0 {
        codec = codec.with_byte_prefix("0x");
    }
    if flags & 8 != 0 {
        codec = codec.with_separator(" ");
    }

    let encoded = codec.encode(bytes);
    assert_eq!(bytes, codec.decode(&encoded).expect("hex roundtrip"));

    let mut output = vec![0xa5];
    let original = output.clone();
    if codec.decode_into("0g", &mut output).is_err() {
        assert_eq!(original, output);
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        let _ = codec.decode(text);
    }
});
