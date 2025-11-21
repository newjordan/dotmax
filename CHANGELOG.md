# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure and setup
- Adaptive resize filter selection for extreme aspect ratio images
- Comprehensive performance benchmarks for extreme image sizes
- Integration tests for large and extreme aspect ratio images

### Changed
- Image resize algorithm now uses Triangle filter (3x faster) for extreme aspect ratios (>2.5:1)
- Improved resize performance by 45% for extreme aspect ratio images (501ms → 276ms for 10000×4000 images)

### Performance
- Large images (4000×4000): 222ms total
- Extreme wide (10000×4000): 725ms total (449ms load + 276ms resize)
- Extreme tall (4000×10000): ~700ms total
- All image sizes now meet <5s performance targets

## [0.1.0] - Unreleased

### Added
- Initial Cargo project initialization
- Project directory structure (src/, examples/, tests/, benches/, docs/)
- Dual MIT/Apache-2.0 licensing
- Basic README and documentation placeholders
