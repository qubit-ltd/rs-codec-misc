# Qubit Misc Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec-misc/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec-misc/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec-misc/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec-misc/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec-misc.svg?color=blue)](https://crates.io/crates/qubit-codec-misc)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Reusable byte and text codecs for Rust applications.

## Overview

Qubit Misc Codec provides small, explicit codecs for stable byte and text
encodings commonly needed across Qubit Rust crates and applications. Its API
stays lightweight, typed, and idiomatic, with direct concrete methods for common
use cases and traits for generic boundaries.

This crate focuses on textual encodings with clear wire-format semantics:

- hexadecimal byte strings
- Base64 byte strings
- C integer literal fragments
- C string literal byte fragments
- percent-encoded UTF-8 text
- `application/x-www-form-urlencoded` UTF-8 text fragments

It intentionally does not replace Rust's `Display`, `FromStr`, `TryFrom`, or
`serde` APIs for ordinary object conversion.

## Design Goals

- **Explicit Semantics**: each codec documents its alphabet, separator, padding,
  and decoding rules.
- **Small API Surface**: expose direct `encode` and `decode` methods first, with
  traits available for generic call sites.
- **No Hidden Panics**: malformed input is reported as `MiscCodecError` instead of
  panicking.
- **Layered Traits**: `Codec` covers low-level single-value or quantum
  conversion, while `ValueEncoder` and `ValueDecoder` remain owned whole-value
  convenience traits. Generic adapters, buffered engines, and policy hooks live
  in `qubit-codec`; import them from the core crate when building custom
  adapters around these codecs.
- **Reusable Implementations**: common encodings live in one crate instead of
  being reimplemented by downstream crates.
- **Minimal Dependencies**: rely on well-maintained crates only where they add
  real value.

## Features

### 🔡 **Hexadecimal Bytes**

- **Lowercase by Default**: `HexCodec::new()` produces contiguous lowercase hex.
- **Uppercase Mode**: `HexCodec::upper()` or `with_uppercase(true)` produces
  uppercase digits.
- **Optional Whole Prefix**: add and require a prefix such as `0x` before the
  entire encoded value.
- **Optional Per-Byte Prefix**: add and require a byte prefix such as `0x`
  before each encoded byte.
- **Optional Separator**: write and accept separators between bytes, such as
  `:` or a space.
- **Whitespace Handling**: optionally ignore ASCII whitespace while decoding.
- **Prefix Case Handling**: optionally ignore ASCII case when matching
  configured prefixes while decoding.
- **Named Configuration**: reuse a `HexCodecConfig` through
  `HexCodec::from_config` when several codecs share the same policy.
- **Buffer APIs**: `encode_into` and `decode_into` append into existing buffers.

Whitespace policy depends on the separator configuration. With a non-empty
separator, each byte requires two adjacent hex digits: `1 f:8b` is rejected
with separator `:`. Without a non-empty separator, ignored whitespace may
split a byte, so `0x1 f` is accepted with byte prefix `0x`. Complete structural
tokens take precedence over extra ignored whitespace: with ignored ASCII
whitespace enabled and separator ` : `, `1f\t : 8b` decodes to
`[0x1f, 0x8b]`. In this example, `\t` denotes an actual tab character.
Required separators remain required. Non-whitespace separators are matched
exactly; a whitespace-only separator still requires whitespace between bytes,
and ignored ASCII whitespace may consume additional adjacent whitespace.

### 🔐 **Base64 Bytes**

- **Standard Alphabet**: padded and no-padding standard Base64.
- **URL-Safe Alphabet**: padded and no-padding URL-safe Base64.
- **Strict Padding**: each facade accepts only its configured padding policy;
  complete groups such as `TWFu` are valid with either policy. Whitespace and
  non-canonical trailing bits are rejected.
- **Quantum Core**: `Base64QuantumCodec` handles complete three-byte to
  four-unit Base64 quanta; final padding stays in the facade/transcoder layer.
- **Typed Errors**: malformed input is reported as
  `MiscCodecError::InvalidBase64`, retaining the `base64::DecodeError` source,
  error category, and input index when available.

