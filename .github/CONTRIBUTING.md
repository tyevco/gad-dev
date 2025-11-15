# Contributing to Godot Asset Browser

Thank you for your interest in contributing to Godot Asset Browser! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow. Please be respectful, inclusive, and constructive in all interactions.

### Our Standards

- **Be respectful**: Treat everyone with respect and kindness
- **Be inclusive**: Welcome diverse perspectives and experiences
- **Be constructive**: Provide helpful feedback and suggestions
- **Be professional**: Keep discussions focused on the project

## Getting Started

### Prerequisites

- **Rust**: 1.70 or later (stable toolchain)
- **Godot**: 4.0 or later
- **Git**: For version control
- **Cargo**: Comes with Rust installation

### Setting Up Development Environment

1. **Fork the repository** on GitHub

2. **Clone your fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/gad-dev.git
   cd gad-dev
   ```

3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/tyevco/gad-dev.git
   ```

4. **Install Rust toolchain**:
   ```bash
   rustup update stable
   rustup component add rustfmt clippy
   ```

5. **Build the project**:
   ```bash
   cd rust
   cargo build
   ```

6. **Run tests**:
   ```bash
   cargo test --lib
   ```

## Development Workflow

### 1. Create a Branch

Always create a new branch for your work:

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `perf/` - Performance improvements
- `refactor/` - Code refactoring

### 2. Make Changes

- Write clear, concise commit messages
- Keep commits focused and atomic
- Follow the coding standards (see below)

### 3. Test Your Changes

```bash
# Run all tests
cargo test --lib

# Run specific test
cargo test --lib test_name

# Run with output
cargo test --lib -- --nocapture

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy --all-targets --all-features
```

### 4. Commit Your Changes

```bash
git add .
git commit -m "type: brief description

Longer description if needed.

Fixes #issue-number"
```

Commit message types:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `perf:` - Performance improvement
- `test:` - Adding or updating tests
- `refactor:` - Code refactoring
- `style:` - Formatting changes
- `ci:` - CI/CD changes

### 5. Push and Create Pull Request

```bash
git push origin your-branch-name
```

Then create a pull request on GitHub.

## Coding Standards

### Rust Code Style

We follow the standard Rust style guidelines:

1. **Formatting**: Use `cargo fmt` before committing
   ```bash
   cargo fmt
   ```

2. **Linting**: Fix all `clippy` warnings
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Naming Conventions**:
   - `snake_case` for functions, variables, modules
   - `PascalCase` for types, structs, enums
   - `SCREAMING_SNAKE_CASE` for constants
   - Descriptive names that explain purpose

4. **Documentation**:
   - Add doc comments (`///`) for public APIs
   - Include examples in documentation
   - Explain complex logic with inline comments

5. **Error Handling**:
   - Use `Result<T, E>` for fallible operations
   - Provide meaningful error messages
   - Avoid unwrap/expect in library code

### Code Organization

- Keep functions small and focused
- Group related functionality in modules
- Maintain clear separation of concerns
- Follow existing project structure

### Example

```rust
/// Calculates the total size of all assets in a category
///
/// # Arguments
/// * `category` - The asset category to count
///
/// # Returns
/// * `usize` - Total number of assets in the category
///
/// # Example
/// ```
/// let count = manager.count_assets_by_category(AssetCategory::Tools);
/// assert!(count > 0);
/// ```
pub fn count_assets_by_category(&self, category: AssetCategory) -> usize {
    self.assets
        .lock()
        .unwrap()
        .iter()
        .filter(|asset| asset.category == category)
        .count()
}
```

## Testing

### Writing Tests

- Write unit tests for all new functionality
- Use descriptive test names: `test_feature_description`
- Test edge cases and error conditions
- Use `#[cfg(test)]` for test modules

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_creation() {
        let asset = Asset::new(/* ... */);
        assert_eq!(asset.id, "expected_id");
    }

    #[test]
    fn test_error_handling() {
        let result = some_fallible_operation();
        assert!(result.is_err());
    }
}
```

### Performance Tests

For performance-critical code, consider adding benchmarks:

```rust
// In rust/benches/asset_loading.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("operation_name", |b| {
        b.iter(|| {
            // Code to benchmark
        });
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

## Pull Request Process

### Before Submitting

1. **Update your branch**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all checks**:
   ```bash
   cargo fmt
   cargo clippy --all-targets --all-features
   cargo test --lib
   ```

3. **Update documentation** if needed

4. **Add yourself to contributors** (if first contribution)

### Submitting the PR

1. Fill out the pull request template completely
2. Link related issues using `Fixes #123` or `Relates to #456`
3. Provide clear description of changes
4. Include screenshots for UI changes
5. Wait for CI checks to pass

### Review Process

- Maintainers will review your PR within a few days
- Address feedback and requested changes
- Keep the conversation constructive
- Once approved, a maintainer will merge your PR

### After Merge

1. **Delete your branch**:
   ```bash
   git branch -d your-branch-name
   git push origin --delete your-branch-name
   ```

2. **Update your fork**:
   ```bash
   git checkout main
   git pull upstream main
   git push origin main
   ```

## Reporting Bugs

Use the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.yml) and provide:

- Clear description of the bug
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Godot version, plugin version)
- Error logs and screenshots

## Suggesting Features

Use the [Feature Request template](.github/ISSUE_TEMPLATE/feature_request.yml) and provide:

- Problem description
- Proposed solution
- Use cases
- Priority level
- Mockups or examples (if applicable)

## Performance Considerations

When contributing performance-sensitive code:

1. **Measure first**: Profile before optimizing
2. **Document trade-offs**: Explain performance decisions
3. **Add benchmarks**: Include criterion benchmarks
4. **Avoid premature optimization**: Focus on correctness first

See [PERFORMANCE_OPTIMIZATION.md](../PERFORMANCE_OPTIMIZATION.md) for details.

## Documentation

### Code Documentation

- Use `///` for public API documentation
- Use `//!` for module-level documentation
- Include examples in documentation
- Explain why, not just what

### User Documentation

When adding user-facing features:

- Update `USER_GUIDE.md`
- Add examples
- Include screenshots
- Explain common use cases

## Questions?

- Check existing documentation
- Search closed issues
- Ask in GitHub Discussions
- Reach out to maintainers

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

---

Thank you for contributing to Godot Asset Browser! 🎉
