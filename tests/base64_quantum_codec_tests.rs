// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
// =============================================================================
//! Tests for complete Base64 quantum encoding and decoding.

use qubit_codec::Codec;
use qubit_codec_misc::{
    Base64QuantumCodec,
    MiscCodecError,
};

#[test]
fn test_base64_quantum_standard_roundtrip() {
    let mut codec = Base64QuantumCodec::standard();
    let mut output = [0u8; 4];

    let (decoded, consumed) = unsafe {
        Codec::decode(&mut codec, b"YWJj", 0)
            .expect("base64 quantum should decode")
    };
    let written = unsafe {
        Codec::encode(&mut codec, b"abc", &mut output, 0)
            .expect("base64 quantum should encode")
    };

    assert_eq!(*b"abc", decoded);
    assert_eq!(4, consumed.get());
    assert_eq!(4, written);
    assert_eq!(b"YWJj", &output);
}

#[test]
fn test_base64_quantum_url_safe_roundtrip() {
    let mut codec = Base64QuantumCodec::url_safe();
    let mut output = [0u8; 4];

    let (decoded, consumed) = unsafe {
        Codec::decode(&mut codec, b"-__u", 0)
            .expect("URL-safe quantum should decode")
    };
    let written = unsafe {
        Codec::encode(&mut codec, &[0xfb, 0xff, 0xee], &mut output, 0)
            .expect("URL-safe quantum should encode")
    };

    assert_eq!([0xfb, 0xff, 0xee], decoded);
    assert_eq!(4, consumed.get());
    assert_eq!(4, written);
    assert_eq!(b"-__u", &output);
}

#[test]
fn test_base64_quantum_alphabet_and_errors() {
    let mut standard = Base64QuantumCodec::default();
    let mut url_safe = Base64QuantumCodec::url_safe();

    let (decoded, _) = unsafe { Codec::decode(&mut standard, b"++//", 0) }
        .expect("standard symbols should decode");
    assert_eq!([0xfb, 0xef, 0xff], decoded);

    let (decoded, _) = unsafe { Codec::decode(&mut standard, b"0123", 0) }
        .expect("digit symbols should decode");
    assert_eq!([0xd3, 0x5d, 0xb7], decoded);
    assert!(matches!(
        super::invalid_source(
            unsafe { Codec::decode(&mut url_safe, b"@@@@", 0) }
                .expect_err("invalid Base64 quantum should fail")
        ),
        MiscCodecError::InvalidInput {
            codec: "base64-quantum",
            ..
        }
    ));
}
