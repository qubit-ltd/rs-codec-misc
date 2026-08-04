// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for structured Base64 error categories.

use qubit_codec_misc::Base64ErrorKind;

#[test]
fn test_base64_error_kind_variants_are_distinct() {
    assert_ne!(Base64ErrorKind::InvalidByte, Base64ErrorKind::InvalidLength);
    assert_ne!(
        Base64ErrorKind::InvalidLastSymbol,
        Base64ErrorKind::InvalidPadding
    );
}
