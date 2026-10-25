// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzzes C string and integer literal codecs.

#![no_main]

use libfuzzer_sys::fuzz_target;
use qubit_codec::Codec;
use qubit_codec::DecodeFailure;
use qubit_codec_misc::CIntegerLiteralCodec;
use qubit_codec_misc::CStringLiteralCodec;

const MAX_INPUT_LEN: usize = 4 * 1024;

/// Decodes a byte stream incrementally and checks the codec's consumption
/// bounds.
fn decode_in_chunks(encoded: &[u8], chunk_seed: &[u8]) -> Result<Vec<u8>, ()> {
    let mut codec = CStringLiteralCodec::new();
    let mut pending = Vec::new();
    let mut output = Vec::new();
    let mut offset = 0;
    let mut seed_index = 0;
    while offset < encoded.len() {
        let seed = chunk_seed.get(seed_index).copied().unwrap_or(1);
        seed_index += 1;
        let end = (offset + usize::from(seed % 5).max(1)).min(encoded.len());
        pending.extend_from_slice(&encoded[offset..end]);
        offset = end;
        while !pending.is_empty() {
            // SAFETY: pending is nonempty and the complete available slice is passed.
            // The codec's streaming preconditions are therefore satisfied.
            match unsafe { Codec::decode(&mut codec, &pending, 0) } {
                Ok((value, consumed)) => {
                    assert!(consumed.get() <= pending.len());
                    output.push(value);
                    pending.drain(..consumed.get());
                }
                Err(DecodeFailure::Incomplete { required_total, .. }) => {
                    assert!(required_total.get() > pending.len());
                    break;
                }
                Err(DecodeFailure::Invalid { consumed, .. }) => {
                    if let Some(consumed) = consumed {
                        assert!(consumed.get() <= pending.len());
                    }
                    return Err(());
                }
            }
        }
    }
    while !pending.is_empty() {
        // SAFETY: pending is nonempty and the complete available slice is passed.
        // EOF is explicit here, so the codec's EOF decoding preconditions hold.
        match unsafe { Codec::decode_eof(&mut codec, &pending, 0) } {
            Ok((value, consumed)) => {
                assert!(consumed.get() <= pending.len());
                output.push(value);
                pending.drain(..consumed.get());
            }
            Err(DecodeFailure::Invalid { consumed, .. }) => {
                if let Some(consumed) = consumed {
                    assert!(consumed.get() <= pending.len());
                }
                return Err(());
            }
            Err(DecodeFailure::Incomplete { .. }) => {
                panic!("C literal decode remained incomplete at EOF")
            }
        }
    }
    Ok(output)
}

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_LEN)];
    let literals = CStringLiteralCodec::new();
    let encoded = literals.encode(data);
    assert_eq!(data, literals.decode(&encoded).expect("C literal roundtrip"));
    assert_eq!(
        data,
        decode_in_chunks(encoded.as_bytes(), data).expect("encoded chunks")
    );

    let text = String::from_utf8_lossy(data);
    let owned = literals.decode(&text);
    let chunked = decode_in_chunks(text.as_bytes(), data);
    assert_eq!(owned.is_ok(), chunked.is_ok());
    if let (Ok(owned), Ok(chunked)) = (owned, chunked) {
        assert_eq!(owned, chunked);
    }
    let _ = decode_in_chunks(data, data);

    let integers = CIntegerLiteralCodec::new();
    let _ = integers.decode(&text);

    let mut value_bytes = [0u8; 8];
    let copied = data.len().min(value_bytes.len());
    value_bytes[..copied].copy_from_slice(&data[..copied]);
    let value = u64::from_le_bytes(value_bytes);
    for literal in [format!("{value}"), format!("0{value:o}"), format!("0x{value:x}")] {
        assert_eq!(value, integers.decode(&literal).expect("formatted u64 should decode"));
    }
    let overflow = u128::from(u64::MAX) + 1 + u128::from(value);
    for literal in [
        format!("{overflow}"),
        format!("0{overflow:o}"),
        format!("0x{overflow:x}"),
    ] {
        assert!(integers.decode(&literal).is_err(), "{literal}");
    }
});
