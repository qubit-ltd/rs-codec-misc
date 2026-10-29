use qubit_codec_misc::Base64Codec;
use qubit_codec_misc::Base64Decoder;

#[test]
fn test_base64_decoder_constructors_are_independent_of_codec_lifetime() {
    let decoder = {
        let codec = Base64Codec::url_safe_no_pad();
        codec.decoder()
    };
    let constructed = [
        Base64Decoder::standard(),
        Base64Decoder::standard_no_pad(),
        Base64Decoder::url_safe(),
        Base64Decoder::url_safe_no_pad(),
        Base64Decoder::default(),
        decoder,
    ];
    assert_eq!(constructed.len(), 6);
}
