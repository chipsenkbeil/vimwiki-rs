# vimwiki-rs: Rust libraries and tooling for vimwiki

![CI](https://github.com/chipsenkbeil/vimwiki-rs/workflows/CI/badge.svg)

Welcome to the primary repository for all things Rust + vimwiki! This
repository houses several Rust crates alongside binaries like `vimwiki-server`
that enable parsing, querying, modifying, and generating vimwiki content.

## vimwiki [![Latest Version](https://img.shields.io/crates/v/vimwiki.svg)](https://crates.io/crates/vimwiki)

[Documentation](https://docs.rs/vimwiki)

Represents the language definition and parsing support for the vimwiki language.

## vimwiki_macros [![Latest Version](https://img.shields.io/crates/v/vimwiki_macros.svg)](https://crates.io/crates/vimwiki_macros)

[Documentation](https://docs.rs/vimwiki_macros)

Contains macros to generate vimwiki components in Rust at compile time.

## vimwiki-server [![Latest Version](https://img.shields.io/crates/v/vimwiki-server.svg)](https://crates.io/crates/vimwiki-server)

[Documentation](https://docs.rs/vimwiki-server)

Provides graphql server to inspect and manipulate vimwiki files.

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
