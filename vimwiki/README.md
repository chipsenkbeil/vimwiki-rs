# vimwiki

This crate represents the language definition and parsing support for
the vimwiki language. This has been broken out to be a shareable crate for
others to build on top of the vimwiki language and write their own tooling.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
vimwiki = "0.1.0-alpha.2"
```

## Examples

```rust
use vimwiki::{RawStr, LE, elements::*};

// Load a file into a String
let input = std::fs::read_to_string("/path/to/file.wiki").unwrap();

// Parse the input as a LocatedElement of Page
let page: LE<Page> = RawStr::Vimwiki(&input).try_into().unwrap();
```

## License

This project is licensed under either of

Apache License, Version 2.0, (LICENSE-APACHE or
[apache-license][apache-license]) MIT license (LICENSE-MIT or
[mit-license][mit-license]) at your option.

[apache-license]: http://www.apache.org/licenses/LICENSE-2.0
[mit-license]: http://opensource.org/licenses/MIT
