# rfc2047-decoder [![Crates.io](https://img.shields.io/crates/v/rfc2047-decoder)](https://crates.io/crates/rfc2047-decoder) [![Crates.io](https://img.shields.io/crates/d/rfc2047-decoder)](https://crates.io/crates/rfc2047-decoder)

Simple [RFC 2047 MIME Message Header](https://tools.ietf.org/html/rfc2047)
decoder library for [Rust](https://www.rust-lang.org/).

```rust
use rfc2047_decoder;

fn main() -> rfc2047_decoder::Result<()> {
    let encoded_str = "=?UTF-8?Q?str?=";
    let decoded_str = "str";

    assert_eq!(
        rfc2047_decoder::decode(encoded_str.as_bytes())?,
        decoded_str
    );

    Ok(())
}
```
