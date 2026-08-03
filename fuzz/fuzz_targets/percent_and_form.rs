// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzzes strict percent and form-urlencoded text codecs.

#![no_main]

use form_urlencoded::byte_serialize;
use libfuzzer_sys::fuzz_target;
use qubit_codec_misc::{
    FormUrlencodedCodec,
    PercentCodec,
};

const MAX_INPUT_LEN: usize = 4 * 1024;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_LEN)];
    let text = String::from_utf8_lossy(data);
    let percent = PercentCodec::new();
    let form = FormUrlencodedCodec::new();

    let percent_encoded = percent.encode(&text);
    assert_eq!(
        text,
        percent.decode(&percent_encoded).expect("percent roundtrip")
    );

    let form_encoded = form.encode(&text);
    let expected: String = byte_serialize(text.as_bytes()).collect();
    assert_eq!(expected, form_encoded);
    assert_eq!(text, form.decode(&form_encoded).expect("form roundtrip"));

    let _ = percent.decode(&text);
    let _ = form.decode(&text);
});
