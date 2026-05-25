/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Bidirectional codec trait.

use crate::{
    Decoder,
    Encoder,
};

/// Combines an [`Encoder`] and a [`Decoder`] into a bidirectional codec.
pub trait Codec<EncodeInput: ?Sized, DecodeInput: ?Sized>: Encoder<EncodeInput> + Decoder<DecodeInput> {}

impl<T, EncodeInput: ?Sized, DecodeInput: ?Sized> Codec<EncodeInput, DecodeInput> for T where
    T: Encoder<EncodeInput> + Decoder<DecodeInput>
{
}
