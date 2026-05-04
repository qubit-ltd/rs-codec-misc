/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # qubit-codec
//!
//! Reusable byte and text codecs for Rust applications.
//!
//! This crate focuses on stable textual encodings such as hexadecimal,
//! Base64, percent encoding, and `application/x-www-form-urlencoded` strings.
//!

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

mod base64_codec;
mod codec;
mod codec_error;
mod decoder;
mod encoder;
mod form_urlencoded_codec;
mod hex_codec;
mod percent_codec;

pub use base64_codec::Base64Codec;
pub use codec::Codec;
pub use codec_error::{
    CodecError,
    CodecResult,
};
pub use decoder::Decoder;
pub use encoder::Encoder;
pub use form_urlencoded_codec::FormUrlencodedCodec;
pub use hex_codec::HexCodec;
pub use percent_codec::PercentCodec;
