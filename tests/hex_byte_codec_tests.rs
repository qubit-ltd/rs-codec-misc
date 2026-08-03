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
//! Tests for the low-level hexadecimal byte codec.

use qubit_codec::Codec;
use qubit_codec_misc::{
    HexByteCodec,
    MiscCodecError,
};

#[test]
fn test_hex_byte_codec_roundtrips_one_byte() {
    let mut codec = HexByteCodec::upper();
    let mut output = [0u8; 2];

    let (decoded, consumed) = unsafe {
        Codec::decode(&mut codec, b"Af", 0)
            .expect("single hex byte should decode")
    };
    let written = unsafe {
        Codec::encode(&mut codec, &0xaf, &mut output, 0)
            .expect("single hex byte should encode")
    };

    assert_eq!(0xaf, decoded);
    assert_eq!(2, consumed.get());
    assert_eq!(2, written);
    assert_eq!(b"AF", &output);
    assert_eq!(2, <HexByteCodec as Codec>::MIN_UNITS_PER_VALUE);
    assert_eq!(2, <HexByteCodec as Codec>::MAX_ENCODE_UNITS_PER_VALUE);
    assert_eq!(2, <HexByteCodec as Codec>::MAX_DECODE_UNITS_PER_VALUE);
    assert!(codec.is_uppercase());
    assert!(!HexByteCodec::upper().with_uppercase(false).is_uppercase());
}

#[test]
fn test_hex_byte_codec_reports_digit_positions() {
    let mut codec = HexByteCodec::new();
    let high = unsafe { Codec::decode(&mut codec, b"xf", 0) }
        .expect_err("invalid high hex digit should fail");
    let low = unsafe { Codec::decode(&mut codec, b"fx", 0) }
        .expect_err("invalid low hex digit should fail");

    assert!(matches!(
        super::invalid_source(high),
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 0,
            character: 'x'
        }
    ));
    assert!(matches!(
        super::invalid_source(low),
        MiscCodecError::InvalidDigit {
            radix: 16,
            index: 1,
            character: 'x'
        }
    ));
}
