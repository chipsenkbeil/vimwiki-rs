# vimwiki macros

Contains macros to generate vimwiki components in Rust at compile time.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
vimwiki_macros = "0.1.0-alpha.5"
```

## Examples

```rust
use vimwiki_macros::*;

// Parse vimwiki language at compile-time and generate Rust-based elements
let page = vimwiki_page! {r#"
= Header =

Some paragraph with *bold* content
and some [[links]].

- List item 1
- List item 2
"#};
```

## License

This project is licensed under either of

Apache License, Version 2.0, (LICENSE-APACHE or
[apache-license][apache-license]) MIT license (LICENSE-MIT or
[mit-license][mit-license]) at your option.

[apache-license]: http://www.apache.org/licenses/LICENSE-2.0
[mit-license]: http://opensource.org/licenses/MIT
