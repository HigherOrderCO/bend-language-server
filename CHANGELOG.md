# Changelog

All notable changes to the `bend-language-server` Rust project will be documented in this file.
To see changes related to the VSCode extension, see [editors/code/CHANGELOG.md](./editors/code/CHANGELOG.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and both this changelog and the crate's versioning scheme follow what the
[Bend repository](https://github.com/HigherOrderCO/Bend) is using at the moment.

## [Unreleased]

All current features are yet unreleased on a stable version, and are only available in the `0.2.37` alpha versions.

## [0.2.37-alpha.4] - 2024-09-02

### Fixed

- Multi-byte characters highlighting other characters

## [0.2.37-alpha.3] - 2024-08-30

### Added

- Multi-line (block) comments

### Fixed

- Single-line comments

## [0.2.37-alpha.2] - 2024-08-27

### Added

- `--version` command to executable

## [0.2.37-alpha.1] - 2024-08-23

First release!

### Added

- Semantic token highlighting through tree-sitter
- Diagnostic reporting

<!-- still haven't added a release to GitHub -->
[0.2.37-alpha.3]: https://github.com/HigherOrderCO/bend-language-server/
[0.2.37-alpha.2]: https://github.com/HigherOrderCO/bend-language-server/
[0.2.37-alpha.1]: https://github.com/HigherOrderCO/bend-language-server/
[Unreleased]: https://github.com/HigherOrderCO/bend-language-server/

# Collaborators

This project got inspiration from [Pedro Braga](https://github.com/mrpedrobraga)'s initial language server implementation and [Rohan Vashisht](https://github.com/RohanVashisht1234)'s VSCode extension. Thank you for your collaboration!
