# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- ...
### Changed
- ...
### Deprecated
- ...
### Removed
- ...
### Fixed
- ...
### Security:
- ...


## [0.1.1] - 2020-02-07
### Added
- `SinkMock` that behave more like real sink and cover a better use case (compare to `SinkFeedback`).
### Removed
- `pin_project` dependency


## [0.1.0] - 2020-02-07
### Added
- Init with [cargo-generate](https://github.com/ashleygwilliams/cargo-generate) with [template](https://github.com/xoac/crates-io-lib-template)
- `FuseLast` container for Iterators with `IteratorsExt` trait.
- `SinkFeedback` that can be created with `from_iter()` function.
- `drain()` and `interleave_pending()` creators.

[Unreleased]: https://github.com/xoac//futures-test-sink/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/xoac/futures-test-sink/releases/tag/v0.1.1
[0.1.0]: https://github.com/xoac/futures-test-sink/releases/tag/v0.1.0


