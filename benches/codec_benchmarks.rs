// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Representative codec throughput benchmarks.

use std::hint::black_box;

use criterion::BatchSize;
use criterion::Criterion;
use criterion::Throughput;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_codec_misc::Base64Codec;
use qubit_codec_misc::CIntegerLiteralCodec;
use qubit_codec_misc::CStringLiteralCodec;
use qubit_codec_misc::FormUrlencodedCodec;
use qubit_codec_misc::HexCodec;
use qubit_codec_misc::PercentCodec;

const TEXT: &str = "field=value with spaces + punctuation / 中间文本";
const HEX_BYTES: &[u8] = b"benchmark bytes with a representative payload";
const INTEGER_LITERAL: &str = "0xBEEFC0DE";

fn benchmark_percent_and_form(c: &mut Criterion) {
    let percent = PercentCodec::new();
    let form = FormUrlencodedCodec::new();
    let mut group = c.benchmark_group("text-codecs");
    group.throughput(Throughput::Bytes(TEXT.len() as u64));
    group.bench_function("percent_encode", |bencher| {
        bencher.iter(|| black_box(percent.encode(black_box(TEXT))));
    });
    group.bench_function("form_encode", |bencher| {
        bencher.iter(|| black_box(form.encode(black_box(TEXT))));
    });
    let percent_encoded = percent.encode(TEXT);
    let form_encoded = form.encode(TEXT);
    group.bench_function("percent_decode", |bencher| {
        bencher.iter(|| black_box(percent.decode(black_box(&percent_encoded))));
    });
    group.bench_function("form_decode", |bencher| {
        bencher.iter(|| black_box(form.decode(black_box(&form_encoded))));
    });
    group.finish();
}

fn benchmark_hex(c: &mut Criterion) {
    let codec = HexCodec::upper()
        .with_byte_prefix("0x")
        .with_separator(" ")
        .with_ignored_ascii_whitespace(true);
    let encoded = codec.encode(HEX_BYTES);
    let mut group = c.benchmark_group("hex-codec");
    group.throughput(Throughput::Bytes(HEX_BYTES.len() as u64));
    group.bench_function("encode_prefixed_separated", |bencher| {
        bencher.iter(|| black_box(codec.encode(black_box(HEX_BYTES))));
    });
    group.bench_function("decode_prefixed_separated", |bencher| {
        bencher.iter(|| black_box(codec.decode(black_box(&encoded))));
    });
    group.finish();

    let mut group = c.benchmark_group("hex-codec-decode_into_reuse");
    for size in [64_usize, 64 * 1024] {
        let bytes = vec![0x5a; size];
        let plain_codec = HexCodec::new();
        let plain_encoded = plain_codec.encode(&bytes);
        let mut plain_output = Vec::with_capacity(size);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("continuous_{size}B"), |bencher| {
            bencher.iter(|| {
                plain_output.clear();
                plain_codec
                    .decode_into(black_box(&plain_encoded), &mut plain_output)
                    .expect("valid continuous hex input");
                black_box(&plain_output);
            });
        });

        let separated_codec = HexCodec::new().with_byte_prefix("0x").with_separator(" ");
        let separated_encoded = separated_codec.encode(&bytes);
        let mut separated_output = Vec::with_capacity(size);
        group.bench_function(format!("prefixed_separated_{size}B"), |bencher| {
            bencher.iter(|| {
                separated_output.clear();
                separated_codec
                    .decode_into(black_box(&separated_encoded), &mut separated_output)
                    .expect("valid prefixed separated hex input");
                black_box(&separated_output);
            });
        });

        for (format, codec) in [
            ("continuous", HexCodec::new()),
            (
                "prefixed_separated",
                HexCodec::new().with_byte_prefix("0x").with_separator(" "),
            ),
        ] {
            let encoded = codec.encode(&bytes);
            for (allocation, initial_capacity) in [("empty", 0), ("presized", size)] {
                let mut observed = Vec::with_capacity(initial_capacity);
                let before = observed.capacity();
                codec.decode_into(&encoded, &mut observed).expect("valid hex input");
                let after = observed.capacity();
                eprintln!(
                    "hex cold capacity: input={} output={} mode={format}/{allocation} before={before} after={after} grew={}",
                    encoded.len(),
                    observed.len(),
                    after > before
                );

                group.bench_function(format!("{format}_{size}B_cold_{allocation}"), |bencher| {
                    bencher.iter_batched(
                        || Vec::with_capacity(initial_capacity),
                        |mut output| {
                            codec
                                .decode_into(black_box(&encoded), &mut output)
                                .expect("valid hex input");
                            black_box(output);
                        },
                        BatchSize::SmallInput,
                    );
                });
            }
        }
    }
    group.finish();
}

fn benchmark_base64_and_c_literals(c: &mut Criterion) {
    let base64 = Base64Codec::standard();
    let c_literals = CStringLiteralCodec::new();
    let integers = CIntegerLiteralCodec::new();
    let base64_encoded = base64.encode(HEX_BYTES);
    let c_literal_encoded = c_literals.encode(HEX_BYTES);
    let mut group = c.benchmark_group("misc-codecs");
    group.throughput(Throughput::Bytes(HEX_BYTES.len() as u64));
    group.bench_function("base64_encode", |bencher| {
        bencher.iter(|| black_box(base64.encode(black_box(HEX_BYTES))));
    });
    group.bench_function("base64_decode", |bencher| {
        bencher.iter(|| black_box(base64.decode(black_box(&base64_encoded))));
    });
    group.bench_function("c_string_encode", |bencher| {
        bencher.iter(|| black_box(c_literals.encode(black_box(HEX_BYTES))));
    });
    group.bench_function("c_string_decode", |bencher| {
        bencher.iter(|| black_box(c_literals.decode(black_box(&c_literal_encoded))));
    });
    group.throughput(Throughput::Bytes(INTEGER_LITERAL.len() as u64));
    group.bench_function("c_integer_decode", |bencher| {
        bencher.iter(|| black_box(integers.decode(black_box(INTEGER_LITERAL))));
    });
    group.finish();
}

criterion_group!(
    benches,
    benchmark_percent_and_form,
    benchmark_hex,
    benchmark_base64_and_c_literals
);
criterion_main!(benches);
