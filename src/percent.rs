// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Percent text codec.

mod internal;
mod percent_codec;

pub(crate) use internal::percent_decode_byte;
pub(crate) use internal::percent_decode_bytes;
pub(crate) use internal::percent_encode_byte;
pub(crate) use internal::percent_encode_bytes;
pub(crate) use internal::percent_encode_len;
pub(crate) use internal::percent_invalid_consumed;
pub use percent_codec::PercentCodec;
