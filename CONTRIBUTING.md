# Contributing to termcompositor

Thank you for your interest in contributing to termcompositor! This document provides guidelines and information for contributors.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Commit Message Guidelines](#commit-message-guidelines)
- [Reporting Issues](#reporting-issues)
- [Code of Conduct](#code-of-conduct)

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/termcompositor.git
   cd termcompositor
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/216598762/termcompositor.git
   ```
4. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.73 or higher (see MSRV in Cargo.toml)
- Cargo (Rust package manager)
- Git

### Installation

1. **Build the project**:
   ```bash
   cargo build
   ```

2. **Run tests**:
   ```bash
   cargo test
   ```

3. **Run with all features** (for full development):
   ```bash
   cargo build --all-features
   ```

### Feature Flags

| Feature | Default | Enables |
|---------|---------|---------|
| `font-rasterizer` | **on** | Real glyph rendering in `TextLayer` via `fontdue` |
| `kitty-encoder` | off | Kitty graphics protocol output |
| `sixel-encoder` | off | Sixel output |
| `image-decoder` | off | `ImageLayer` (PNG + JPEG) |

## Code Style

We use `cargo clippy` and `cargo fmt` for linting and formatting.

### Formatting

```bash
# Check formatting
cargo fmt --check

# Auto-format
cargo fmt
```

### Linting

```bash
# Check for issues
cargo clippy --all-targets

# Check with warnings as errors
cargo clippy --all-targets -- -D warnings
```

### Code Conventions

- Follow Rust API Guidelines
- Use `#[must_use]` on builder methods
- Add documentation comments (`///`) for all public items
- Keep functions focused and reasonably sized
- Use meaningful variable and function names
- Prefer `Result<(), Error>` over `panic!` in library code

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with a specific feature
cargo test --features font-rasterizer

# Run tests with all features
cargo test --all-features

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Writing Tests

- Place unit tests in the same file as the code they test (Rust convention)
- Place integration tests in the `tests/` directory
- Name test functions `test_*` or use `#[test]` attribute
- Use descriptive test names that explain what is being tested
- Test edge cases and error conditions

Example test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_layer_creation() {
        let layer = RectLayer::new(0, 0, 100, 50, [255, 0, 0, 255]);
        let mut fb = FrameBuffer::new(200, 200);
        layer.render(&mut fb, (0, 0), 1.0);
        // Verify the layer was rendered
        let pixel = fb.get_pixel(10, 10);
        assert_eq!(pixel, [255, 0, 0, 255]);
    }
}
```

### Test Coverage

We aim for comprehensive test coverage. Check coverage with:

```bash
# Run all tests with verbose output
cargo test --all-features 2>&1 | tail -30
```

## Pull Request Process

### Before Submitting

1. **Update your fork**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run tests**:
   ```bash
   cargo test --all-features
   ```

3. **Run linter**:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

4. **Run formatter**:
   ```bash
   cargo fmt --check
   ```

5. **Commit your changes** (see [Commit Message Guidelines](#commit-message-guidelines))

### Submitting a Pull Request

1. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a Pull Request** on GitHub

3. **Fill out the PR template**:
   - Description of changes
   - Related issues (if any)
   - Testing done
   - Checklist (tests pass, linting passes, etc.)

### PR Requirements

- [ ] All tests pass
- [ ] Code follows style guidelines (`cargo clippy`, `cargo fmt`)
- [ ] Documentation is updated (if applicable)
- [ ] Commit messages follow guidelines
- [ ] PR has a clear description

## Commit Message Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, etc.)
- **refactor**: Code refactoring
- **test**: Adding or updating tests
- **chore**: Maintenance tasks
- **perf**: Performance improvements

### Examples

```
feat(layer): add GradientLayerBuilder with fluent API
fix(compositor): handle zero-width framebuffers gracefully
docs(readme): update installation instructions
test(animation): add integration tests for frame timing
chore(deps): update fontdue to 0.9.3
```

### Scope

Optional scope that provides contextual information:

- `layer` - Layer implementations (RectLayer, TextLayer, etc.)
- `compositor` - Rendering engine
- `animation` - Animation loop and frame scheduling
- `encoder` - Kitty/Sixel encoding
- `geometry` - Transform and coordinate math
- `cli` - Command line interface
- `deps` - Dependencies

## Reporting Issues

### Bug Reports

When filing a bug report, please include:

1. **Environment information**:
   - Rust version (`rustc --version`)
   - OS and version
   - Feature flags enabled
   - Package version

2. **Steps to reproduce**:
   - Exact commands run
   - Input files (if applicable)
   - Expected behavior
   - Actual behavior

3. **Error messages**:
   - Full error output
   - Backtrace (if available)

### Feature Requests

When requesting a feature, please include:

1. **Problem description**: What problem does this solve?
2. **Proposed solution**: How should it work?
3. **Alternatives considered**: Other approaches
4. **Additional context**: Examples, mockups, etc.

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive experience for everyone.

### Expected Behavior

- Be respectful and inclusive
- Accept constructive criticism
- Focus on what is best for the community
- Show empathy towards others

### Unacceptable Behavior

- Harassment, trolling, or discrimination
- Publishing others' private information
- Other conduct deemed inappropriate

## Questions?

If you have questions about contributing, feel free to:

1. Open a [discussion](https://github.com/216598762/termcompositor/discussions)
2. Comment on an existing issue
3. Reach out to maintainers

Thank you for contributing to termcompositor!
