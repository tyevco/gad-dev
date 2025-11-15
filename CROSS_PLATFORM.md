# Cross-Platform Compatibility Guide

## Overview

The Godot Asset Browser is designed to work across Windows, Linux, and macOS. This document outlines platform-specific considerations, testing procedures, and known compatibility notes.

## Supported Platforms

- ✅ **Linux** (x86_64, tested on Ubuntu 20.04+)
- ✅ **Windows** (x86_64, Windows 10/11)
- ✅ **macOS** (x86_64 and ARM64/M1+, macOS 11+)

## Platform-Specific Code

### Unix File Permissions (asset_manager.rs:1430)

```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    if let Some(mode) = file.unix_mode() {
        fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
            .ok(); // Ignore permission errors
    }
}
```

**Purpose:** Preserves Unix file permissions when extracting ZIP archives on Linux/macOS.

**Windows Behavior:** This code block is not compiled on Windows. Windows uses a different permission model (ACLs), which is handled automatically by the filesystem.

**Impact:** None. This is a proper use of conditional compilation.

## Cross-Platform Compatibility Considerations

### 1. Path Handling

**Status:** ✅ **Compatible**

All path operations use Rust's `std::path::PathBuf` and `Path`, which handle platform-specific path separators automatically:
- Windows: `C:\Users\Name\AppData\...` (backslash)
- Unix: `/home/user/.local/...` (forward slash)

**Locations:**
- `asset_manager.rs`: Uses `PathBuf::join()` for all path operations
- `config_manager.rs`: Platform-agnostic path construction
- `download_manager.rs`: Proper path handling for temp files
- `repository_adapter.rs`: Handles file:// URIs correctly

