/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # qubit-codec-misc
//!
//! Reusable byte and text codecs for Rust applications.
//!
//! This crate focuses on stable textual encodings such as hexadecimal,
//! Base64, percent encoding, and `application/x-www-form-urlencoded` strings.
//!

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

mod base64_codec;
mod base64_quantum_codec;
mod c_integer_literal_codec;
mod c_string_literal_codec;
mod form_urlencoded_codec;
mod hex_codec;
mod misc_codec_error;
mod percent_codec;

pub use base64_codec::Base64Codec;
pub use base64_quantum_codec::Base64QuantumCodec;
pub use c_integer_literal_codec::CIntegerLiteralCodec;
pub use c_string_literal_codec::CStringLiteralCodec;
pub use form_urlencoded_codec::FormUrlencodedCodec;
pub use hex_codec::HexCodec;
pub use misc_codec_error::{
    MiscCodecError,
    MiscCodecResult,
};
pub use percent_codec::PercentCodec;
pub use qubit_codec::{
    BufferedTranscoder,
    Codec,
    TranscodeProgress,
    TranscodeStatus,
    ValueDecoder,
    ValueEncoder,
};
