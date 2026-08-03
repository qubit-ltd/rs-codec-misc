// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
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
