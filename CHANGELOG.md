# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.3.0] - Mar 10, 2025

### Added

- Allow ignoring attribute errors while saving/importing bookmarks (longer
    titles will be trimmed, some invalid tags will be corrected)
- Allow copying URI(s) to the system clipboard via the TUI

### Fixed

- Listing bookmarks by tag(s) shows all tags for each bookmark returned

## [v0.2.0] - Feb 27, 2025

### Changed

- Allow searching over multiple terms
- Respect XDG_DATA_HOME on MacOS as well, if set

## [v0.1.0] - Feb 20, 2025

### Added

- Initial release

[unreleased]: https://github.com/dhth/bmm/compare/v0.3.0...HEAD
[v0.3.0]: https://github.com/dhth/bmm/compare/v0.2.0...v0.3.0
[v0.2.0]: https://github.com/dhth/bmm/compare/v0.1.0...v0.2.0
[v0.1.0]: https://github.com/dhth/bmm/commits/v0.1.0/
