# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

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
