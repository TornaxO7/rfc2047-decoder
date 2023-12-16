# rfc2047-decoder [![Crates.io](https://img.shields.io/crates/v/rfc2047-decoder?style=flat-square)](https://crates.io/crates/rfc2047-decoder) [![Crates.io](https://img.shields.io/crates/d/rfc2047-decoder?style=flat-square)](https://crates.io/crates/rfc2047-decoder)

# State
This project is considered as finished, only bugs will be fixed so don't wonder, if the last commit is
a long time ago.

# Introduction

Rust library for decoding [RFC 2047 MIME Message Headers](https://tools.ietf.org/html/rfc2047).

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
            .too_long_encoded_word_strategy(rfc2047_decoder::RecoverStrategy::Skip)
            .decode(encoded_str.as_bytes())
            .unwrap(),
        decoded_str
    );
}
```

## Sponsoring

[![github](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors&style=flat-square)](https://github.com/sponsors/soywod)
[![paypal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff&style=flat-square)](https://www.paypal.com/paypalme/soywod)
[![ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff&style=flat-square)](https://ko-fi.com/soywod)
[![buy-me-a-coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000&style=flat-square)](https://www.buymeacoffee.com/soywod)
[![liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222&style=flat-square)](https://liberapay.com/soywod)