When using streaming decode hooks that skip or replace malformed percent,
form-URL, or C-string input, the codec reports the invalid span visible in the
current chunk. Recovery output for malformed input may therefore depend on
chunk boundaries. This caveat does not apply to valid input or strict decoding.

### 🔤 **C String Literal Bytes**

- **Mixed Text and Escapes**: decodes fragments such as `PK\003\004` and
  `\xd0\xcf`.
- **C Escape Support**: handles simple, octal, hexadecimal, and universal byte
  escapes.
- **Byte-Oriented Output**: decodes directly to raw bytes without requiring
  UTF-8.

### 🔢 **C Integer Literals**

- **Radix Detection**: decodes decimal, octal, and `0x`/`0X` hexadecimal
  integer literals.
- **Unsigned Output**: returns `u64` for non-negative integer literal fragments.
- **Precise Errors**: reports invalid digits with their original input index.
- **Restricted Token Contract**: accepts complete non-negative tokens only;
  signs, suffixes, digit separators, and token-stream parsing are not included.
- **Value-Token Decode**: remains a `ValueDecoder<str>` convenience codec because
  integer literal encoding strategy and token boundaries are not part of the
  single-value core abstraction yet.

### 🌐 **Percent-Encoding**

- **UTF-8 Text**: encodes and decodes UTF-8 strings.
- **RFC 3986 Unreserved Set**: leaves ASCII letters, digits, `-`, `.`, `_`, and
  `~` unchanged.
- **Uppercase Escapes**: writes percent escapes such as `%2F` and `%E4`.
- **Malformed Escape Detection**: reports truncated or invalid `%XX` sequences.

### 📝 **Form URL Encoding**

- **Form Fragment Codec**: handles `application/x-www-form-urlencoded` text
  fragments.
- **Space as Plus**: encodes spaces as `+` and decodes `+` back to spaces.
- **WHATWG Character Set**: leaves only ASCII letters, digits, `*`, `-`, `.`,
  and `_` unescaped.
- **Percent Compatibility**: shares the same UTF-8 and `%XX` validation behavior
  as `PercentCodec`.

### 🎯 **Focused Public API**

