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
//! Representative codec throughput benchmarks.

use std::hint::black_box;

use criterion::{
    Criterion,
    Throughput,
    criterion_group,
    criterion_main,
};
use qubit_codec_misc::{
    FormUrlencodedCodec,
    HexCodec,
    PercentCodec,
};

const TEXT: &str = "field=value with spaces + punctuation / 中间文本";
const HEX_BYTES: &[u8] = b"benchmark bytes with a representative payload";

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
}

criterion_group!(benches, benchmark_percent_and_form, benchmark_hex);
criterion_main!(benches);
