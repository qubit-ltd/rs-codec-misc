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
//! Fuzzes C string and integer literal codecs.

#![no_main]

use libfuzzer_sys::fuzz_target;
use qubit_codec::{
    Codec,
    DecodeFailure,
};
use qubit_codec_misc::{
    CIntegerLiteralCodec,
    CStringLiteralCodec,
};

const MAX_INPUT_LEN: usize = 4 * 1024;

fn decode_in_chunks(encoded: &[u8], chunk_seed: &[u8]) -> Vec<u8> {
    let mut codec = CStringLiteralCodec::new();
    let mut pending = Vec::new();
    let mut output = Vec::new();
    let mut offset = 0usize;
    let mut seed_index = 0usize;
    while offset < encoded.len() {
        let seed = chunk_seed.get(seed_index).copied().unwrap_or(1);
        seed_index += 1;
        let width = usize::from(seed % 5).max(1);
        let end = (offset + width).min(encoded.len());
        pending.extend_from_slice(&encoded[offset..end]);
        offset = end;
        loop {
            if pending.is_empty() {
                break;
            }
            match unsafe { Codec::decode(&mut codec, &pending, 0) } {
                Ok((value, consumed)) => {
                    output.push(value);
                    pending.drain(..consumed.get());
                }
                Err(DecodeFailure::Incomplete { .. }) => break,
                Err(DecodeFailure::Invalid { .. }) => return output,
            }
        }
    }
    while !pending.is_empty() {
        match unsafe { Codec::decode_eof(&mut codec, &pending, 0) } {
            Ok((value, consumed)) => {
                output.push(value);
                pending.drain(..consumed.get());
            }
            Err(_) => break,
        }
    }
    output
}

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_LEN)];
    let literals = CStringLiteralCodec::new();
    let encoded = literals.encode(data);
    assert_eq!(
        data,
        literals.decode(&encoded).expect("C literal roundtrip")
    );
    assert_eq!(data, decode_in_chunks(encoded.as_bytes(), data));

    let text = String::from_utf8_lossy(data);
    let integers = CIntegerLiteralCodec::new();
    let _ = integers.decode(&text);
});
