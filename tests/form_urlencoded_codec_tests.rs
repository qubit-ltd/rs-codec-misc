// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for application/x-www-form-urlencoded text encoding.

use qubit_codec_misc::FormUrlencodedCodec;

#[test]
fn test_form_urlencoded_codec_uses_plus_for_spaces() {
    let codec = FormUrlencodedCodec::new();

    assert_eq!(
        "name%3Da+b%2Bc%26city%3D%E4%B8%8A%E6%B5%B7",
        codec.encode("name=a b+c&city=上海")
    );
    assert_eq!(
        "name=a b+c&city=上海",
        codec
            .decode("name%3Da+b%2Bc%26city%3D%E4%B8%8A%E6%B5%B7")
            .expect("form text should decode")
    );
}
