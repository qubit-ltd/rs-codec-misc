// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Regression coverage for the two percent encoding policies.

#[test]
fn test_percent_policy_regressions_are_covered_by_public_form_tests() {
    assert_eq!(
        "*%7E",
        qubit_codec_misc::FormUrlencodedCodec::new().encode("*~")
    );
}
