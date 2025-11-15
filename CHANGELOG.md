# Changelog

All notable changes to Godot Asset Browser will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Release Preparation (Phase 7.2)
- Auto-update mechanism (version_checker module)
  - Automatic version checking against GitHub releases
  - Semantic version parsing and comparison
  - Platform-specific download URL selection
  - Configurable check interval (default 24 hours)
  - Update notifications with release notes
  - 7 comprehensive tests
- CI/CD pipeline with GitHub Actions
  - Automated testing on Linux, Windows, and macOS
  - Code coverage tracking with Codecov
  - Security audits with cargo-audit
  - Performance benchmarks tracking
  - Automated release builds for all platforms
  - Weekly dependency updates with Dependabot
- GitHub community files
  - Issue templates (bug reports, feature requests)
  - Pull request template with comprehensive checklist
  - Contributing guidelines (450+ lines)
  - Code owners file for automated review assignments
  - Asset Library submission guide
- Release infrastructure
  - CHANGELOG.md with Keep a Changelog format
  - Release notes template
  - Multi-platform build workflows
  - Automated artifact packaging

### Performance (Phase 7.1)
- Search index with pre-computed lowercase strings (6x faster search)
- Lazy loading with pagination (40x faster page loads)
- Zero-copy memory access methods (90-100% memory reduction)
  - `with_assets()` - Zero-copy closure access
  - `get_asset_ids()` - Minimal-memory ID access
  - `has_asset()` - Zero-allocation existence checks
  - `count_assets_by_category()` - Zero-allocation counting
- Preview image caching system
  - Persistent cache with LRU eviction
  - Concurrent preloading (4 concurrent downloads)
  - 100MB default cache size
- Performance test suite (22 tests)
- Benchmark infrastructure with Criterion

#### Testing & Documentation (Phase 6)
- Comprehensive test suite (216 tests passing)
  - 87 core functionality tests
  - 22 performance optimization tests
  - 6 image cache tests
  - 7 version checker tests
  - 94 other module tests
- User guide (USER_GUIDE.md)
- Cross-platform guide (CROSS_PLATFORM.md)
- Testing guide (TESTING_GUI.md)
- Build release guide (BUILD_RELEASE.md)
- Performance optimization guide (PERFORMANCE_OPTIMIZATION.md)

## [0.1.0] - TBD

### Added
- Initial release
- Core asset management functionality
- Asset browser GUI
- Search and filtering
- Category-based organization
- Download manager with progress tracking
- Multiple asset source support
- Authentication management
- Health checking and circuit breaker
- User features (collections, history, statistics)

### Supported Platforms
- Linux (x86_64)
- Windows (x86_64)
- macOS (x86_64, universal)

### Requirements
- Godot 4.0 or later
- Internet connection for asset downloads

---

## Release Template

Use this template for new releases:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New features and capabilities

### Changed
- Changes to existing functionality

### Deprecated
- Features that will be removed in upcoming releases

### Removed
- Features that have been removed

### Fixed
- Bug fixes

### Security
- Security improvements and fixes

### Performance
- Performance improvements

[X.Y.Z]: https://github.com/tyevco/gad-dev/compare/vX.Y.Z-1...vX.Y.Z
```

---

[Unreleased]: https://github.com/tyevco/gad-dev/compare/v0.1.0...HEAD
