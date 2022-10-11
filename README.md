# rfc2047-decoder [![Crates.io](https://img.shields.io/crates/v/rfc2047-decoder?style=flat-square)](https://crates.io/crates/rfc2047-decoder) [![Crates.io](https://img.shields.io/crates/d/rfc2047-decoder?style=flat-square)](https://crates.io/crates/rfc2047-decoder)

Rust library for decoding [RFC 2047 MIME Message
Headers](https://tools.ietf.org/html/rfc2047).

```rust
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
            .skip_encoded_word_length(true)
            .decode(encoded_str.as_bytes())
            .unwrap(),
        decoded_str
    );
}
```
