// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for lightweight encoder and decoder traits.

use qubit_codec::{
    CodecTranscodeDecoder,
    CodecTranscodeEncoder,
    CodecValueEncoder,
    EncodeOutcome,
    TranscodeDecodeEngine,
    TranscodeDecoder,
    TranscodeEncodeEngine,
    TranscodeEncoder,
    TranscodeError,
    TranscodeFailure,
};
use qubit_codec_misc::{
    Base64Codec,
    CIntegerLiteralCodec,
    CStringLiteralCodec,
    FormUrlencodedCodec,
    HexByteCodec,
    HexCodec,
    MiscCodecError,
    PercentCodec,
    ValueDecoder,
    ValueEncoder,
};

#[test]
fn test_codec_types_can_be_used_through_traits() {
    let mut codec = HexCodec::new();
    let encoded = ValueEncoder::<[u8]>::encode(&mut codec, b"abc")
        .expect("hex encode should succeed");
    let decoded = ValueDecoder::<str>::decode(&mut codec, &encoded)
        .expect("hex decode should succeed");

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
    let encode_error =
        TranscodeError::<core::convert::Infallible>::unencodable_value(2);
    assert!(matches!(
        encode_error,
        TranscodeError::Failure(TranscodeFailure::UnencodableValue { .. })
    ));
    let decode_error =
        TranscodeError::<core::convert::Infallible>::incomplete_input(2, 3, 1);
    assert!(matches!(
        decode_error,
        TranscodeError::Failure(TranscodeFailure::IncompleteInput { .. })
    ));
    let transcode_error =
        TranscodeError::<core::convert::Infallible>::invalid_input_index(2, 1);
    assert!(matches!(
        transcode_error,
        TranscodeError::Failure(TranscodeFailure::InvalidInputIndex { .. })
    ));
}

#[test]
fn test_value_traits_accept_text_codecs() {
    fn roundtrip<C>(mut codec: C, text: &str) -> String
    where
        C: ValueEncoder<
                str,
                Output = String,
                Error = qubit_codec_misc::MiscCodecError,
            > + ValueDecoder<
                str,
                Output = String,
                Error = qubit_codec_misc::MiscCodecError,
            >,
    {
        let encoded = ValueEncoder::<str>::encode(&mut codec, text)
            .expect("text should encode");
        ValueDecoder::<str>::decode(&mut codec, &encoded)
            .expect("text should decode")
    }

    assert_eq!("a b", roundtrip(PercentCodec::new(), "a b"));
    assert_eq!("a b", roundtrip(FormUrlencodedCodec::new(), "a b"));
}

#[test]
fn test_value_trait_map_error_methods_return_domain_errors() {
    fn sample_error() -> MiscCodecError {
        MiscCodecError::InvalidInput {
            codec: "test",
            reason: "sample".to_owned(),
        }
    }

    assert_eq!(
        "invalid test input: sample",
        ValueEncoder::<[u8]>::map_error(
            &Base64Codec::standard(),
            sample_error()
        )
        .to_string(),
    );
    assert_eq!(
        "invalid test input: sample",
        ValueDecoder::<str>::map_error(
            &Base64Codec::standard(),
            sample_error()
        )
        .to_string(),
    );
    assert_eq!(
        "invalid test input: sample",
        ValueEncoder::<[u8]>::map_error(
            &CStringLiteralCodec::new(),
            sample_error()
        )
        .to_string(),
    );
    assert_eq!(
        "invalid test input: sample",
        ValueDecoder::<str>::map_error(
            &CStringLiteralCodec::new(),
            sample_error()
        )
        .to_string(),
    );
    assert_eq!(
        "invalid test input: sample",
        ValueDecoder::<str>::map_error(
            &CIntegerLiteralCodec::new(),
            sample_error()
        )
        .to_string(),
    );
    assert_eq!(
        "invalid test input: sample",
        ValueEncoder::<str>::map_error(&PercentCodec::new(), sample_error())
            .to_string(),
    );
    assert_eq!(
        "invalid test input: sample",
        ValueDecoder::<str>::map_error(&PercentCodec::new(), sample_error())
            .to_string(),
    );
    assert_eq!(
        "invalid test input: sample",
        ValueEncoder::<str>::map_error(
            &FormUrlencodedCodec::new(),
            sample_error()
        )
        .to_string(),
    );
    assert_eq!(
        "invalid test input: sample",
        ValueDecoder::<str>::map_error(
            &FormUrlencodedCodec::new(),
            sample_error()
        )
        .to_string(),
    );
    assert_eq!(
        "invalid test input: sample",
        ValueEncoder::<[u8]>::map_error(&HexCodec::new(), sample_error())
            .to_string(),
    );
    assert_eq!(
        "invalid test input: sample",
        ValueDecoder::<str>::map_error(&HexCodec::new(), sample_error())
            .to_string(),
    );
}
