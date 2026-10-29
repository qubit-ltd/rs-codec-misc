// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::cell::Cell;

use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec_misc::Base64Codec;
use qubit_codec_misc::Base64Decoder;
use qubit_codec_misc::Base64Encoder;

thread_local! {
    // Const Cell initialization uses no heap allocation.
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    // Const Cell initialization uses no heap allocation.
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

struct ThreadCountingAllocator;

fn note_allocation() {
    if COUNTING.try_with(Cell::get).unwrap_or(false) {
        let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
    }
}

// SAFETY: Every allocation operation is forwarded to System with the same
// pointer and layout arguments, preserving System's allocator contract.
unsafe impl GlobalAlloc for ThreadCountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        note_allocation();
        // SAFETY: The caller supplied this layout, which is forwarded unchanged.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        note_allocation();
        // SAFETY: The caller supplied this layout, which is forwarded unchanged.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        note_allocation();
        // SAFETY: The caller supplied this pointer, layout, and size, all forwarded
        // unchanged.
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: The caller supplied this pointer and layout, both forwarded
        // unchanged.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: ThreadCountingAllocator = ThreadCountingAllocator;

struct CountingGuard;

impl Drop for CountingGuard {
    fn drop(&mut self) {
        let _ = COUNTING.try_with(|enabled| enabled.set(false));
    }
}

fn measured<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    COUNTING.with(|enabled| assert!(!enabled.get(), "no nested measurements"));
    ALLOCATIONS.with(|count| count.set(0));
    COUNTING.with(|enabled| enabled.set(true));
    let guard = CountingGuard;
    let result = operation();
    drop(guard);
    let allocations = ALLOCATIONS.with(Cell::get);
    (result, allocations)
}

