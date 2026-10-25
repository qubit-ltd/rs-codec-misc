// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for lightweight encoder and decoder traits.

use qubit_codec::Codec;
use qubit_codec::CodecTranscodeDecoder;
use qubit_codec::CodecTranscodeEncoder;
use qubit_codec::CodecValueEncoder;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeDecoder;
use qubit_codec::TranscodeEncodeError;
use qubit_codec::TranscodeEncoder;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec_misc::CStringLiteralCodec;
use qubit_codec_misc::FormUrlencodedCodec;
use qubit_codec_misc::HexByteCodec;
use qubit_codec_misc::HexCodec;
use qubit_codec_misc::MiscCodecError;
use qubit_codec_misc::PercentCodec;

#[test]
fn test_codec_types_can_be_used_through_traits() {
    let mut codec = HexCodec::new();
    let encoded = ValueEncoder::<[u8]>::encode(&mut codec, b"abc").expect("hex encode should succeed");
    let decoded = ValueDecoder::<str>::decode(&mut codec, &encoded).expect("hex decode should succeed");

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

    assert_codec_value_encoder::<CodecValueEncoder<HexByteCodec>>();
    assert_codec_transcode_decoder::<CodecTranscodeDecoder<HexByteCodec>>();
    assert_codec_transcode_encoder::<CodecTranscodeEncoder<HexByteCodec>>();

    let encode_error = TranscodeEncodeError::<core::convert::Infallible, u8>::unencodable(2, 0xff);
    assert!(matches!(encode_error, TranscodeEncodeError::Unencodable { .. }));
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
fn test_hex_adapter_encoder_retries_small_output() {
    let mut encoder = CodecTranscodeEncoder::new(HexByteCodec::new());
    let mut output = [0xa5; 4];
    assert_eq!(0, encoder.reset(&mut [], 0).unwrap());
    let short = encoder.transcode(&[0, 0x1f], 1, &mut output[..2], 1).unwrap();
    assert_eq!(0, short.read());
    assert_eq!(0, short.written());
    assert!(matches!(short.status(), TranscodeStatus::NeedOutput { required } if required.get() == 2));
    assert_eq!([0xa5; 4], output);
    let done = encoder.transcode(&[0, 0x1f], 1, &mut output[..3], 1).unwrap();
    assert_eq!(1, done.read());
    assert_eq!(2, done.written());
    assert_eq!(TranscodeStatus::Complete, done.status());
    assert_eq!([0xa5, b'1', b'f', 0xa5], output);
    assert_eq!(0, encoder.finish(&mut output, 3).unwrap());
}

#[test]
fn test_hex_adapter_decoder_preserves_incomplete_tail_and_eof() {
    let mut decoder = CodecTranscodeDecoder::new(HexByteCodec::new());
    let mut output = [0xa5; 3];
    assert_eq!(0, decoder.reset(&mut [], 0).unwrap());
    let short = decoder.transcode(b"!1", 1, &mut output, 1).unwrap();
    assert_eq!(0, short.read());
    assert_eq!(0, short.written());
    assert!(matches!(short.status(), TranscodeStatus::NeedInput { required } if required.get() == 2));
    assert_eq!([0xa5; 3], output);
    let done = decoder.transcode_eof(b"!1f", 1, &mut output, 1).unwrap();
    assert_eq!(2, done.read());
    assert_eq!(1, done.written());
    assert_eq!(TranscodeStatus::Complete, done.status());
    assert_eq!([0xa5, 0x1f, 0xa5], output);
    assert_eq!(0, decoder.finish(&mut output, 2).unwrap());

    decoder.reset(&mut [], 0).unwrap();
    let error = decoder.transcode_eof(b"!1", 1, &mut output, 1).unwrap_err();
    assert!(matches!(
        error,
        TranscodeDecodeError::Failure(TranscodeFailure::IncompleteInput {
            input_index: 1,
            required: 2,
            available: 1
        })
    ));
    assert_eq!([0xa5, 0x1f, 0xa5], output);
}

fn assert_percent_adapter_rejects_impossible_prefix<C>(codec: C)
where
    C: Codec<Value = u8, Unit = u8, DecodeError = MiscCodecError>,
{
    let mut decoder = CodecTranscodeDecoder::new(codec);
    let mut output = [0xa5; 3];
    decoder.reset(&mut [], 0).unwrap();
    let error = decoder.transcode(b"!%z", 1, &mut output, 1).unwrap_err();
    assert!(matches!(error.domain_ref(), Some(MiscCodecError::InvalidEscape { .. })));
    assert_eq!([0xa5; 3], output);

    decoder.reset(&mut [], 0).unwrap();
    let progress = decoder.transcode(b"!%A", 1, &mut output, 1).unwrap();
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
    assert!(matches!(progress.status(), TranscodeStatus::NeedInput { required } if required.get() == 3));

    let progress = decoder.transcode_eof(b"!%A0", 1, &mut output, 1).unwrap();
    assert_eq!(3, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([0xa5, 0xa0, 0xa5], output);
}

#[test]
fn test_percent_adapters_reject_impossible_prefix_without_refill() {
    assert_percent_adapter_rejects_impossible_prefix(PercentCodec::new());
    assert_percent_adapter_rejects_impossible_prefix(FormUrlencodedCodec::new());
}

#[test]
fn test_c_universal_adapter_rejects_invalid_short_prefix() {
    let mut decoder = CodecTranscodeDecoder::new(CStringLiteralCodec::new());
    let mut output = [0xa5; 3];
    decoder.reset(&mut [], 0).unwrap();

    let error = decoder.transcode(br"\uZ", 0, &mut output, 0).unwrap_err();

    assert!(matches!(
        error.domain_ref(),
        Some(MiscCodecError::InvalidDigit {
            radix: 16,
            index: 2,
            character: 'Z'
        })
    ));
    assert_eq!([0xa5; 3], output);
}

#[test]
fn test_safe_hex_adapter_readme() {
    let mut decoder = CodecTranscodeDecoder::new(HexByteCodec::new());
    let chunks: [&[u8]; 3] = [b"1", b"f8", b"b"];
    let mut pending = Vec::new();
    let mut output = [0_u8; 2];
    let mut written = 0;

    decoder.reset(&mut output, 0).expect("reset decoder");
    for (index, chunk) in chunks.iter().enumerate() {
        pending.extend_from_slice(chunk);
        let progress = decoder
            .transcode(&pending, 0, &mut output, written)
            .expect("decode available input");
        let read = progress.read();
        written += progress.written();
        pending.drain(..read);
        assert!(!matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
        if index < 2 {
            assert!(matches!(progress.status(), TranscodeStatus::NeedInput { .. }));
        } else {
            assert_eq!(TranscodeStatus::Complete, progress.status());
        }
    }

    // Complete says this call consumed its visible input; only the source tells us
    // it has ended.
    assert!(pending.is_empty());
    let eof = decoder
        .transcode_eof(&pending, 0, &mut output, written)
        .expect("finish decoding at confirmed EOF");
    assert_eq!(TranscodeStatus::Complete, eof.status());
    written += eof.written();
    assert_eq!([0x1f, 0x8b], output);
    assert_eq!(2, written);
    assert_eq!(0, decoder.finish(&mut output, written).expect("finish adapter"));

    output = [0xa5; 2];
    decoder.reset(&mut output, 0).expect("reset for short input");
    let error = decoder.transcode_eof(b"1", 0, &mut output, 0).unwrap_err();
    assert!(matches!(
        error,
        TranscodeDecodeError::Failure(TranscodeFailure::IncompleteInput {
            input_index: 0,
            required: 2,
            available: 1
        })
    ));
    assert_eq!([0xa5; 2], output);

    decoder.reset(&mut output, 0).expect("reset for invalid input");
    let error = decoder.transcode(b"1g", 0, &mut output, 0).unwrap_err();
    assert!(matches!(
        error.domain_ref(),
        Some(qubit_codec_misc::MiscCodecError::InvalidDigit { index: 1, .. })
    ));
    assert_eq!([0xa5; 2], output);
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
