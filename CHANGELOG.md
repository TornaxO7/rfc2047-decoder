# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

* Fixed decoding strings bigger than 76 chars [#15]

## [0.1.3] - 2022-10-10

### Fixed

* Max length of encoded words [#1]
* Manage tokens special chars [#3]

### Changed

* Refactored parser using chumsky [#7]

## [0.1.2] - 2020-12-30

### Fixed

* Multiple encoded words separator

## [0.1.1] - 2020-12-30

### Added

* Added evaluator with AST

### Changed

* Decoded fn accepts now `&[u8]` instead of `&str`

### Fixed

* Removed space between encoded words [#2]

## [0.1.0] - 2020-12-28

First official release.

[unreleased]: https://github.com/soywod/rfc2047-decoder/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/soywod/rfc2047-decoder/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/soywod/rfc2047-decoder/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/soywod/rfc2047-decoder/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/soywod/rfc2047-decoder/releases/tag/v0.1.0

[#1]: https://github.com/soywod/rfc2047-decoder/issues/1
[#2]: https://github.com/soywod/rfc2047-decoder/issues/2
[#3]: https://github.com/soywod/rfc2047-decoder/issues/3
[#7]: https://github.com/soywod/rfc2047-decoder/issues/7
[#15]: https://github.com/soywod/rfc2047-decoder/issues/15
