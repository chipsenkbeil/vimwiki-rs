# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

- `vimwiki-cli` now includes a **format** subcommand to format vimwiki text
- `vimwiki-core` now supports converting an ast into vimwiki text

### Changed

- Moved `iter::*` to root level of `vimwiki-core` crate
- `ListItemContents` now contains a `Vec<BlockElement>` and the associated
  parser now supports other types such as `CodeBlock`, `MathBlock`,
  `Blockquote`, and `Table` as options for being included
- List items no longer keep raw text, but instead paragraphs of text as one
  of the potential elements
- HTML output of list items with text now yields `<li><p>...</p></li>` instead
  of the previous `<li>...</li>`

### Fixed

- Local anchor links were adding `index.html` in front of the anchor
  regardless of the page's name
- Bump to `0.3.0` of `vimvar` dependency to support `init.lua` when searching
  for wiki paths

### Removed

- Deleted `vimwiki-server` as no longer going to support a GraphQL server of
  documents since `AsyncGraphql` dependency keeps breaking. Instead, will
  focus on expanding the `vimwiki-cli` offering to provide features to
  query wiki documents

### Performance

- Refactored text parser to yield a 5x speedup on local testing of wikis that
  previously took ~30s now finishing in ~6s for parsing and output

## [0.1.0] - 2021-06-06

### Added

- `vimwiki-cli` crate that exposes a cli interface to convert vimwiki to html
  and inspect vimwiki using **jsonpath** queries
- `vimwiki-wasm` crate that exposes a wasm version of `vimwiki` to enable
  parsing and manipulation in the browser
- `vimvar` dependency to support loading **g:vimwiki_list** directly from
  neovim/vim config files, used both by `vimwiki-cli` and `vimwiki-server`
- `--quiet` option added to `vimwiki-server` to support only showing
  error-level log messages
- `vimwiki` now supports optional feature **html** to render vimwiki as html

### Changed

- `vimwiki` crate renamed to `vimwiki-core` in order to provide a `vimwiki`
  crate that contains both core and macros to simplify usage
- `vimwiki-server` now supports colored output of logging and defaults to
  info-level logging
- Move `vimwiki-cli` logic into *lib.rs* that is imported within *main.rs*
- Transclusion links now use spaces between properties instead of pipe
  symbols (`{{img.png|desc|prop1="value" prop2="value"}}`)

### Fixed

- Raw links weren't captured if preceded by text on a line ([#119](https://github.com/chipsenkbeil/vimwiki-rs/issues/119))

## [0.1.0-alpha.6] - 2021-05-28

### Added

- This `CHANGELOG.md` file to keep track of future changes
- `scripts/release.sh` to keep track of all version changes and update multiple
  `Cargo.toml` as well as other files like this changelog
- `vimwiki_<ELEMENT>_format` and `vimwiki_<ELEMENT>_raw_format` support into
  the `vimwiki_macros` crate to support injecting content into vimwiki macros
  at compile-time
  ([#102](https://github.com/chipsenkbeil/vimwiki-rs/issues/102))

### Changed

- `vimwiki_macros` is now more hygienic through proper testing
- `vimwiki_macros` now uses `syn` for some parsing and `proc-macro-crate`
  to detect and get the root of the `vimwiki` crate
  ([#92](https://github.com/chipsenkbeil/vimwiki-rs/issues/92))
- `vimwiki` now exports all items at the top level including items
  found under the `elements` module

### Bugs

- `vimwiki_list_format!(...)` and `vimwiki_definition_list_format!(...)` and
  their raw counterparts do not support `{}` injection without explicit number
  or names as the generated code may not maintain the same order of list items.
  It is recommended to use `{0}` or `{name}` instead when working with those
  two types
