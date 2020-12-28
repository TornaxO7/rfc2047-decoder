use rfc2047_decoder;

fn main() -> rfc2047_decoder::Result<()> {
    let encoded_str = "=?UTF-8?Q?encoded_str_with_symbol_=E2=82=AC?=";
    let decoded_str = "encoded str with symbol €";

    assert_eq!(rfc2047_decoder::decode(encoded_str)?, decoded_str);
    Ok(())
}
