# json-parse
[![build-img]][build-url]
[![docs-img]][docs-url]
[![crates-img]][crates-url]
---

A low-level JSON parser for Rust with a simple API and full spec support.

```rust
use json_parse::{parse, JsonElement::*};

let json = "[1, true, null]";
let parsed = parse(json).unwrap();

assert_eq!(parsed, Array(
    vec![Number(1.0), Boolean(true), Null]
));
```

Consider using this library if:
- You need a lightweight parser with no external dependencies.
- You want nice user-facing error messages and the ability to pinpoint exactly where a parsing error happened.
- You need to maintain the relative order of the keys in a JSON object (for example, to build a JSON formatter).

This library may not be a good fit if:
- You intend to use it to serialize and deserialize your own data (use [serde](https://crates.io/crates/serde) instead).
- You want utilities and sugar to navigate the contents of a JSON (use [json](https://docs.rs/json/latest/json/) instead).

[build-img]: https://img.shields.io/github/actions/workflow/status/agubelu/json-parse/run_tests.yml?branch=master
[build-url]: https://github.com/agubelu/json-parse/actions?query=branch%3Amaster

[docs-img]: https://img.shields.io/crates/v/json-parse.svg
[docs-url]: https://crates.io/crates/json-parse

[crates-img]: https://img.shields.io/docsrs/json-parse
[crates-url]: https://docs.rs/json-parse/latest