# Build and Release Process

## Overview

This document describes the complete build and release process for the Godot Asset Browser plugin, including compilation, testing, packaging, and distribution.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Development Build](#development-build)
3. [Release Build](#release-build)
4. [Testing Before Release](#testing-before-release)
5. [Packaging](#packaging)
6. [Publishing Release](#publishing-release)
7. [Post-Release](#post-release)
8. [Hotfix Process](#hotfix-process)

---

## Prerequisites

### Required Tools

#### All Platforms
- **Rust** 1.70 or higher
  - Install: https://rustup.rs/
  - Verify: `rustc --version`
- **Git** for version control
  - Install: https://git-scm.com/
  - Verify: `git --version`
- **Godot Engine** 4.0 or higher
  - Download: https://godotengine.org/download
  - Verify: `godot --version`

#### Platform-Specific

**Linux:**
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# Fedora
sudo dnf groupinstall "Development Tools"
sudo dnf install openssl-devel

# Arch
sudo pacman -S base-devel openssl
```

**Windows:**
- Visual Studio 2019+ with C++ tools, or
- MinGW-w64 toolchain

**macOS:**
```bash
xcode-select --install
brew install openssl pkg-config
```

### Optional Tools

- **GitHub CLI** (`gh`) for automated releases
  - Install: https://cli.github.com/
- **cargo-release** for version management
  - Install: `cargo install cargo-release`
- **cargo-bump** for semantic versioning
  - Install: `cargo install cargo-bump`

---

## Development Build

### Building Rust Components

```bash
cd rust

# Debug build (fast compilation, slower runtime)
cargo build

# Check for errors without building
cargo check

# Run tests
cargo test

# Run clippy (linter)
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Building for Godot

```bash
# Build the GDExtension library
cargo build --release

# The output will be in:
# Linux: target/release/libgodot_asset_browser.so
# Windows: target/release/godot_asset_browser.dll
# macOS: target/release/libgodot_asset_browser.dylib
```

### Testing in Godot

1. Copy the built library to your test project:
   ```bash
   # Linux
   cp target/release/libgodot_asset_browser.so ../test_project/addons/godot_asset_browser/

   # Windows
   copy target\release\godot_asset_browser.dll ..\test_project\addons\godot_asset_browser\

   # macOS
   cp target/release/libgodot_asset_browser.dylib ../test_project/addons/godot_asset_browser/
   ```

2. Open the test project in Godot
3. Verify the plugin loads without errors
4. Test all functionality manually

---

## Release Build

### Version Management

#### 1. Update Version Numbers

Update version in all relevant files:

**rust/Cargo.toml:**
```toml
[package]
name = "godot_asset_browser"
version = "1.2.0"  # Update this
edition = "2021"
```

**plugin.cfg:**
```ini
[plugin]
name="Godot Asset Browser"
description="Browse and install assets from the Godot Asset Library"
author="Your Name"
version="1.2.0"  # Update this
```

**asset_library_gui.gdextension:**
```toml
[configuration]
entry_symbol = "gdext_rust_init"
compatibility_minimum = 4.0
version = "1.2.0"  # Update this
```

**README.md:**
```markdown
# Godot Asset Browser

Version: 1.2.0  # Update this
```

#### 2. Update Changelog

**CHANGELOG.md:**
```markdown
# Changelog

## [1.2.0] - 2024-XX-XX

### Added
- New feature descriptions
- Another feature

### Changed
- Modified behavior descriptions

### Fixed
- Bug fix descriptions

### Breaking Changes
- Any breaking changes (if major version bump)
```

#### 3. Commit Version Changes

```bash
git add .
git commit -m "chore: bump version to 1.2.0"
git push origin main
```

### Building Release Binaries

#### Build for All Platforms

**Linux Build:**
```bash
cd rust
cargo build --release

# Output: target/release/libgodot_asset_browser.so
```

**Windows Cross-Compile (from Linux):**
```bash
# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu

# Output: target/x86_64-pc-windows-gnu/release/godot_asset_browser.dll
```

**Windows Native Build:**
```powershell
cd rust
cargo build --release

# Output: target\release\godot_asset_browser.dll
```

**macOS Build (x86_64):**
```bash
cd rust
cargo build --release

# Output: target/release/libgodot_asset_browser.dylib
```

**macOS Build (ARM64 - Apple Silicon):**
```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Output: target/aarch64-apple-darwin/release/libgodot_asset_browser.dylib
```

#### Universal macOS Binary

```bash
# Build both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create universal binary
lipo -create \
  target/x86_64-apple-darwin/release/libgodot_asset_browser.dylib \
  target/aarch64-apple-darwin/release/libgodot_asset_browser.dylib \
  -output target/release/libgodot_asset_browser.dylib
```

### Automated Build Script

Create `scripts/build_all.sh`:

```bash
#!/bin/bash
set -e

echo "Building for all platforms..."

# Linux
echo "Building for Linux..."
cargo build --release

# Windows
echo "Building for Windows..."
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# macOS (if on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Building for macOS..."
    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin
    cargo build --release --target x86_64-apple-darwin
    cargo build --release --target aarch64-apple-darwin

    # Create universal binary
    lipo -create \
        target/x86_64-apple-darwin/release/libgodot_asset_browser.dylib \
        target/aarch64-apple-darwin/release/libgodot_asset_browser.dylib \
        -output target/release/libgodot_asset_browser.dylib
fi

echo "Build complete!"
```

Make it executable:
```bash
chmod +x scripts/build_all.sh
```

---

## Testing Before Release

### Automated Testing

```bash
cd rust

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test suite
cargo test --lib asset_library::

# Check code coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Manual Testing Checklist

Create `RELEASE_TESTING_CHECKLIST.md`:

```markdown
# Release Testing Checklist

## Platform Testing
- [ ] Linux (Ubuntu 20.04)
- [ ] Linux (Arch/Fedora)
- [ ] Windows 10
- [ ] Windows 11
- [ ] macOS Intel
- [ ] macOS Apple Silicon

## Core Functionality
- [ ] Plugin loads in Godot without errors
- [ ] Asset Browser tab appears in bottom panel
- [ ] Asset list populates from Godot Asset Library
- [ ] Search functionality works
- [ ] Category filter works
- [ ] Sort options work
- [ ] Asset details display correctly
- [ ] Download progress shows correctly
- [ ] Asset installation succeeds
- [ ] Asset uninstallation succeeds
- [ ] Asset updates work
- [ ] Dependency resolution works
- [ ] Error handling displays correctly
- [ ] Offline mode functions

## Performance
- [ ] Loads 100+ assets without lag
- [ ] Search is responsive (< 100ms)
- [ ] Downloads don't block UI
- [ ] Memory usage is reasonable (< 100MB)

## Regression Testing
- [ ] All previous features still work
- [ ] No new console errors/warnings
- [ ] Configuration persists across restarts
- [ ] Cache management functions correctly

## Documentation
- [ ] README is up to date
- [ ] CHANGELOG is complete
- [ ] API documentation is current
- [ ] User guide reflects new features
```

### Integration Testing in Real Projects

Test the plugin in actual Godot projects:

1. **Empty Project**: Test basic functionality
2. **Small Project**: Test integration with existing assets
3. **Large Project**: Test performance and stability

---

## Packaging

### Directory Structure

```
godot_asset_browser_v1.2.0/
├── addons/
│   └── godot_asset_browser/
│       ├── rust/
│       │   └── lib/
│       │       ├── linux/
│       │       │   └── libgodot_asset_browser.so
│       │       ├── windows/
│       │       │   └── godot_asset_browser.dll
│       │       └── macos/
│       │           └── libgodot_asset_browser.dylib
│       ├── plugin.cfg
│       ├── asset_library_gui.gdextension
│       ├── icon.png
│       └── README.md
├── LICENSE
├── README.md
├── CHANGELOG.md
└── USER_GUIDE.md
```

### Packaging Script

Create `scripts/package_release.sh`:

```bash
#!/bin/bash
set -e

VERSION="$1"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.2.0"
    exit 1
fi

PACKAGE_DIR="godot_asset_browser_v${VERSION}"
RELEASE_DIR="releases"

echo "Packaging version ${VERSION}..."

# Create directories
mkdir -p "${RELEASE_DIR}"
rm -rf "${PACKAGE_DIR}"
mkdir -p "${PACKAGE_DIR}/addons/godot_asset_browser/rust/lib"

# Copy plugin files
cp -r addons/godot_asset_browser/* "${PACKAGE_DIR}/addons/godot_asset_browser/"

# Copy binaries
mkdir -p "${PACKAGE_DIR}/addons/godot_asset_browser/rust/lib/linux"
mkdir -p "${PACKAGE_DIR}/addons/godot_asset_browser/rust/lib/windows"
mkdir -p "${PACKAGE_DIR}/addons/godot_asset_browser/rust/lib/macos"

cp rust/target/release/libgodot_asset_browser.so \
   "${PACKAGE_DIR}/addons/godot_asset_browser/rust/lib/linux/"

cp rust/target/x86_64-pc-windows-gnu/release/godot_asset_browser.dll \
   "${PACKAGE_DIR}/addons/godot_asset_browser/rust/lib/windows/"

cp rust/target/release/libgodot_asset_browser.dylib \
   "${PACKAGE_DIR}/addons/godot_asset_browser/rust/lib/macos/"

# Copy documentation
cp LICENSE "${PACKAGE_DIR}/"
cp README.md "${PACKAGE_DIR}/"
cp CHANGELOG.md "${PACKAGE_DIR}/"
cp USER_GUIDE.md "${PACKAGE_DIR}/"

# Create ZIP archive
cd "${PACKAGE_DIR}/.."
zip -r "${RELEASE_DIR}/godot_asset_browser_v${VERSION}.zip" "${PACKAGE_DIR}"

# Create tar.gz archive (for Linux users)
tar -czf "${RELEASE_DIR}/godot_asset_browser_v${VERSION}.tar.gz" "${PACKAGE_DIR}"

# Cleanup
rm -rf "${PACKAGE_DIR}"

echo "Packages created:"
echo "  - ${RELEASE_DIR}/godot_asset_browser_v${VERSION}.zip"
echo "  - ${RELEASE_DIR}/godot_asset_browser_v${VERSION}.tar.gz"
```

### Verify Package

```bash
# Extract and verify ZIP
unzip -l releases/godot_asset_browser_v1.2.0.zip

# Test in Godot
mkdir test_project
cd test_project
unzip ../releases/godot_asset_browser_v1.2.0.zip
# Open in Godot and test
```

---

## Publishing Release

### Create Git Tag

```bash
# Create annotated tag
git tag -a v1.2.0 -m "Release version 1.2.0"

# Push tag to remote
git push origin v1.2.0

# Or push all tags
git push origin --tags
```

### GitHub Release

#### Manual Release

1. Go to https://github.com/your-org/godot-asset-browser/releases
2. Click "Draft a new release"
3. **Tag version**: Select v1.2.0
4. **Release title**: "Godot Asset Browser v1.2.0"
5. **Description**: Copy from CHANGELOG.md
6. **Attach binaries**:
   - `godot_asset_browser_v1.2.0.zip`
   - `godot_asset_browser_v1.2.0.tar.gz`
7. Click "Publish release"

#### Automated Release with GitHub CLI

Create `scripts/create_release.sh`:

```bash
#!/bin/bash
set -e

VERSION="$1"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

# Extract changelog for this version
CHANGELOG=$(awk "/## \[${VERSION}\]/,/## \[/" CHANGELOG.md | sed '1d;$d')

# Create GitHub release
gh release create "v${VERSION}" \
    --title "Godot Asset Browser v${VERSION}" \
    --notes "${CHANGELOG}" \
    "releases/godot_asset_browser_v${VERSION}.zip" \
    "releases/godot_asset_browser_v${VERSION}.tar.gz"

echo "Release v${VERSION} published!"
echo "View at: https://github.com/your-org/godot-asset-browser/releases/tag/v${VERSION}"
```

### Godot Asset Library Submission

1. **Prepare Asset**:
   - Ensure plugin.cfg is correct
   - Create icon (256x256 PNG)
   - Prepare screenshots (1280x720 recommended)

2. **Submit to Asset Library**:
   - Go to https://godotengine.org/asset-library/asset/submit
   - Fill in the form:
     - **Title**: Godot Asset Browser
     - **Description**: [Copy from README]
     - **Category**: Addon
     - **Repository URL**: https://github.com/your-org/godot-asset-browser
     - **License**: MIT
     - **Version**: 1.2.0
     - **Godot Version**: 4.0+
     - **Icon**: Upload icon.png
     - **Screenshots**: Upload at least 3 screenshots
   - Submit for review

3. **Wait for Approval**:
   - Asset library team reviews submissions
   - Usually takes 1-3 days
   - You'll receive email notification

---

## Post-Release

### Announcement

Announce the release on:
- [x] GitHub Discussions
- [x] Reddit (r/godot)
- [x] Discord (Godot Discord server)
- [x] Twitter/X
- [x] Godot Forum
- [x] Project website/blog

**Announcement Template:**

```markdown
# Godot Asset Browser v1.2.0 Released!

We're excited to announce version 1.2.0 of the Godot Asset Browser plugin!

## What's New

[Copy highlights from CHANGELOG]

## Installation

Download from:
- GitHub Releases: [link]
- Godot Asset Library: [link]

See the [User Guide](link) for installation instructions.

## Feedback

Try it out and let us know what you think!
- Report bugs: [GitHub Issues]
- Suggest features: [GitHub Discussions]
- Join discussion: [Discord]

Thank you to all contributors!
```

### Update Documentation Site

If you have a documentation website:
```bash
# Update docs
cd docs
./update_version.sh 1.2.0
./build_docs.sh
./deploy_docs.sh
```

### Monitor Issues

After release:
- Monitor GitHub Issues for bug reports
- Respond to questions promptly
- Track feature requests
- Plan next release based on feedback

### Metrics Tracking

Track release metrics:
- Download count (GitHub API)
- Star count
- Issues opened/closed
- Forum discussion activity

---

## Hotfix Process

For critical bugs discovered after release:

### 1. Create Hotfix Branch

```bash
git checkout main
git pull origin main
git checkout -b hotfix/1.2.1
```

### 2. Fix the Bug

```bash
# Make necessary changes
# Test thoroughly
cargo test

# Commit fix
git add .
git commit -m "fix: critical bug in asset installation"
```

### 3. Update Version

Update to patch version (1.2.0 → 1.2.1):
- Cargo.toml
- plugin.cfg
- asset_library_gui.gdextension

### 4. Build and Test

```bash
./scripts/build_all.sh
# Test on all platforms
```

### 5. Merge and Release

```bash
# Merge to main
git checkout main
git merge hotfix/1.2.1
git push origin main

# Create tag
git tag -a v1.2.1 -m "Hotfix: critical bug fixes"
git push origin v1.2.1

# Package and release
./scripts/package_release.sh 1.2.1
./scripts/create_release.sh 1.2.1
```

### 6. Notify Users

Send notification about the hotfix:
- GitHub release with "hotfix" label
- Update forum posts
- Discord announcement

---

## CI/CD Automation

### GitHub Actions Workflow

Create `.github/workflows/release.yml`:

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: cd rust && cargo build --release
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: linux-lib
          path: rust/target/release/libgodot_asset_browser.so

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: cd rust && cargo build --release
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: windows-lib
          path: rust/target/release/godot_asset_browser.dll

  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Build x86_64
        run: |
          cd rust
          cargo build --release --target x86_64-apple-darwin
      - name: Build ARM64
        run: |
          cd rust
          rustup target add aarch64-apple-darwin
          cargo build --release --target aarch64-apple-darwin
      - name: Create universal binary
        run: |
          lipo -create \
            rust/target/x86_64-apple-darwin/release/libgodot_asset_browser.dylib \
            rust/target/aarch64-apple-darwin/release/libgodot_asset_browser.dylib \
            -output rust/target/release/libgodot_asset_browser.dylib
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: macos-lib
          path: rust/target/release/libgodot_asset_browser.dylib

  create-release:
    needs: [build-linux, build-windows, build-macos]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Download artifacts
        uses: actions/download-artifact@v3

      - name: Package release
        run: |
          mkdir -p release/addons/godot_asset_browser/rust/lib/{linux,windows,macos}
          cp linux-lib/* release/addons/godot_asset_browser/rust/lib/linux/
          cp windows-lib/* release/addons/godot_asset_browser/rust/lib/windows/
          cp macos-lib/* release/addons/godot_asset_browser/rust/lib/macos/

          # Copy plugin files
          cp -r addons/godot_asset_browser/* release/addons/godot_asset_browser/
          cp LICENSE README.md CHANGELOG.md USER_GUIDE.md release/

          # Create archives
          cd release
          zip -r ../godot_asset_browser_${GITHUB_REF_NAME}.zip .
          tar -czf ../godot_asset_browser_${GITHUB_REF_NAME}.tar.gz .

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            godot_asset_browser_${{ github.ref_name }}.zip
            godot_asset_browser_${{ github.ref_name }}.tar.gz
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

## Checklist

### Pre-Release Checklist

- [ ] All tests passing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version numbers updated in all files
- [ ] Code reviewed and approved
- [ ] Manual testing completed on all platforms
- [ ] No open critical bugs
- [ ] Dependencies up to date
- [ ] Security audit passed

### Release Checklist

- [ ] Git tag created
- [ ] Binaries built for all platforms
- [ ] Release packaged (ZIP and tar.gz)
- [ ] GitHub release created
- [ ] Release notes published
- [ ] Godot Asset Library updated
- [ ] Documentation site updated
- [ ] Announcements posted

### Post-Release Checklist

- [ ] Monitor GitHub Issues
- [ ] Respond to user feedback
- [ ] Update roadmap based on feedback
- [ ] Plan next release
- [ ] Thank contributors

---

## Resources

- **Semantic Versioning**: https://semver.org/
- **Keep a Changelog**: https://keepachangelog.com/
- **GitHub Releases**: https://docs.github.com/en/repositories/releasing-projects-on-github
- **Godot Asset Library**: https://docs.godotengine.org/en/stable/community/asset_library/submitting_to_assetlib.html
- **Rust Release Process**: https://doc.rust-lang.org/cargo/reference/publishing.html