#[test]
fn test_base64_streaming_and_errors_allocate_nothing() {
    let codecs = [
        Base64Codec::standard(),
        Base64Codec::standard_no_pad(),
        Base64Codec::url_safe(),
        Base64Codec::url_safe_no_pad(),
    ];
    let mut encoders = codecs.each_ref().map(Base64Codec::encoder);
    let mut decoders = codecs.each_ref().map(Base64Codec::decoder);
    let inputs: [&[u8]; 4] = [b"M", b"M", b"\xfb\xff", b"\xfb\xff"];
    let encoded: [&[u8]; 4] = [b"TQ==", b"TQ", b"-_8=", b"-_8"];
    let mut text = [[0u8; 8]; 4];
    let mut bytes = [[0u8; 8]; 4];

    let (results, allocations) = measured(|| {
        let mut results = [(0usize, 0usize); 4];
        for i in 0..4 {
            let reset_encoder = encoders[i].reset(&mut [], 0);
            let encoded_len = encoders[i].transcode_complete_into(inputs[i], &mut text[i]);
            let reset_decoder = decoders[i].reset(&mut [], 0);
            let decoded_len = decoders[i].transcode_complete_into(encoded[i], &mut bytes[i]);
            results[i] = (
                usize::from(reset_encoder.is_ok()) + usize::from(reset_decoder.is_ok()),
                encoded_len.unwrap_or(usize::MAX) + decoded_len.unwrap_or(usize::MAX),
            );
        }
        results
    });
    assert_eq!(allocations, 0);
    assert_eq!(results, [(2, 5), (2, 3), (2, 6), (2, 5)]);
    assert_eq!(&text[0][..4], b"TQ==");
    assert_eq!(&text[1][..2], b"TQ");
    assert_eq!(&text[2][..4], b"-_8=");
    assert_eq!(&text[3][..3], b"-_8");
    assert_eq!(&bytes[0][..1], b"M");
    assert_eq!(&bytes[2][..2], &[0xfb, 0xff]);

    let mut encoder = Base64Encoder::standard();
    let mut decoder = Base64Decoder::standard();
    let mut output = [0xa5; 16];
    let mut decoded = [0xa5; 8];
    let (results, allocations) = measured(|| {
        let reset_encode = encoder.reset(&mut [], 0);
        let need_output = encoder.transcode(b"Man", 0, &mut [], 0);
        let finish = encoder.finish(&mut output, 0);
        let reset_decode = decoder.reset(&mut [], 0);
        let need_input = decoder.transcode(b"TQ", 0, &mut decoded, 0);
        let eof = decoder.transcode_eof(b"TQ==", 0, &mut decoded, 0);
        let finish_decode = decoder.finish(&mut [], 0);
        (
            reset_encode,
            need_output,
            finish,
            reset_decode,
            need_input,
            eof,
            finish_decode,
        )
    });
    assert_eq!(allocations, 0);
    assert!(
        results.0.is_ok()
            && results.1.as_ref().is_ok_and(|p| p.status()
                == TranscodeStatus::NeedOutput {
                    required: std::num::NonZeroUsize::new(4).unwrap()
                })
    );
    assert!(
        results.2.is_ok()
            && results.3.is_ok()
            && results
                .4
                .as_ref()
                .is_ok_and(|p| matches!(p.status(), TranscodeStatus::NeedInput { .. }))
    );
    assert!(results.5.is_ok() && results.6.is_ok());

    let mut retry_encoder = Base64Encoder::standard();
    let mut short_output = [0xa5; 3];
    let mut full_output = [0xa5; 4];
    retry_encoder.reset(&mut [], 0).unwrap();
    let retained = retry_encoder.transcode(b"M", 0, &mut [], 0);
    let (retry, allocations) = measured(|| {
        let short_finish = retry_encoder.finish(&mut short_output, 0);
        let full_finish = retry_encoder.finish(&mut full_output, 0);
        (short_finish, full_finish)
    });
    assert_eq!(allocations, 0);
    assert!(retained.is_ok() && retry.0.is_err() && retry.1.is_ok());
    assert_eq!(full_output, *b"TQ==");

    let mut index_decoder = Base64Decoder::standard();
    index_decoder.reset(&mut [], 0).unwrap();
    let (invalid_index, allocations) = measured(|| index_decoder.transcode(b"TQ==", 5, &mut decoded, 0));
    assert_eq!(allocations, 0);
    assert!(invalid_index.is_err());

    let mut failure_decoder = Base64Decoder::standard();
    failure_decoder.reset(&mut [], 0).unwrap();
    let (error, allocations) = measured(|| failure_decoder.transcode_eof(b"YR==", 0, &mut decoded, 0));
    assert_eq!(allocations, 0);
    assert!(error.unwrap_err().is_domain());
    let (poisoned, allocations) = measured(|| failure_decoder.finish(&mut [], 0));
    assert_eq!(allocations, 0);
    assert!(poisoned.is_err());
    let (recovered, allocations) = measured(|| {
        let reset = failure_decoder.reset(&mut [], 0);
        let decode = failure_decoder.transcode_eof(b"TQ==", 0, &mut decoded, 0);
        let finish = failure_decoder.finish(&mut [], 0);
        (reset, decode, finish)
    });
    assert_eq!(allocations, 0);
    assert!(recovered.0.is_ok() && recovered.1.is_ok() && recovered.2.is_ok());

    let mut terminal = Base64Decoder::standard();
    terminal.reset(&mut [], 0).unwrap();
    let (outcomes, allocations) = measured(|| {
        let first = terminal.transcode(b"TQ==", 0, &mut decoded, 0);
        let extra = terminal.transcode(b"TWFu", 0, &mut decoded, 0);
        let invalid_index = terminal.transcode(b"", 1, &mut decoded, 0);
        (first, extra, invalid_index)
    });
    assert_eq!(allocations, 0);
    assert!(outcomes.0.is_ok() && outcomes.1.unwrap_err().is_domain() && outcomes.2.is_err());
}
