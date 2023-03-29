# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2023-03-29

### Changed

- Bumped `base64` to `v0.21.0`.
- Bumped `chumsky` to `v0.9.2`.

## [0.2.1] - 2023-01-08

### Fixed

- Fixed discarded errors [#20].

## [0.2.0] - 2022-10-11

### Added

- Added Nix support
- Allowed decoding strings bigger than 76 chars [#15]

### Changed

- Renamed error variants to match the [Rust API
  guidelines](https://rust-lang.github.io/api-guidelines/naming.html#names-use-a-consistent-word-order-c-word-order):
  - `lexer::Error::EncodingIssue` becomes `ParseBytesError`
  - `lexer::Error::EncodedWordTooLong` becomes
    `ParseEncodedWordTooLongError`
  - `parser::Error::UnknownCharset` becomes `ParseEncodingError`
  - `parser::Error::UnknownCharset` has been removed (unused)
  - `parser::Error::UnknownEncoding` becomes `ParseEncodingError`
  - `parser::Error::EncodedWordTooBig` becomes
    `ParseEncodingTooBigError`
  - `parser::Error::EmptyEncoding` becomes `ParseEncodingEmptyError`
  - `evaluator::Error::DecodeUtf8` becomes `DecodeUtf8Error`
  - `evaluator::Error::DecodeBase64` becomes `DecodeBase64Error`
  - `evaluator::Error::DecodeQuotedPrintable` becomes
    `DecodeQuotedPrintableError`

## [0.1.3] - 2022-10-10

### Fixed

- Max length of encoded words [#1]
- Manage tokens special chars [#3]

### Changed

- Refactored parser using chumsky [#7]

## [0.1.2] - 2020-12-30

### Fixed

- Multiple encoded words separator

## [0.1.1] - 2020-12-30

### Added

- Added evaluator with AST

### Changed

- Decoded fn accepts now `&[u8]` instead of `&str`

### Fixed

- Removed space between encoded words [#2]

## [0.1.0] - 2020-12-28

First official release.

[unreleased]: https://github.com/soywod/rfc2047-decoder/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/soywod/rfc2047-decoder/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/soywod/rfc2047-decoder/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/soywod/rfc2047-decoder/compare/v0.1.3...v0.2.0
[0.1.3]: https://github.com/soywod/rfc2047-decoder/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/soywod/rfc2047-decoder/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/soywod/rfc2047-decoder/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/soywod/rfc2047-decoder/releases/tag/v0.1.0

[#1]: https://github.com/soywod/rfc2047-decoder/issues/1
[#2]: https://github.com/soywod/rfc2047-decoder/issues/2
[#3]: https://github.com/soywod/rfc2047-decoder/issues/3
[#7]: https://github.com/soywod/rfc2047-decoder/issues/7
[#15]: https://github.com/soywod/rfc2047-decoder/issues/15
[#20]: https://github.com/soywod/rfc2047-decoder/issues/20
