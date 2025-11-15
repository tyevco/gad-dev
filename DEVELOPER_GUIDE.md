# Godot Asset Browser - Developer Setup Guide

This guide will help you set up a development environment for contributing to the Godot Asset Browser (GAB) project.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Initial Setup](#initial-setup)
3. [Development Workflow](#development-workflow)
4. [Building](#building)
5. [Testing](#testing)
6. [Code Style](#code-style)
7. [Debugging](#debugging)
8. [Common Issues](#common-issues)

---

## Prerequisites

### Required Software

1. **Rust** (1.70 or higher)
   ```bash
   # Install via rustup (recommended)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Verify installation
   rustc --version
   cargo --version
   ```

2. **Godot Engine** (4.0 or higher)
   - Download from [godotengine.org](https://godotengine.org/download)
   - Or use your package manager:
     ```bash
     # macOS (Homebrew)
     brew install godot

     # Linux (Flatpak)
     flatpak install flathub org.godotengine.Godot
     ```

3. **Git**
   ```bash
   git --version
   ```

### Recommended Tools

- **IDE/Editor**: VSCode, CLion, or RustRover
- **VSCode Extensions**:
  - rust-analyzer
  - CodeLLDB (for debugging)
  - Even Better TOML

---

## Initial Setup

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/gad-dev.git
cd gad-dev
```

### 2. Install Rust Dependencies

```bash
cd rust
cargo fetch
```

This will download all required crates.

### 3. Build the Project

```bash
cargo build
```

First build may take several minutes as it compiles all dependencies.

### 4. Set Up Git Hooks (Optional)

```bash
# Install pre-commit hook for code formatting
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/sh
cd rust
cargo fmt -- --check
cargo clippy -- -D warnings
EOF

chmod +x .git/hooks/pre-commit
```

---

## Development Workflow

### Project Structure

```
gad-dev/
├── rust/                    # Rust source code
│   ├── src/
│   │   ├── asset_library/  # Core asset management modules
│   │   ├── gui/            # UI components
│   │   └── lib.rs          # Library entry point
│   ├── Cargo.toml          # Rust dependencies
│   └── target/             # Build artifacts (gitignored)
├── docs/                    # Documentation
├── examples/                # Example code
└── tests/                   # Integration tests
```

### Making Changes

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**

3. **Format your code:**
   ```bash
   cargo fmt
   ```

4. **Check for issues:**
   ```bash
   cargo clippy
   ```

5. **Run tests:**
   ```bash
   cargo test
   ```

6. **Commit your changes:**
   ```bash
   git add .
   git commit -m "feat: Add awesome feature"
   ```

### Commit Message Convention

We follow Conventional Commits:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `chore:` - Maintenance tasks

Examples:
```
feat: Add bulk download functionality
fix: Resolve dependency installation order
docs: Update API documentation for AssetManager
refactor: Simplify download queue management
test: Add unit tests for UserFeatures
```

---

## Building

### Debug Build

For development (faster compilation, includes debug symbols):

```bash
cargo build
```

Output: `target/debug/libgodot_asset_browser.so` (Linux) or similar

### Release Build

For production (optimized, smaller binary):

```bash
cargo build --release
```

Output: `target/release/libgodot_asset_browser.so` (Linux) or similar

### Platform-Specific Notes

#### Linux
```bash
cargo build --release
# Output: target/release/libgodot_asset_browser.so
```

#### macOS
```bash
cargo build --release
# Output: target/release/libgodot_asset_browser.dylib
```

#### Windows
```bash
cargo build --release
# Output: target\release\godot_asset_browser.dll
```

### Testing in Godot

1. Build the library (debug or release)
2. Copy the library to a Godot project:
   ```bash
   # Example for Linux
   cp target/debug/libgodot_asset_browser.so /path/to/godot/project/addons/godot_asset_browser/
   ```
3. Open the Godot project
4. Enable the plugin in Project Settings → Plugins

---

## Testing

### Running All Tests

```bash
cargo test
```

### Running Specific Tests

```bash
# Run tests in a specific module
cargo test asset_manager

# Run a specific test
cargo test test_bulk_install

# Run with output
cargo test -- --nocapture
```

### Writing Tests

Example unit test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_creation() {
        let asset = Asset::new(
            "test_id".to_string(),
            "Test Asset".to_string(),
            AssetCategory::Tools,
            // ... other fields
        );

        assert_eq!(asset.id, "test_id");
        assert_eq!(asset.name, "Test Asset");
    }

    #[tokio::test]
    async fn test_async_download() {
        let manager = AssetManager::new();
        let result = manager.download_asset("test".to_string()).await;
        // assertions...
    }
}
```

---

## Code Style

### Formatting

We use `rustfmt` with default settings:

```bash
cargo fmt
```

### Linting

We use `clippy` for additional checks:

```bash
cargo clippy
```

Fix warnings with:
```bash
cargo clippy --fix
```

### Documentation

All public APIs must be documented:

```rust
/// Downloads an asset from its URL to the cache directory.
///
/// # Arguments
/// * `asset_id` - The ID of the asset to download
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the downloaded file on success
/// * `Err(String)` - Error message on failure
///
/// # Examples
/// ```
/// let path = manager.download_asset("asset_123".to_string()).await?;
/// println!("Downloaded to: {:?}", path);
/// ```
pub async fn download_asset(&self, asset_id: String) -> Result<PathBuf, String> {
    // implementation
}
```

---

## Debugging

### VSCode Setup

Create `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug unit tests",
            "cargo": {
                "args": [
                    "test",
                    "--no-run",
                    "--lib"
                ]
            },
            "args": [],
            "cwd": "${workspaceFolder}/rust"
        }
    ]
}
```

### Logging

Use Godot's logging from Rust:

```rust
use godot::prelude::*;

godot_print!("Debug message");
godot_warn!("Warning message");
godot_error!("Error message");
```

### Debug Builds

Debug builds include:
- Debug symbols
- Runtime checks
- Easier debugging

But are slower and larger.

---

## Common Issues

### Issue: Compilation Errors with gdext

**Problem:** Errors related to Godot bindings

**Solution:**
```bash
# Update gdext to latest version
cd rust
cargo update godot
cargo clean
cargo build
```

### Issue: Library Not Loading in Godot

**Problem:** Godot doesn't recognize the plugin

**Solutions:**
1. Check library path in `.gdextension` file
2. Verify library is in correct location
3. Check Godot version compatibility (4.0+)
4. Look at Godot console for error messages

### Issue: Slow Compilation

**Solutions:**
1. Use debug builds during development
2. Install `sccache` for incremental compilation:
   ```bash
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   ```
3. Use `cargo check` instead of `cargo build` for quick checks
4. Enable parallel compilation in `.cargo/config.toml`:
   ```toml
   [build]
   jobs = 8
   ```

### Issue: Test Failures

**Debugging steps:**
1. Run single test with output:
   ```bash
   cargo test test_name -- --nocapture
   ```
2. Check test isolation - tests should not depend on each other
3. Look for async issues - ensure proper `.await` usage

### Issue: Memory Leaks

**Tools:**
```bash
# Install valgrind (Linux)
sudo apt install valgrind

# Run with valgrind
cargo build
valgrind --leak-check=full ./target/debug/godot_asset_browser
```

---

## Advanced Topics

### Cross-Platform Compilation

#### For Windows (from Linux/Mac)

```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

#### For macOS (from Linux)

Requires osxcross toolchain

#### For Linux (from macOS/Windows)

```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```

### Profiling

#### CPU Profiling

```bash
cargo install flamegraph
cargo flamegraph --test test_name
```

#### Memory Profiling

```bash
cargo install heaptrack
heaptrack target/debug/godot_asset_browser
```

### Benchmarking

```bash
# Add criterion dependency for benchmarks
cargo install cargo-criterion
cargo criterion
```

---

## Continuous Integration

Our CI pipeline (`.github/workflows/ci.yml`) runs:

1. `cargo fmt --check` - Formatting
2. `cargo clippy` - Linting
3. `cargo test` - Tests
4. `cargo build --release` - Release build

Make sure all checks pass before submitting PR.

---

## Getting Help

- **Documentation**: Check `API.md` for API reference
- **Issues**: [GitHub Issues](https://github.com/yourusername/gad-dev/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/gad-dev/discussions)
- **Godot Rust Discord**: Join for gdext-specific questions

---

## Contributing Checklist

Before submitting a PR:

- [ ] Code formatted with `cargo fmt`
- [ ] No clippy warnings (`cargo clippy`)
- [ ] All tests pass (`cargo test`)
- [ ] New features have tests
- [ ] Public APIs have documentation
- [ ] Commit messages follow convention
- [ ] CHANGELOG.md updated (if applicable)
- [ ] No breaking changes (or clearly documented)

---

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [gdext Documentation](https://godot-rust.github.io/docs/gdext/)
- [Godot Engine Documentation](https://docs.godotengine.org/)
- [Async Rust](https://rust-lang.github.io/async-book/)

---

Happy coding! 🦀
