// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # qubit-codec-misc
//!
//! Reusable byte and text codecs for Rust applications.
//!
//! This crate provides whole-value codecs, byte-level codecs, and streaming
//! Base64 adapters for common text formats.
//!
//! | Format | Owned value API | Low-level or streaming API |
//! | --- | --- | --- |
//! | Hexadecimal text with configurable formatting | [`HexCodec`] and [`HexCodecConfig`] | [`HexByteCodec`] converts one byte and does not handle prefixes or separators. |
//! | Base64 values | [`Base64Codec`] | [`Base64QuantumCodec`] handles one complete quantum; [`Base64Encoder`] and [`Base64Decoder`] handle streams. |
//! | C string literal byte fragments | [`CStringLiteralCodec`] | The same type implements [`qubit_codec::Codec`] for one byte at a time. |
//! | Non-negative C integer literal fragments | [`CIntegerLiteralCodec`] | A value decoder only; it does not expose a low-level byte codec or token-stream interface. |
//! | UTF-8 percent-encoded text | [`PercentCodec`] | The same type implements [`qubit_codec::Codec`] over bytes. |
//! | UTF-8 form-encoded text fragments | [`FormUrlencodedCodec`] | The same type implements [`qubit_codec::Codec`] over bytes. |
//!
//! Choose an owned method such as [`HexCodec::encode`] or
//! [`CStringLiteralCodec::decode`] when the complete value is available and a
//! convenient `String` or `Vec<u8>` result is wanted. Use
//! [`qubit_codec::Codec`] when integrating a single byte or complete quantum
//! into a caller-owned buffer. Use [`qubit_codec::Transcoder`] with
//! [`Base64Encoder`] or [`Base64Decoder`] for stateful Base64 streams.
//!
//! Direct calls to [`qubit_codec::Codec`] methods require callers to satisfy
//! the method's buffer capacity, index, and input validity preconditions.
//! Read the relevant type documentation and examples before calling methods
//! directly. In particular, [`HexByteCodec`] handles a byte, not a formatted
//! hexadecimal token. The low-level percent and form codecs transform bytes;
//! UTF-8 validity is checked by their owned text APIs, not by each byte call.
//! [`CIntegerLiteralCodec`] decodes complete unsigned literal values and does
//! not define token boundaries for streams. C string literal fragments are a
//! byte escape format and do not claim to implement every C compiler's source
//! syntax.
//!
//! Base64 stream adapters must be [`qubit_codec::Transcoder::reset`] before
//! streaming and finalized with `finish`. A decoder must be told when the
//! source has reached EOF by using `transcode_eof`; the end of a buffer alone
//! does not establish EOF. See the [`Base64Encoder`] and [`Base64Decoder`]
//! examples for buffer handling, and the [`Base64Codec`], [`HexByteCodec`], and
//! [`CStringLiteralCodec`] examples for whole-value and byte-level usage.
//!
//! ```
//! use qubit_codec_misc::CStringLiteralCodec;
//!
//! let bytes = CStringLiteralCodec::new().decode(r"PK\003\004")?;
//! assert_eq!(bytes, b"PK\x03\x04");
//! # Ok::<(), qubit_codec_misc::MiscCodecError>(())
//! ```

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

mod base64_codec;
mod base64_decoder;
mod base64_encoder;
mod base64_error_kind;
mod base64_quantum_codec;
mod c_integer_literal_codec;
mod c_string_literal_codec;
mod form_urlencoded_codec;
mod hex;
mod hex_byte_codec;
mod hex_codec_config;
mod internal;
mod misc_error;
mod percent;

pub use base64_codec::Base64Codec;
pub use base64_decoder::Base64Decoder;
pub use base64_encoder::Base64Encoder;
pub use base64_error_kind::Base64ErrorKind;
pub use base64_quantum_codec::Base64QuantumCodec;
pub use c_integer_literal_codec::CIntegerLiteralCodec;
pub use c_string_literal_codec::CStringLiteralCodec;
pub use form_urlencoded_codec::FormUrlencodedCodec;
pub use hex::HexCodec;
pub use hex_byte_codec::HexByteCodec;
pub use hex_codec_config::HexCodecConfig;
pub use misc_error::MiscCodecError;
pub use misc_error::MiscCodecResult;
pub use percent::PercentCodec;