- **`ValueEncoder<Input>`**: encodes borrowed input into an associated output type.
- **`ValueDecoder<Input>`**: decodes borrowed input into an associated output type.
- **[`Codec`](https://docs.rs/qubit-codec/0.17/qubit_codec/trait.Codec.html) with associated `Value` and `Unit`**: a low-level trait with unsafe encoding and decoding entry points
  for one value or codec quantum over caller-provided unit buffers. Callers
  must uphold the documented index and capacity preconditions; implementations
  use `impl Codec`, not `unsafe impl Codec`.
- **`CodecValueEncoder<C>` / `CodecTranscodeEncoder<C>` /
  `CodecTranscodeDecoder<C>`**: default value and buffered adapters
  available from `qubit-codec`.
- **`TranscodeEncodeEngine` / `TranscodeEncodeHooks` /
  `TranscodeDecodeEngine` / `TranscodeDecodeHooks`**: reusable buffered engines
  and policy hooks available from `qubit-codec` for custom adapters.
- **`MiscCodecError` / `MiscCodecResult`**: common error and result types for bundled
  codecs.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qubit-codec-misc = "0.8"
```

## Example: Parsing a Magic Rule

An application that has already extracted attributes from XML can use these codecs to parse a magic rule. This is an application composition example, not a complete MIME loader: the library parses syntax and numeric values, while the application checks the real field width and mask length, associates errors with the source rule, and decides whether to load it. A parse failure rejects the rule; it must not silently keep partially parsed values.

Here, `number` is a separate illustrative numeric field; it does not imply that the string magic and the numeric value belong to the same MIME matcher rule. `u16` is for demonstration only; choose the width according to the actual target field.

```rust
use qubit_codec_misc::CIntegerLiteralCodec;
use qubit_codec_misc::CStringLiteralCodec;
use qubit_codec_misc::HexCodec;

fn parse_rule(value: &str, number: &str, mask: &str)
    -> Result<(Vec<u8>, u16, Vec<u8>), String>
{
    let bytes = CStringLiteralCodec::new().decode(value).map_err(|e| e.to_string())?;
    let number = CIntegerLiteralCodec::new().decode(number).map_err(|e| e.to_string())?;
    let number = u16::try_from(number).map_err(|_| "value exceeds 16 bits".to_owned())?;
    let mask = HexCodec::new()
        .with_prefix("0x")
        .with_ignore_prefix_case(true)
        .decode(mask).map_err(|e| e.to_string())?;
    if mask.len() != bytes.len() {
        return Err("mask length must equal magic length".to_owned());
    }
    Ok((bytes, number, mask))
}

fn main() -> Result<(), String> {
    let (bytes, number, mask) = parse_rule(r"PK\003\004", "0x1234", "0XFFFFFFFF")?;
    assert_eq!(b"PK\x03\x04".as_slice(), bytes.as_slice());
    assert_eq!(0x1234, number);
    assert_eq!(vec![0xff; 4], mask);
    assert_eq!(parse_rule("PK", "0x10000", "0xFFFF").unwrap_err(),
        "value exceeds 16 bits");
    assert_eq!(parse_rule("PK", "1", "0xFF").unwrap_err(),
        "mask length must equal magic length");
    Ok(())
}
```

## Quick Start

### Hexadecimal Bytes

```rust
use qubit_codec_misc::HexCodec;

fn main() {
    let codec = HexCodec::upper()
        .with_prefix("0x")
        .with_separator(" ");

    let encoded = codec.encode(&[0x1f, 0x8b, 0x00, 0xff]);
    assert_eq!("0x1F 8B 00 FF", encoded);

    let decoded = codec
        .decode("0x1F 8B 00 FF")
        .expect("hex text should decode");
    assert_eq!(vec![0x1f, 0x8b, 0x00, 0xff], decoded);
}
```

### Base64 Bytes

```rust
use qubit_codec_misc::Base64Codec;

fn main() {
    let codec = Base64Codec::standard();

    let encoded = codec.encode(b"hello");
    assert_eq!("aGVsbG8=", encoded);

    let decoded = codec
        .decode("aGVsbG8=")
        .expect("Base64 text should decode");
    assert_eq!(b"hello".to_vec(), decoded);
}
```

### URL-Safe Base64 Without Padding

```rust
use qubit_codec_misc::Base64Codec;

fn main() {
    let codec = Base64Codec::url_safe_no_pad();

    let encoded = codec.encode(&[251, 255, 239]);
    assert_eq!("-__v", encoded);

    let decoded = codec
        .decode("-__v")
        .expect("URL-safe Base64 text should decode");
    assert_eq!(vec![251, 255, 239], decoded);
}
```

### C String Literal Bytes

```rust
use qubit_codec_misc::CStringLiteralCodec;

fn main() {
    let codec = CStringLiteralCodec::new();

    let decoded = codec
        .decode(r"PK\003\004")
        .expect("C string literal should decode");
    assert_eq!(b"PK\x03\x04".to_vec(), decoded);

    let encoded = codec.encode(&[0xd0, 0xcf, 0x11, 0xe0]);
    assert_eq!(r"\xD0\xCF\x11\xE0", encoded);
}
```

Hexadecimal escapes consume one or two digits. Octal escapes consume up to
three digits and retain the low eight bits. Universal `\u` escapes require
exactly four hex digits, and `\U` escapes require exactly eight; both must fit
in one byte and do not convert Unicode to UTF-8. Surrounding quotes are not
removed. For example, `\x123` yields `[0x12, b'3']` and `\777` yields
`[0xff]`. Raw source text may contain printable ASCII, tab, LF, VT, and FF;
other characters must be escaped. An escaped space (`\ `) is accepted. For
streaming input, call `Codec::decode_eof` after confirming EOF to resolve
format-specific short escapes.

### C Integer Literals

```rust
use qubit_codec_misc::CIntegerLiteralCodec;

fn main() {
    let codec = CIntegerLiteralCodec::new();

    assert_eq!(123, codec.decode("123").expect("decimal should decode"));
    assert_eq!(83, codec.decode("0123").expect("octal should decode"));
    assert_eq!(
        0xbeef_c0de,
        codec.decode("0xBEEFC0DE").expect("hex should decode")
    );
}
```

### Percent-Encoding UTF-8 Text

```rust
use qubit_codec_misc::PercentCodec;

fn main() {
    let codec = PercentCodec::new();

    let encoded = codec.encode("a b/中");
    assert_eq!("a%20b%2F%E4%B8%AD", encoded);

    let decoded = codec
        .decode("a%20b%2F%E4%B8%AD")
        .expect("percent-encoded text should decode");
    assert_eq!("a b/中", decoded);
}
```

### Form URL Encoding

```rust
use qubit_codec_misc::FormUrlencodedCodec;

fn main() {
    let codec = FormUrlencodedCodec::new();

    let encoded = codec.encode("name=Qubit Codec");
    assert_eq!("name%3DQubit+Codec", encoded);

    let decoded = codec
        .decode("name%3DQubit+Codec")
        .expect("form-url-encoded text should decode");
    assert_eq!("name=Qubit Codec", decoded);
}
```

### Generic Trait Usage

Use the traits when application code should depend on an encoding capability
instead of a concrete codec type.

These traits are defined by the `qubit-codec` crate, so generic call sites
should declare that crate as a direct dependency.

```rust
use qubit_codec::ValueEncoder;
use qubit_codec_misc::{
    MiscCodecError,
    HexCodec,
};

fn encode_payload<C>(
    codec: &mut C,
    payload: &[u8],
) -> Result<String, MiscCodecError>
where
    C: ValueEncoder<[u8], Output = String, Error = MiscCodecError>,
{
    codec.encode(payload)
}

fn main() {
    let mut codec = HexCodec::new();
    let text = encode_payload(&mut codec, &[0xab, 0xcd])
        .expect("hex encoding should not fail");
    assert_eq!("abcd", text);
}
```

## Safe Streaming Hex Byte Adapter

Add both direct dependencies:

```toml
[dependencies]
qubit-codec = "0.17"
qubit-codec-misc = "0.8"
```

`HexByteCodec` decodes one byte at a time. It does not accept `HexCodec` whole-value prefixes, per-byte prefixes, or separator configuration; those formatting options do not transfer between the codecs. Keep unconsumed input in `pending`, remove only the `read` prefix, and treat only the `written` output prefix as valid. This fixed two-byte output is sufficient for the example and explicitly cannot produce `NeedOutput`; a general driver must grow or replace its output buffer on `NeedOutput` and retry with unconsumed input. `Complete` means the current call consumed its visible input, not that the source reached EOF. Once the source confirms EOF, call `transcode_eof` with the pending tail, then `finish`.

```rust
use qubit_codec::CodecTranscodeDecoder;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec_misc::HexByteCodec;

fn main() {
    let mut decoder = CodecTranscodeDecoder::new(HexByteCodec::new());
    let chunks: [&[u8]; 3] = [b"1", b"f8", b"b"];
    let mut pending = Vec::new();
    let mut output = [0_u8; 2];
    let mut written = 0;

    decoder.reset(&mut output, 0).expect("reset decoder");
    for (index, chunk) in chunks.iter().enumerate() {
        pending.extend_from_slice(chunk);
        let progress = decoder
            .transcode(&pending, 0, &mut output, written)
            .expect("decode available input");
        let read = progress.read();
        written += progress.written();
        pending.drain(..read);
        assert!(!matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
        if index < 2 {
            assert!(matches!(progress.status(), TranscodeStatus::NeedInput { .. }));
        } else {
            assert_eq!(TranscodeStatus::Complete, progress.status());
        }
    }

    // Complete says this call consumed its visible input; only the source tells us it has ended.
    assert!(pending.is_empty());
    let eof = decoder
        .transcode_eof(&pending, 0, &mut output, written)
        .expect("finish decoding at confirmed EOF");
    assert_eq!(TranscodeStatus::Complete, eof.status());
    written += eof.written();
    assert_eq!([0x1f, 0x8b], output);
    assert_eq!(2, written);
    assert_eq!(0, decoder.finish(&mut output, written).expect("finish adapter"));

    output = [0xa5; 2];
    decoder.reset(&mut output, 0).expect("reset for short input");
    let error = decoder.transcode_eof(b"1", 0, &mut output, 0).unwrap_err();
    assert!(matches!(
        error,
        TranscodeDecodeError::Failure(TranscodeFailure::IncompleteInput {
            input_index: 0,
            required: 2,
            available: 1
        })
    ));
    assert_eq!([0xa5; 2], output);

    decoder.reset(&mut output, 0).expect("reset for invalid input");
    let error = decoder.transcode(b"1g", 0, &mut output, 0).unwrap_err();
    assert!(matches!(
        error.domain_ref(),
        Some(qubit_codec_misc::MiscCodecError::InvalidDigit { index: 1, .. })
    ));
    assert_eq!([0xa5; 2], output);
}
```

For the related short-escape boundary, see the [CStringLiteralCodec EOF short-escape test](https://github.com/qubit-ltd/rs-codec-misc/blob/main/tests/codec_tests.rs) (the test checks that a short escape is resolved only after EOF is known).

## API Reference

### Trait Operations

| Trait | Method | Description |
|-------|--------|-------------|
| `ValueEncoder<Input>` | `encode(&Input)` | Encode borrowed input into an associated output type |
| `ValueDecoder<Input>` | `decode(&Input)` | Decode borrowed input into an associated output type |
| `Codec` with associated `Value` and `Unit` | `decode`, `encode` | Convert one value or codec quantum against caller-provided unit buffers |

The low-level `Codec` implementations intentionally exclude facade concerns:
hex prefix/separator handling, UTF-8 `String` validation, and Base64 final
padding are handled by value helpers or future buffered layers.

### `HexCodec` Operations

| Method | Description |
|--------|-------------|
| `new()` | Create a lowercase codec without prefix or separators |
| `upper()` | Create an uppercase codec without prefix or separators |
| `with_uppercase(enabled)` | Configure digit case |
| `with_prefix(prefix)` | Add and require a whole-output prefix, such as `0x1F8B` |
| `with_byte_prefix(prefix)` | Add and require a prefix before every byte, such as `0x1F 0x8B` |
| `with_separator(separator)` | Add and accept a separator between bytes |
| `with_ignored_ascii_whitespace(enabled)` | Ignore ASCII whitespace while decoding |
| `with_ignore_prefix_case(enabled)` | Ignore ASCII case when matching configured prefixes while decoding |
| `encode(bytes)` | Encode bytes into hexadecimal text |
| `encode_into(bytes, output)` | Append encoded text into an existing `String` |
| `decode(text)` | Decode hexadecimal text into bytes |
| `decode_into(text, output)` | Append decoded bytes into an existing `Vec<u8>` |

### `HexCodecConfig` Operations

| Method | Description |
|--------|-------------|
| `new()` | Create the default lowercase, contiguous-hex configuration |
| `with_uppercase(enabled)` | Configure digit case |
| `with_prefix(prefix)` | Configure a whole-output prefix |
| `with_byte_prefix(prefix)` | Configure a prefix before every byte |
| `with_separator(separator)` | Configure the separator between bytes |
| `with_ignored_ascii_whitespace(enabled)` | Configure whitespace handling while decoding |
| `with_ignore_prefix_case(enabled)` | Configure case-insensitive prefix matching |
| `is_uppercase()` / `prefix()` / `byte_prefix()` | Inspect configured output formatting |
| `separator()` / `ignores_ascii_whitespace()` / `ignores_prefix_case()` | Inspect configured decoding policy |
| `HexCodec::from_config(config)` | Create a codec from the reusable configuration |
| `HexCodec::config()` | Borrow the codec's configuration |

### `HexByteCodec` Operations

| Method or Trait | Description |
|-----------------|-------------|
| `new()` | Create a lowercase single-byte hex codec |
| `upper()` | Create an uppercase single-byte hex codec |
| `with_uppercase(enabled)` | Configure digit case |
| `Codec<Value = u8, Unit = u8>` | Decode or encode one byte as exactly two ASCII hex units |

### `Base64Codec` Operations

| Method | Alphabet | Padding | Description |
|--------|----------|---------|-------------|
| `standard()` | Standard | Yes | Create standard Base64 codec |
| `standard_no_pad()` | Standard | No | Create standard Base64 codec without padding |
| `url_safe()` | URL-safe | Yes | Create URL-safe Base64 codec |
| `url_safe_no_pad()` | URL-safe | No | Create URL-safe Base64 codec without padding |
| `encode(bytes)` | Configured | Configured | Encode bytes into Base64 text |
| `decode(text)` | Configured | Configured | Decode Base64 text into bytes |

The padded and unpadded modes reject each other's partial-group form. For
example, `Base64Codec::standard().decode("Zg")` and
`Base64Codec::standard_no_pad().decode("Zg==")` both return an error, while
both modes accept `"TWFu"` because it encodes a complete three-byte group and
needs no padding.

### `Base64QuantumCodec` Operations

| Method | Alphabet | Units | Description |
|--------|----------|-------|-------------|
| `standard()` | Standard | 4 | Create a standard Base64 quantum codec |
| `url_safe()` | URL-safe | 4 | Create a URL-safe Base64 quantum codec |
| `Codec<Value = [u8; 3], Unit = u8>` | Configured | 4 | Encode or decode one complete Base64 quantum without padding finalization |

### `CStringLiteralCodec` Operations

| Method | Description |
|--------|-------------|
| `new()` | Create a C string literal byte codec |
| `encode(bytes)` | Encode bytes into a C string literal fragment |
| `decode(text)` | Decode a C string literal fragment into bytes |

### `CIntegerLiteralCodec` Operations

| Method | Description |
|--------|-------------|
| `new()` | Create a C integer literal decoder |
| `decode(text)` | Decode a non-negative C integer literal fragment into `u64` |

`CIntegerLiteralCodec` intentionally remains a value-token decoder. It does not
implement `Codec<Value = u64, Unit = u8>` yet because that would require committing to token
boundary and encode-format policy that belongs above the single-value core.

### Text Codec Operations

| Type | Method | Description |
|------|--------|-------------|
| `PercentCodec` | `new()` | Create a percent codec |
| `PercentCodec` | `encode(text)` | Encode UTF-8 text using percent encoding |
| `PercentCodec` | `decode(text)` | Decode percent-encoded UTF-8 text |
| `FormUrlencodedCodec` | `new()` | Create a form-url-encoded codec |
| `FormUrlencodedCodec` | `encode(text)` | Encode UTF-8 text, using `+` for spaces |
| `FormUrlencodedCodec` | `decode(text)` | Decode UTF-8 text, treating `+` as spaces |

## Error Handling

Bundled decoders return `MiscCodecResult<T>`, an alias for
`Result<T, MiscCodecError>`.

| Error | Meaning |
|-------|---------|
| `MissingPrefix` | A configured whole or per-byte hex prefix was required but missing |
| `InvalidDigit` | Input contained a digit that is invalid for the requested radix |
| `InvalidLength` | Input length does not satisfy a codec requirement |
| `InvalidEscape` | Input contained a malformed or unsupported escape sequence |
| `InvalidCharacter` | Input contained a character that cannot appear in that context |
| `InvalidInput` | Input was rejected by a codec-specific validator |
| `InvalidBase64` | Base64 decoding failed; includes a structured category and preserves the underlying `base64::DecodeError` |
| `InvalidUtf8` | Decoded bytes were not valid UTF-8 |

Complete-value helpers report truncated `%XX` and C escapes as
`InvalidEscape`. Low-level `Codec::decode` calls use
`qubit_codec::DecodeFailure::Incomplete` for open-stream tails; callers that
have confirmed EOF should call `Codec::decode_eof` so format-specific short
escapes can be resolved.

## Performance Considerations

Codec implementations operate on borrowed byte slices or strings and return
owned output only when the target format requires it. Configuration is stored in
small value types, and generic trait use does not require dynamic dispatch.

Run `cargo bench --all-features` to measure representative text and Hex
workloads. The fuzz targets under `fuzz/` can be compiled with
`cargo +nightly fuzz build` and run with `cargo +nightly fuzz run
percent_and_form` (or the other target names).

## Dependencies

Runtime dependencies are intentionally small:

- `base64` provides the Base64 engines.
- `percent-encoding` provides the percent escape byte table.
- `thiserror` provides the public error type implementation.

The `form_urlencoded` crate is used only by differential tests and fuzz targets;
it is not part of the runtime API.

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
./.infra/bin/ci-check.sh

# Check code coverage
./.infra/bin/coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./.infra/bin/align-ci.sh` to format code and
`./.infra/bin/ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-codec-misc](https://github.com/qubit-ltd/rs-codec-misc)
