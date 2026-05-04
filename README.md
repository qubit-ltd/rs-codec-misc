# qubit-codec

Reusable byte and text codecs for Rust applications.

This crate focuses on stable textual encodings that are useful across Qubit
Rust crates:

- hexadecimal bytes
- Base64 bytes
- percent-encoded UTF-8 text
- `application/x-www-form-urlencoded` UTF-8 text fragments

It intentionally does not replace Rust's `Display`, `FromStr`, `TryFrom`, or
`serde` APIs for ordinary object conversion.

```rust
use qubit_codec::{
    Base64Codec,
    HexCodec,
    PercentCodec,
};

let hex = HexCodec::upper()
    .with_prefix("0x")
    .with_separator(" ")
    .encode(&[0x1f, 0x8b]);
assert_eq!("0x1F 8B", hex);

let bytes = Base64Codec::standard()
    .decode("aGVsbG8=")
    .expect("valid Base64 should decode");
assert_eq!(b"hello".to_vec(), bytes);

let text = PercentCodec::new().encode("a b/中");
assert_eq!("a%20b%2F%E4%B8%AD", text);
```
