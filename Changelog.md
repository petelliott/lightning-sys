# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2020-08-22
### Added
- Introduced `Changelog.md` (#48)
- Set up CI builds for multiple major architectures and OSes (#15, #18, #27)
- Introduced basic code coverage metrics with Tarpaulin and Codecov (#24)
- Ported remaining examples from GNU lightning (#28, #31)
- Laid groundwork for interface generation mechanization (#32)
- Added some missing entry points (#34, #35, #37, #42)

### Changed
- Allow multiple `JitState`s to exist at once (#14)
- Use GNU Lightning 2.1.3 (#19)
- Switched back to static linking for liblightning (#23)
- Set a Minimum Supported Rust Version (MSRV) of 1.39.0 (#41)
- Switched back to a Git submodule for GNU lightning (#45)
- Made various build and code quality improvements (#10, #11, #17, #20, #25, #26, #36, #40, #46)

### Fixed
- Corrected a register reference in Fibonacci example (#16)
- Corrected lifetimes and made API semantically mutating (#30)

## [0.2.1] - 2020-05-08
### Added
- Added assertions to factorial example

### Changed
- Corrected link to GitHub repo

## [0.2.0] - 2019-08-17
### Added
- Added support for f32 and f64 entry points
- Added predicate functions `forward_p`, `indirect_p`, `target_p`
- Added the tail-call-optimized factorial example from GNU lightning

### Changed
- Relicensed under LGPL (from GPL), matching GNU lightning

## [0.1.2] - 2019-08-13
### Added
- Added Fibonacci example from GNU lightning
- Added branch/jump instructions
- Added (`panic`-ing) bounds checks to guard against invalid register indices
- Introduced aliases for entry points that redirect to other entry points

### Fixed
- Fixed some bugs uncovered by Fibonacci example

## [0.1.1] - 2019-08-12
Early preview release

## [0.1.0] - 2019-07-21
Initial (non-functional) release

[Unreleased]: https://github.com/petelliott/lightning-sys/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/petelliott/lightning-sys/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/petelliott/lightning-sys/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/petelliott/lightning-sys/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/petelliott/lightning-sys/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/petelliott/lightning-sys/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/petelliott/lightning-sys/releases/tag/v0.1.0
