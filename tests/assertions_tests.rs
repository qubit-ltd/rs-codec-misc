pub(crate) fn invalid_source(
    failure: qubit_codec::CodecDecodeFailure<qubit_codec_misc::MiscCodecError>,
) -> qubit_codec_misc::MiscCodecError {
    match failure {
        qubit_codec::CodecDecodeFailure::Invalid { source, .. } => source,
        qubit_codec::CodecDecodeFailure::Incomplete { .. } => {
            panic!("expected invalid misc codec decode failure")
        }
    }
}

pub(crate) fn incomplete_required(
    failure: qubit_codec::CodecDecodeFailure<qubit_codec_misc::MiscCodecError>,
) -> usize {
    match failure {
        qubit_codec::CodecDecodeFailure::Incomplete { required_total } => required_total.get(),
        qubit_codec::CodecDecodeFailure::Invalid { .. } => {
            panic!("expected incomplete misc codec decode failure")
        }
    }
}
