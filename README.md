# vimwiki-rs: Rust libraries and tooling for vimwiki

[![CI](https://github.com/chipsenkbeil/vimwiki-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/chipsenkbeil/vimwiki-rs/actions/workflows/ci.yml)

Welcome to the primary repository for all things Rust + vimwiki! This
repository houses several Rust crates alongside binaries like `vimwiki-server`
that enable parsing, querying, modifying, and generating vimwiki content.

## [vimwiki](./vimwiki/README.md) [![Latest Version](https://img.shields.io/crates/v/vimwiki.svg)](https://crates.io/crates/vimwiki)

Represents the language definition and parsing support for the vimwiki language.

## [vimwiki-cli](./vimwiki-cli/README.md) [![Latest Version](https://img.shields.io/crates/v/vimwiki-cli.svg)](https://crates.io/crates/vimwiki-cli)

Provides tiny command-line interface on top of the vimwiki parser and HTML
output functionality of the vimwiki library.

## [vimwiki_macros](./vimwiki_macros/README.md) [![Latest Version](https://img.shields.io/crates/v/vimwiki_macros.svg)](https://crates.io/crates/vimwiki_macros)

Contains macros to generate vimwiki components in Rust at compile time.

## [vimwiki-server](./vimwiki-server/README.md) [![Latest Version](https://img.shields.io/crates/v/vimwiki-server.svg)](https://crates.io/crates/vimwiki-server)

Provides graphql server to inspect and manipulate vimwiki files.

## [vimwiki-wasm](./vimwiki-wasm/README.md) [![Latest Version](https://img.shields.io/crates/v/vimwiki-wasm.svg)](https://crates.io/crates/vimwiki-wasm)

Provides a Web Assembly (wasm) binding to the vimwiki library, enabling parsing
of vimwiki text within a browser (or NodeJS) and outputting in HTML.

# Sister Projects

Alongside this repository are several other projects

## vimwiki

[Link to project](https://github.com/vimwiki/vimwiki)

Main project and plugin for vim dedicated to the vimwiki language. This is
what started it all and is your main source for vim functionality and support
for the language.

## vimwiki-server.nvim

[Link to project](https://github.com/chipsenkbeil/vimwiki-server.nvim)

Represents a sister project that offers enhanced and alternative functionality
in the neovim editor for the vimwiki language. This project uses
`vimwiki-server` to power its functionality in combination with the neovim Lua
engine to provide all of its vimwiki goodness.
