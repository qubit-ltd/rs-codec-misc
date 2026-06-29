// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for lightweight encoder and decoder traits.

use qubit_codec::{
    CodecTranscodeDecoder, CodecTranscodeEncoder, CodecValueEncoder, EncodeOutcome,
    TranscodeDecodeEngine, TranscodeDecoder, TranscodeEncodeEngine, TranscodeEncoder,
    TranscodeError,
};
use qubit_codec_misc::{
    FormUrlencodedCodec, HexByteCodec, HexCodec, PercentCodec, ValueDecoder, ValueEncoder,
};

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
        T: ValueEncoder<
                u8,
                Output = Vec<u8>,
                Error = TranscodeError<qubit_codec_misc::MiscCodecError>,
            >,
    >() {
    }
    fn assert_codec_transcode_decoder<T: TranscodeDecoder<u8, u8>>() {}
    fn assert_codec_transcode_encoder<T: TranscodeEncoder<u8, u8>>() {}
    fn assert_transcode_decode_engine<T>() {}
    fn assert_transcode_encode_engine<T>() {}

    assert_codec_value_encoder::<CodecValueEncoder<HexByteCodec>>();
    assert_codec_transcode_decoder::<CodecTranscodeDecoder<HexByteCodec>>();
    assert_codec_transcode_encoder::<CodecTranscodeEncoder<HexByteCodec>>();
    assert_transcode_decode_engine::<TranscodeDecodeEngine<HexByteCodec, ()>>();
    assert_transcode_encode_engine::<TranscodeEncodeEngine<HexByteCodec, ()>>();

    assert_eq!(
        EncodeOutcome::consumed(1),
        EncodeOutcome::Consumed { written: 1 }
    );
    let encode_error = TranscodeError::<core::convert::Infallible>::unencodable_value(2);
    assert!(matches!(
        encode_error,
        TranscodeError::UnencodableValue { .. }
    ));
    let decode_error = TranscodeError::<core::convert::Infallible>::incomplete_input(2, 3, 1);
    assert!(matches!(
        decode_error,
        TranscodeError::IncompleteInput { .. }
    ));
    let transcode_error = TranscodeError::<core::convert::Infallible>::invalid_input_index(2, 1);
    assert!(matches!(
        transcode_error,
        TranscodeError::InvalidInputIndex { .. }
    ));
}

#[test]
fn test_value_traits_accept_text_codecs() {
    fn roundtrip<C>(mut codec: C, text: &str) -> String
    where
        C: ValueEncoder<str, Output = String, Error = qubit_codec_misc::MiscCodecError>
            + ValueDecoder<str, Output = String, Error = qubit_codec_misc::MiscCodecError>,
    {
        let encoded = ValueEncoder::<str>::encode(&mut codec, text).expect("text should encode");
        ValueDecoder::<str>::decode(&mut codec, &encoded).expect("text should decode")
    }

    assert_eq!("a b", roundtrip(PercentCodec::new(), "a b"));
    assert_eq!("a b", roundtrip(FormUrlencodedCodec::new(), "a b"));
}
