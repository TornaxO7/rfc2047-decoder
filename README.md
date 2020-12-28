# rfc2047-decoder

Simple [Rust](https://www.rust-lang.org/) library that decodes [RFC 2047 MIME
Message Headers](https://tools.ietf.org/html/rfc2047).

## Installation

Add `rfc2047-decoder` to your `Cargo.toml`:

```toml
rfc2047-decoder = "0.1.0"
```

## Usage

```rust
use rfc2047_decoder;

fn main() -> rfc2047_decoder::Result<()> {
    let encoded_str = "=?UTF-8?Q?encoded_str_with_symbol_=E2=82=AC?=";
    let decoded_str = "encoded str with symbol €";

    assert_eq!(rfc2047_decoder::decode(encoded_str)?, decoded_str);
    Ok(())
}
```
