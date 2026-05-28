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
  convenience traits. `CodecValueEncoder` and `CodecBufferedEncoder` provide
  default encoder adapters for low-level `Codec` implementations.
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
- **Buffer APIs**: `encode_into` and `decode_into` append into existing buffers.

### 🔐 **Base64 Bytes**

- **Standard Alphabet**: padded and no-padding standard Base64.
- **URL-Safe Alphabet**: padded and no-padding URL-safe Base64.
- **Quantum Core**: `Base64QuantumCodec` handles complete three-byte to
  four-unit Base64 quanta; final padding stays in the facade/transcoder layer.
- **Typed Errors**: malformed input is reported as `MiscCodecError::InvalidInput`.

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
- **Percent Compatibility**: shares the same UTF-8 and `%XX` validation behavior
  as `PercentCodec`.

### 🎯 **Focused Public API**

- **`ValueEncoder<Input>`**: encodes borrowed input into an associated output type.
- **`ValueDecoder<Input>`**: decodes borrowed input into an associated output type.
- **`Codec<Value, Unit>`**: low-level unsafe trait for one value or one codec
  quantum over caller-provided unit buffers.
- **`CodecValueEncoder<C, Value, Unit>` / `CodecBufferedEncoder<C>`**: default
  encoder adapters re-exported from `qubit-codec`.
- **`MiscCodecError` / `MiscCodecResult`**: common error and result types for bundled
  codecs.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qubit-codec-misc = "0.1"
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

```rust
use qubit_codec_misc::{
    MiscCodecError,
    ValueEncoder,
    HexCodec,
};

fn encode_payload<C>(codec: &C, payload: &[u8]) -> Result<String, MiscCodecError>
where
    C: ValueEncoder<[u8], Output = String, Error = MiscCodecError>,
{
    codec.encode(payload)
}

fn main() {
    let text = encode_payload(&HexCodec::new(), &[0xab, 0xcd])
        .expect("hex encoding should not fail");
    assert_eq!("abcd", text);
}
```

## API Reference

### Trait Operations

| Trait | Method | Description |
|-------|--------|-------------|
| `ValueEncoder<Input>` | `encode(&Input)` | Encode borrowed input into an associated output type |
| `ValueDecoder<Input>` | `decode(&Input)` | Decode borrowed input into an associated output type |
| `Codec<Value, Unit>` | `decode_unchecked`, `encode_unchecked` | Convert one value or codec quantum against caller-provided unit buffers |
| `CodecValueEncoder<C, Value, Unit>` | `encode(&Value)` | Encode one value into owned `Vec<Unit>` through `C: Codec<Value, Unit>` |
| `CodecBufferedEncoder<C>` | `transcode(...)` | Encode value slices into caller-provided unit buffers through `C: Codec<Value, Unit>` |

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

### `Base64Codec` Operations

| Method | Alphabet | Padding | Description |
|--------|----------|---------|-------------|
| `standard()` | Standard | Yes | Create standard Base64 codec |
| `standard_no_pad()` | Standard | No | Create standard Base64 codec without padding |
| `url_safe()` | URL-safe | Yes | Create URL-safe Base64 codec |
| `url_safe_no_pad()` | URL-safe | No | Create URL-safe Base64 codec without padding |
| `encode(bytes)` | Configured | Configured | Encode bytes into Base64 text |
| `decode(text)` | Configured | Configured | Decode Base64 text into bytes |

### `Base64QuantumCodec` Operations

| Method | Alphabet | Units | Description |
|--------|----------|-------|-------------|
| `standard()` | Standard | 4 | Create a standard Base64 quantum codec |
| `url_safe()` | URL-safe | 4 | Create a URL-safe Base64 quantum codec |
| `Codec<[u8; 3], u8>` | Configured | 4 | Encode or decode one complete Base64 quantum without padding finalization |

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
implement `Codec<u64, u8>` yet because that would require committing to token
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
| `InvalidUtf8` | Decoded bytes were not valid UTF-8 |

## Performance Considerations

Codec implementations operate on borrowed byte slices or strings and return
owned output only when the target format requires it. Configuration is stored in
small value types, and generic trait use does not require dynamic dispatch.

## Testing & Code Coverage

This project keeps codec behavior covered by integration tests under `tests/`.

### Running Tests

```bash
# Run all tests
cargo test

# Run with coverage report
./coverage.sh

# Generate text format report
./coverage.sh text

# Align code with CI requirements
./align-ci.sh

# Run CI checks (format, clippy, test, coverage, audit)
./ci-check.sh
```

## Dependencies

Runtime dependencies are intentionally small:

- `base64` provides the Base64 engines.
- `thiserror` provides the public error type implementation.

## License

Copyright (c) 2026. Haixing Hu.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

See [LICENSE](LICENSE) for the full license text.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

- Follow Rust API Guidelines.
- Keep tests comprehensive and deterministic.
- Document public APIs and behavior changes.
- Ensure all checks pass before submitting a PR.

## Author

**Haixing Hu**

## Related Projects

More Rust libraries from Qubit are available under the
[qubit-ltd](https://github.com/qubit-ltd) GitHub organization.

---

Repository: [https://github.com/qubit-ltd/rs-codec-misc](https://github.com/qubit-ltd/rs-codec-misc)
