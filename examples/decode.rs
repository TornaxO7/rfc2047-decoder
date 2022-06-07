use rfc2047_decoder;

fn main() {
    let encoded_str = "=?UTF-8?Q?str?=";
    let decoded_str = "str";

    assert_eq!(
        rfc2047_decoder::decode(encoded_str.as_bytes()).unwrap(),
        decoded_str
    );
}
