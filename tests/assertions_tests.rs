pub(crate) fn invalid_source(
    failure: qubit_codec::DecodeFailure<qubit_codec_misc::MiscCodecError>,
) -> qubit_codec_misc::MiscCodecError {
    match failure {
        qubit_codec::DecodeFailure::Invalid { source, .. } => source,
        qubit_codec::DecodeFailure::Incomplete { .. } => {
            panic!("expected invalid misc codec decode failure")
        }
    }
}

pub(crate) fn incomplete_required(
    failure: qubit_codec::DecodeFailure<qubit_codec_misc::MiscCodecError>,
) -> usize {
    match failure {
        qubit_codec::DecodeFailure::Incomplete { required_total } => {
            required_total.get()
        }
        qubit_codec::DecodeFailure::Invalid { .. } => {
            panic!("expected incomplete misc codec decode failure")
        }
    }
}
