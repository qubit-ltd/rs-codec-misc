use qubit_codec_misc::Base64Codec;
use qubit_codec_misc::Base64Encoder;

#[test]
fn test_base64_encoder_constructors_are_independent_of_codec_lifetime() {
    let encoder = {
        let codec = Base64Codec::url_safe_no_pad();
        codec.encoder()
    };
    let constructed = [
        Base64Encoder::standard(),
        Base64Encoder::standard_no_pad(),
        Base64Encoder::url_safe(),
        Base64Encoder::url_safe_no_pad(),
        Base64Encoder::default(),
        encoder,
    ];
    assert_eq!(constructed.len(), 6);
}
