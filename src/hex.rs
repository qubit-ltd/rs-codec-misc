// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Whole-slice hexadecimal byte codec.

mod hex_codec;

pub use hex_codec::HexCodec;
pub(crate) use hex_codec::hex_digit;
pub(crate) use hex_codec::hex_value;
pub(crate) use hex_codec::invalid_hex_digit;
