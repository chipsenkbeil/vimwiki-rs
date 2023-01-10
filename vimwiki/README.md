# vimwiki

This crate represents the language definition and parsing support for
the vimwiki language. This has been broken out to be a shareable crate for
others to build on top of the vimwiki language and write their own tooling.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
vimwiki = "0.1"
```

## Examples

```rust
use vimwiki::{Language, elements::*};

// Load some language as a string
let language = Language::from_vimwiki_str(r#"
= My Header =
///
Some paragraph with *decorations* and [[links]] that you would normally
see in a vimwiki file.
"#);

// Parse the input as a page using vimwiki format
let page: Page = language.parse().unwrap();
```

## Features

By default, no features are enable, but the following are offered:

- **html**: If specified, builds in support to convert vimwiki elements to
  html.
- **macros**: If specified, pulls in `vimwiki_macros` to expose top-level macro
  functions to parse and produce vimwiki elements at compile-time.
- **timekeeper**: If specified, all parser logic runs through a
  statically-allocated `HashMap` that logs the time taken to parse various
  elements and can print out results in a human-readable format. This is
  predominately useful for performance optimizations internally.

## License

This project is licensed under either of

Apache License, Version 2.0, (LICENSE-APACHE or
[apache-license][apache-license]) MIT license (LICENSE-MIT or
[mit-license][mit-license]) at your option.

[apache-license]: http://www.apache.org/licenses/LICENSE-2.0
[mit-license]: http://opensource.org/licenses/MIT
