// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for reusable hexadecimal codec configuration.

use qubit_codec_misc::HexCodecConfig;

#[test]
fn test_hex_codec_config_defaults_are_empty_and_lowercase() {
    let config = HexCodecConfig::new();
    assert_eq!(config, HexCodecConfig::default());

    assert!(!config.is_uppercase());
    assert_eq!(None, config.prefix());
    assert_eq!(None, config.byte_prefix());
    assert_eq!(None, config.separator());
    assert!(!config.ignores_ascii_whitespace());
    assert!(!config.ignores_prefix_case());
}
