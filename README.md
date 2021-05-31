# vimwiki-rs: Rust libraries and tooling for vimwiki

[![CI](https://github.com/chipsenkbeil/vimwiki-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/chipsenkbeil/vimwiki-rs/actions/workflows/ci.yml)

Welcome to the primary repository for all things Rust + vimwiki! This
repository houses several Rust crates alongside binaries like `vimwiki-server`
that enable parsing, querying, modifying, and generating vimwiki content.

## [vimwiki][vimwiki_readme] [![Crates.io][vimwiki_crates_img]][vimwiki_crates_lnk] [![Docs.rs][vimwiki_doc_img]][vimwiki_doc_lnk]

[vimwiki_readme]: ./vimwiki/README.md
[vimwiki_crates_img]: https://img.shields.io/crates/v/vimwiki.svg
[vimwiki_crates_lnk]: https://crates.io/crates/vimwiki
[vimwiki_doc_img]: https://docs.rs/vimwiki/badge.svg
[vimwiki_doc_lnk]: https://docs.rs/vimwiki

Represents the language definition and parsing support for the vimwiki language.

## [vimwiki-cli][vimwiki_cli_readme] [![Crates.io][vimwiki_cli_crates_img]][vimwiki_cli_crates_lnk] [![Docs.rs][vimwiki_cli_doc_img]][vimwiki_cli_doc_lnk]

[vimwiki_cli_readme]: ./vimwiki-cli/README.md
[vimwiki_cli_crates_img]: https://img.shields.io/crates/v/vimwiki-cli.svg
[vimwiki_cli_crates_lnk]: https://crates.io/crates/vimwiki-cli
[vimwiki_cli_doc_img]: https://docs.rs/vimwiki-cli/badge.svg
[vimwiki_cli_doc_lnk]: https://docs.rs/vimwiki-cli

Provides tiny command-line interface on top of the vimwiki parser and HTML
output functionality of the vimwiki library.

## [vimwiki-core][vimwiki_core_readme] [![Crates.io][vimwiki_core_crates_img]][vimwiki_core_crates_lnk] [![Docs.rs][vimwiki_core_doc_img]][vimwiki_core_doc_lnk]

[vimwiki_core_readme]: ./vimwiki-core/README.md
[vimwiki_core_crates_img]: https://img.shields.io/crates/v/vimwiki-core.svg
[vimwiki_core_crates_lnk]: https://crates.io/crates/vimwiki-core
[vimwiki_core_doc_img]: https://docs.rs/vimwiki-core/badge.svg
[vimwiki_core_doc_lnk]: https://docs.rs/vimwiki-core

Provides the core vimwiki elements, parsing, and other features that are
exposed through the primary vimwiki crate.

## [vimwiki_macros][vimwiki_macros_readme] [![Crates.io][vimwiki_macros_crates_img]][vimwiki_macros_crates_lnk] [![Docs.rs][vimwiki_macros_doc_img]][vimwiki_macros_doc_lnk]

[vimwiki_macros_readme]: ./vimwiki-macros/README.md
[vimwiki_macros_crates_img]: https://img.shields.io/crates/v/vimwiki_macros.svg
[vimwiki_macros_crates_lnk]: https://crates.io/crates/vimwiki_macros
[vimwiki_macros_doc_img]: https://docs.rs/vimwiki_macros/badge.svg
[vimwiki_macros_doc_lnk]: https://docs.rs/vimwiki_macros

Contains macros to generate vimwiki components in Rust at compile time.

## [vimwiki-server][vimwiki_server_readme] [![Crates.io][vimwiki_server_crates_img]][vimwiki_server_crates_lnk] [![Docs.rs][vimwiki_server_doc_img]][vimwiki_server_doc_lnk]

[vimwiki_server_readme]: ./vimwiki-server/README.md
[vimwiki_server_crates_img]: https://img.shields.io/crates/v/vimwiki-server.svg
[vimwiki_server_crates_lnk]: https://crates.io/crates/vimwiki-server
[vimwiki_server_doc_img]: https://docs.rs/vimwiki-server/badge.svg
[vimwiki_server_doc_lnk]: https://docs.rs/vimwiki-server

Provides graphql server to inspect and manipulate vimwiki files.

## [vimwiki-wasm][vimwiki_wasm_readme] [![Crates.io][vimwiki_wasm_crates_img]][vimwiki_wasm_crates_lnk] [![Docs.rs][vimwiki_wasm_doc_img]][vimwiki_wasm_doc_lnk]

[vimwiki_wasm_readme]: ./vimwiki-wasm/README.md
[vimwiki_wasm_crates_img]: https://img.shields.io/crates/v/vimwiki-wasm.svg
[vimwiki_wasm_crates_lnk]: https://crates.io/crates/vimwiki-wasm
[vimwiki_wasm_doc_img]: https://docs.rs/vimwiki-wasm/badge.svg
[vimwiki_wasm_doc_lnk]: https://docs.rs/vimwiki-wasm

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
