// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Regression coverage for parser error translation.

use qubit_codec_misc::MiscCodecError;
use qubit_codec_misc::PercentCodec;

#[test]
fn test_complete_percent_decode_exposes_format_error_not_stream_state() {
    assert!(matches!(
        PercentCodec::new().decode("%"),
        Err(MiscCodecError::InvalidEscape { index: 0, .. })
    ));
}
