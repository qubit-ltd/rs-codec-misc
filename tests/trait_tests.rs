// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for lightweight encoder and decoder traits.

use qubit_codec::CodecTranscodeDecoder;
use qubit_codec::CodecTranscodeEncoder;
use qubit_codec::CodecValueEncoder;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeDecoder;
use qubit_codec::TranscodeEncodeError;
use qubit_codec::TranscodeEncoder;
use qubit_codec::TranscodeFailure;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::engine::TranscodeDecodeEngine;
use qubit_codec::engine::TranscodeEncodeEngine;
use qubit_codec_misc::FormUrlencodedCodec;
use qubit_codec_misc::HexByteCodec;
use qubit_codec_misc::HexCodec;
use qubit_codec_misc::MiscCodecError;
use qubit_codec_misc::PercentCodec;

#[test]
fn test_codec_types_can_be_used_through_traits() {
    let mut codec = HexCodec::new();
    let encoded =
        ValueEncoder::<[u8]>::encode(&mut codec, b"abc").expect("hex encode should succeed");
    let decoded =
        ValueDecoder::<str>::decode(&mut codec, &encoded).expect("hex decode should succeed");

    assert_eq!("616263", encoded);
    assert_eq!(b"abc".to_vec(), decoded);
}

#[test]
fn test_core_codec_adapter_types_can_wrap_misc_codecs() {
    fn assert_codec_value_encoder<
        T: ValueEncoder<u8, Output = Vec<u8>, Error = TranscodeEncodeError<MiscCodecError, u8>>,
    >() {
    }
    fn assert_codec_transcode_decoder<T: TranscodeDecoder<Input = u8, Output = u8>>() {}
    fn assert_codec_transcode_encoder<T: TranscodeEncoder<Input = u8, Output = u8>>() {}
    fn assert_transcode_decode_engine<T>() {}
    fn assert_transcode_encode_engine<T>() {}

    assert_codec_value_encoder::<CodecValueEncoder<HexByteCodec>>();
    assert_codec_transcode_decoder::<CodecTranscodeDecoder<HexByteCodec>>();
    assert_codec_transcode_encoder::<CodecTranscodeEncoder<HexByteCodec>>();
    assert_transcode_decode_engine::<TranscodeDecodeEngine<HexByteCodec, ()>>();
    assert_transcode_encode_engine::<TranscodeEncodeEngine<HexByteCodec, ()>>();

    let encode_error = TranscodeEncodeError::<core::convert::Infallible, u8>::unencodable(2, 0xff);
    assert!(matches!(
        encode_error,
        TranscodeEncodeError::Unencodable { .. }
    ));
    let decode_error: TranscodeDecodeError<core::convert::Infallible> =
        TranscodeFailure::incomplete_input(2, 3, 1).into();
    assert!(matches!(
        decode_error,
        TranscodeDecodeError::Failure(TranscodeFailure::IncompleteInput { .. })
    ));
    let transcode_error: TranscodeDecodeError<core::convert::Infallible> =
        TranscodeFailure::invalid_input_index(2, 1).into();
    assert!(matches!(
        transcode_error,
        TranscodeDecodeError::Failure(TranscodeFailure::InvalidInputIndex { .. })
    ));
}

#[test]
fn test_value_traits_accept_text_codecs() {
    fn roundtrip<C>(mut codec: C, text: &str) -> String
    where
        C: ValueEncoder<str, Output = String, Error = MiscCodecError>
            + ValueDecoder<str, Output = String, Error = MiscCodecError>,
    {
        let encoded = ValueEncoder::<str>::encode(&mut codec, text).expect("text should encode");
        ValueDecoder::<str>::decode(&mut codec, &encoded).expect("text should decode")
    }

    assert_eq!("a b", roundtrip(PercentCodec::new(), "a b"));
    assert_eq!("a b", roundtrip(FormUrlencodedCodec::new(), "a b"));
}
