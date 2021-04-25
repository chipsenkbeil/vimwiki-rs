# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

- This `CHANGELOG.md` file to keep track of future changes
- `scripts/release.sh` to keep track of all version changes and update multiple
  `Cargo.toml` as well as other files like this changelog
- `vimwiki_<ELEMENT>_format` and `vimwiki_<ELEMENT>_raw_format` support into
  the `vimwiki_macros` crate to support injecting content into vimwiki macros
  at compile-time [#102](https://github.com/chipsenkbeil/vimwiki-rs/issues/102)

### Changed

- `entity_macros` is now more hygienic through proper testing
- `entity_macros` now uses `syn` for some parsing and `proc-macro-crate`
  to detect and get the root of the `vimwiki` crate