**Best Practices Applied:**
- ✅ Never use hardcoded path separators (`/` or `\`)
- ✅ Use `PathBuf::join()` for combining paths
- ✅ Use `Path::new()` for creating paths from strings
- ✅ Let Rust handle path normalization

### 2. File System Operations

**Status:** ✅ **Compatible**

- File creation, deletion, and directory operations use `std::fs`
- All operations are platform-agnostic
- Proper error handling for permission issues
- Temporary files use `tempfile` crate (cross-platform)

**Archive Extraction:**
- ZIP files: `zip` crate (works on all platforms)
- tar.gz files: `tar` + `flate2` crates (works on all platforms)

### 3. Network Operations

**Status:** ✅ **Compatible**

- HTTP client: `reqwest` crate (cross-platform)
- Async runtime: `tokio` (cross-platform)
- All network operations are platform-agnostic

### 4. Godot Integration

**Status:** ✅ **Compatible**

- GDExtension (godot-rust): Supports all platforms where Godot 4.x runs
- No platform-specific Godot API calls
- GUI components use portable Godot APIs

## Testing on Each Platform

### Linux

#### Build Requirements
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev

# Fedora/RHEL
sudo dnf groupinstall "Development Tools"
sudo dnf install openssl-devel

# Arch
sudo pacman -S base-devel openssl
```

#### Building
```bash
cd rust
cargo build --release
cargo test --lib
```

#### Testing
```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --lib asset_library::
cargo test --lib gui::
```

#### Godot Setup
```bash
# Download Godot 4.x for Linux
wget https://github.com/godotengine/godot-builds/releases/download/4.x/Godot_v4.x_linux.x86_64.zip
unzip Godot_v4.x_linux.x86_64.zip
./Godot_v4.x_linux.x86_64 --path /path/to/project
```

### Windows

#### Build Requirements
- Visual Studio 2019+ with C++ tools, or
- MinGW-w64 toolchain
- Rust (via rustup-init.exe)

#### Building
```powershell
cd rust
cargo build --release
cargo test --lib
```

#### Testing
```powershell
# Run all tests
cargo test

# Run specific test suites
cargo test --lib asset_library::
cargo test --lib gui::
```

#### Common Issues
- **OpenSSL:** If you encounter OpenSSL errors, use the `windows-msvc` target:
  ```powershell
  # Install vcpkg
  vcpkg install openssl:x64-windows

  # Or use rustls instead
  # (requires modifying Cargo.toml to use rustls feature)
  ```

#### Godot Setup
```powershell
# Download Godot 4.x for Windows
# Extract and run
Godot_v4.x_win64.exe --path C:\path\to\project
```

### macOS

#### Build Requirements
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install openssl pkg-config
```

#### Building
```bash
cd rust
cargo build --release
cargo test --lib
```

#### Testing
```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --lib asset_library::
cargo test --lib gui::
```

#### Apple Silicon (M1/M2) Notes
- Rust natively supports ARM64 (aarch64-apple-darwin)
- Godot 4.x has native ARM64 builds
- No special configuration needed

#### Godot Setup
```bash
# Download Godot 4.x for macOS
# Extract and run
open Godot.app --args --path /path/to/project
```

## Continuous Integration

### GitHub Actions Example

```yaml
name: Cross-Platform Tests

on: [push, pull_request]

jobs:
  test-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run tests
        run: cd rust && cargo test --lib

  test-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run tests
        run: cd rust && cargo test --lib

  test-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run tests
        run: cd rust && cargo test --lib

  test-macos-arm:
    runs-on: macos-14  # Apple Silicon runner
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run tests
        run: cd rust && cargo test --lib
```

## Known Platform-Specific Behaviors

### Line Endings
- **Windows:** CRLF (`\r\n`)
- **Unix/macOS:** LF (`\n`)
- **Solution:** Git's `core.autocrlf` handles this automatically
- **Code:** All Rust string operations are line-ending agnostic

### File Permissions
- **Unix/macOS:** Executable bit is preserved in ZIP extraction
- **Windows:** No executable bit concept; uses file extensions (.exe, .bat)
- **Impact:** ZIP archives maintain Unix permissions where applicable

### Case Sensitivity
- **Linux:** Case-sensitive filesystem (default)
- **macOS:** Case-insensitive by default (can be case-sensitive)
- **Windows:** Case-insensitive filesystem
- **Best Practice:** Always use consistent casing in file paths

### Path Length Limits
- **Windows:** 260 character limit (MAX_PATH) by default
  - Can be extended with long path support (Windows 10+)
  - GDExtension handles this automatically
- **Unix/macOS:** 4096 character limit (much more generous)
- **Mitigation:** Keep asset paths reasonably short

## Testing Checklist

### Per-Platform Test Run

- [ ] **Build succeeds** (`cargo build --release`)
- [ ] **All unit tests pass** (`cargo test --lib`)
- [ ] **Archive extraction works** (test ZIP and tar.gz)
- [ ] **File operations work** (create, read, write, delete)
- [ ] **Network operations work** (HTTP downloads)
- [ ] **Godot integration loads** (GDExtension library loads successfully)
- [ ] **GUI displays correctly** (no rendering issues)
- [ ] **Search/filter functions work**
- [ ] **Asset installation works**

### Integration Testing

- [ ] Install an asset from Godot Asset Library
- [ ] Verify asset files are extracted to correct location
- [ ] Verify asset appears in installed assets list
- [ ] Uninstall an asset
- [ ] Verify asset is removed completely

### Performance Testing

- [ ] Load 100+ assets in list
- [ ] Search/filter with large asset count
- [ ] Download multiple assets concurrently
- [ ] Extract large archives (100+ MB)

## Debugging Platform-Specific Issues

### Enable Verbose Logging

```rust
// In your code
#[cfg(debug_assertions)]
println!("Debug info: {:?}", value);
```

### Platform Detection

```rust
#[cfg(target_os = "windows")]
println!("Running on Windows");

#[cfg(target_os = "linux")]
println!("Running on Linux");

#[cfg(target_os = "macos")]
println!("Running on macOS");
```

### Testing Platform-Specific Code Paths

```rust
#[cfg(test)]
mod tests {
    #[test]
    #[cfg(unix)]
    fn test_unix_specific_feature() {
        // Test Unix-specific code
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_specific_feature() {
        // Test Windows-specific code
    }
}
```

## Deployment

### Binary Distribution

#### Linux
- Build with `cargo build --release`
- Binary: `target/release/libgodot_asset_browser.so`
- Distribution: AppImage, Flatpak, or .deb package

#### Windows
- Build with `cargo build --release`
- Binary: `target/release/godot_asset_browser.dll`
- Distribution: Installer (.exe) or portable ZIP

#### macOS
- Build with `cargo build --release`
- Binary: `target/release/libgodot_asset_browser.dylib`
- Distribution: .app bundle or .dmg image
- **Note:** Code signing required for distribution

### Cross-Compilation

#### From Linux to Windows
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

#### From Linux to macOS
```bash
# Requires osxcross
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

#### From macOS to ARM (M1/M2)
```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

## Reporting Platform-Specific Issues

When reporting issues, please include:

1. **Platform:** OS name and version
2. **Architecture:** x86_64, ARM64, etc.
3. **Rust version:** `rustc --version`
4. **Godot version:** Version number and build type
5. **Error output:** Full error message and stack trace
6. **Steps to reproduce:** Detailed reproduction steps

## Resources

- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [GDExtension Platform Notes](https://docs.godotengine.org/en/stable/tutorials/scripting/gdextension/index.html)
- [Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Godot Export Templates](https://godotengine.org/download)

## Conclusion

The Godot Asset Browser is designed with cross-platform compatibility as a priority. All dependencies are platform-agnostic, and platform-specific code is properly isolated using conditional compilation. Regular testing on all three major platforms ensures consistent behavior across environments.
