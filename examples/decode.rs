use rfc2047_decoder;

fn main() {
    let encoded_str = "=?UTF-8?Q?str?=";
    let decoded_str = "str";

    // using the decode helper (default options)
    assert_eq!(
        rfc2047_decoder::decode(encoded_str.as_bytes()).unwrap(),
        decoded_str
    );

    // using the decoder builder (custom options)
    assert_eq!(
        rfc2047_decoder::Decoder::new()
            .too_long_encoded_word_strategy(rfc2047_decoder::RecoverStrategy::Skip)
            .decode(encoded_str.as_bytes())
            .unwrap(),
        decoded_str
    );
}
