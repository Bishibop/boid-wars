# Contributing to Boid Wars

Thank you for your interest in contributing to Boid Wars! This document provides guidelines and instructions for contributing to the project.

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Process](#development-process)
- [Code Style](#code-style)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive criticism
- Respect differing viewpoints and experiences

## Getting Started

### Prerequisites

See [SETUP_GUIDE.md](docs/development/SETUP_GUIDE.md) for detailed setup instructions.

Quick version:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required tools
rustup target add wasm32-unknown-unknown
cargo install wasm-pack trunk cargo-watch

# Clone and setup
git clone https://github.com/yourusername/boid_wars.git
cd boid_wars
make setup
```

### First Time Contributors

Good first issues are labeled with `good-first-issue`. These typically involve:
- Documentation improvements
- Simple bug fixes
- Adding tests
- Code cleanup

Before starting work:
1. Check if someone is already working on the issue
2. Comment that you'd like to work on it
3. Ask questions if anything is unclear

## Development Process

### 1. Fork and Branch

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/yourusername/boid_wars.git
cd boid_wars
git remote add upstream https://github.com/original/boid_wars.git

# Create a feature branch
git checkout -b feature/your-feature-name
```

### 2. Make Changes

Follow the development workflow in [DEVELOPMENT.md](docs/development/DEVELOPMENT.md).

```bash
# Run development environment
make dev

# Make your changes, then check code quality
make check
```

### 3. Commit Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add boid flocking behavior
fix: correct projectile collision detection  
docs: update setup instructions
test: add tests for spatial indexing
refactor: simplify network message handling
perf: optimize entity queries
chore: update dependencies
```

Commit message format:
```
<type>(<scope>): <subject>

<body>

<footer>
```

Example:
```
feat(boids): implement separation behavior

Add separation force calculation to prevent boid overlap.
Uses spatial indexing for efficient neighbor queries.

Closes #123
```

### 4. Keep Your Fork Updated

```bash
git fetch upstream
git checkout main
git merge upstream/main
git push origin main
```

## Code Style

### Rust

Follow Rust best practices as enforced by rustfmt and clippy:

```bash
# Format code
cargo fmt --all

# Check for issues
cargo clippy --all -- -D warnings
```

Key conventions:
- Use descriptive variable names
- Document public APIs
- Prefer iterators over loops
- Handle errors explicitly
- Follow [CODING_STANDARDS.md](docs/development/CODING_STANDARDS.md)

### Performance Considerations

Remember our performance targets:
- 10,000+ entities at 60 FPS
- Sub-150ms network latency

Before optimizing:
1. Profile first
2. Document performance improvements
3. Add benchmarks for critical paths

## Testing

### Running Tests

```bash
# All tests
make test

# Specific test
cargo test test_boid_movement -- --nocapture

# With coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Arrange
        let input = create_test_data();
        
        // Act
        let result = process(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

Test guidelines:
- Test behavior, not implementation
- Use descriptive test names
- One assertion per test when possible
- Mock external dependencies

## Submitting Changes

### Pull Request Process

1. **Update your branch**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run checks**:
   ```bash
   make check
   ```

3. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Create Pull Request**:
   - Use a clear, descriptive title
   - Reference any related issues
   - Describe what changed and why
   - Include screenshots for UI changes
   - Note any breaking changes

### PR Template

```markdown
## Description
Brief description of changes

## Related Issues
Closes #123

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Tests pass locally
- [ ] Added new tests
- [ ] Tested on Chrome/Firefox

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-reviewed code
- [ ] Updated documentation
- [ ] No console warnings
```

### Review Process

- PRs require at least one approval
- Address review feedback promptly
- Keep PRs focused and small when possible
- Be patient - reviewers are volunteers

## Reporting Issues

### Bug Reports

Include:
- Clear description
- Steps to reproduce
- Expected vs actual behavior
- Browser and OS information
- Console errors
- Screenshots if applicable

### Feature Requests

Include:
- Problem description
- Proposed solution
- Alternative solutions considered
- Implementation thoughts

## Development Tips

### Debugging

- Use browser DevTools (F12)
- Enable Bevy diagnostics with `--features debug`
- Check network traffic for multiplayer issues
- See [DEVELOPMENT.md#debugging](docs/development/DEVELOPMENT.md#debugging)

### Performance

- Profile before optimizing
- Use Chrome DevTools Performance tab
- Monitor entity count and FPS
- Test with 10k+ entities

### Common Issues

- **Certificate errors**: Use `make dev` (WebSocket mode)
- **WASM build fails**: Clear cache with `rm -rf target/wasm32-unknown-unknown`
- **Port conflicts**: Check what's using ports 5001/8080

## Getting Help

- Check existing issues and PRs
- Read the documentation in `docs/`
- Ask in issue comments
- Be specific about your problem

## Recognition

Contributors are recognized in:
- GitHub contributors page
- Release notes
- Special thanks in README

Thank you for contributing to Boid Wars! 🚀