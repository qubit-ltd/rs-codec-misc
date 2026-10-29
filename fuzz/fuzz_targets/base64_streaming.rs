// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Differentially fuzzes bounded Base64 streaming transcoding.

#![no_main]

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::engine::general_purpose::URL_SAFE;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use libfuzzer_sys::fuzz_target;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec_misc::Base64Codec;

const MAX_INPUT_LEN: usize = 4 * 1024;

fn control(seed: &[u8], step: usize) -> u8 {
    if seed.is_empty() {
        0
    } else {
        seed[step % seed.len()]
    }
}

/// Runs a transcoder through bounded, deterministic input and output windows.
fn drive<T>(
    transcoder: &mut T,
    input: &[u8],
    seed: &[u8],
    decoder: bool,
) -> Result<Vec<u8>, T::Error>
where
    T: Transcoder<Input = u8, Output = u8>,
{
    assert_eq!(transcoder.reset(&mut [], 0)?, 0);
    let prefix = usize::from(control(seed, 0) % 4);
    let mut source = vec![0xa5; prefix];
    source.extend_from_slice(input);
    let bound = transcoder
        .max_total_output_len(input.len())
        .expect("bounded input")
        .max(4);
    let mut storage = vec![0xa5; bound + 5];
    let mut actual = Vec::new();
    let mut cursor = prefix;
    let mut visible = prefix;
    let mut forced_capacity = None;
    for step in 0..input.len() * 4 + 64 {
        if forced_capacity.is_none() && visible < source.len() {
            visible = (visible + 1 + usize::from(control(seed, step + 1) % 16)).min(source.len());
        }
        let eof = visible == source.len();
        let out_start = usize::from(control(seed, step + 2) % 4);
        let choices = [0, 1, 2, 3, 4, bound];
        let capacity = forced_capacity
            .take()
            .unwrap_or(choices[usize::from(control(seed, step + 3)) % choices.len()]);
        assert!(capacity <= bound);
        storage.fill(0xa5);
        let result = if eof {
            transcoder.transcode_eof(
                &source[..visible],
                cursor,
                &mut storage[..out_start + capacity],
                out_start,
            )
        } else {
            transcoder.transcode(
                &source[..visible],
                cursor,
                &mut storage[..out_start + capacity],
                out_start,
            )
        };
        assert!(storage[..out_start].iter().all(|byte| *byte == 0xa5));
        assert!(
            storage[out_start + capacity..]
                .iter()
                .all(|byte| *byte == 0xa5)
        );
        let progress = result?;
        progress
            .validate(cursor, visible - cursor, out_start, capacity)
            .expect("valid progress");
        cursor += progress.read();
        actual.extend_from_slice(&storage[out_start..out_start + progress.written()]);
        match progress.status() {
            TranscodeStatus::Complete if eof => {
                let mut tail = [0xa5; 6];
                let written = transcoder.finish(&mut tail[..5], 1)?;
                assert!(written <= 4);
                assert_eq!(tail[0], 0xa5);
                assert_eq!(tail[5], 0xa5);
                actual.extend_from_slice(&tail[1..1 + written]);
                return Ok(actual);
            }
            TranscodeStatus::Complete => {}
            TranscodeStatus::NeedInput { .. } => {
                assert!(decoder && !eof, "only decoder open tails need more input");
            }
            TranscodeStatus::NeedOutput { required } => {
                forced_capacity = Some(required.get());
            }
        }
    }
    panic!("bounded streaming driver did not converge");
}

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_LEN)];
    let (mode, rest) = data
        .split_first()
        .map_or((0, &[][..]), |(flag, rest)| (*flag, rest));
    let seed_len = rest.len().min(8);
    let (seed, payload) = rest.split_at(seed_len);
    let (codec, engine) = match mode % 4 {
        0 => (Base64Codec::standard(), &STANDARD),
        1 => (Base64Codec::standard_no_pad(), &STANDARD_NO_PAD),
        2 => (Base64Codec::url_safe(), &URL_SAFE),
        _ => (Base64Codec::url_safe_no_pad(), &URL_SAFE_NO_PAD),
    };

    let expected_text = engine.encode(payload);
    let mut encoder = codec.encoder();
    assert_eq!(
        drive(&mut encoder, payload, seed, false).expect("encode payload"),
        expected_text.as_bytes(),
    );
    assert!(encoder.transcode(&[], 0, &mut [], 0).is_err());
    assert!(encoder.finish(&mut [], 0).is_err());
    assert_eq!(
        drive(&mut encoder, payload, seed, false).expect("reset encoder"),
        expected_text.as_bytes(),
    );

    let mut decoder = codec.decoder();
    assert_eq!(
        drive(&mut decoder, expected_text.as_bytes(), seed, true).expect("decode generated text"),
        payload,
    );
    assert!(decoder.transcode(&[], 0, &mut [], 0).is_err());
    assert!(decoder.finish(&mut [], 0).is_err());

    let expected = engine.decode(payload);
    let actual = drive(&mut decoder, payload, seed, true);
    match (expected, actual) {
        (Ok(expected), Ok(actual)) => assert_eq!(actual, expected),
        (Err(_), Err(error)) => {
            assert!(error.is_domain());
            assert!(matches!(
                decoder
                    .transcode(&[], 0, &mut [], 0)
                    .unwrap_err()
                    .failure_ref(),
                Some(TranscodeFailure::LifecyclePoisoned)
            ));
            assert!(matches!(
                decoder.finish(&mut [], 0).unwrap_err().failure_ref(),
                Some(TranscodeFailure::LifecyclePoisoned)
            ));
        }
        (expected, actual) => panic!("acceptance mismatch: {expected:?}, {actual:?}"),
    }

    assert_eq!(
        drive(&mut decoder, b"TWFu", seed, true).expect("reset decoder"),
        b"Man",
    );
});
