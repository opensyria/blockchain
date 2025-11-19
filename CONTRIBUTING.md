# Contributing to OpenSyria Blockchain

**مرحباً بك في مشروع بلوكتشين سوريا المفتوحة!**

We welcome contributions from developers, blockchain enthusiasts, and members of the Syrian community worldwide. This guide will help you get started.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [How Can I Contribute?](#how-can-i-contribute)
3. [Getting Started](#getting-started)
4. [Development Workflow](#development-workflow)
5. [Coding Standards](#coding-standards)
6. [Testing Guidelines](#testing-guidelines)
7. [Documentation](#documentation)
8. [Pull Request Process](#pull-request-process)
9. [Community](#community)

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of:
- Background, nationality, or ethnicity
- Gender identity or expression
- Level of experience
- Political or religious beliefs
- Geographic location (inside or outside Syria)

### Expected Behavior

- **Be Respectful**: Treat everyone with respect and courtesy
- **Be Collaborative**: Work together towards shared goals
- **Be Professional**: Keep discussions focused on technical merit
- **Be Inclusive**: Welcome newcomers and help them learn
- **Be Patient**: Remember that we all started somewhere

### Unacceptable Behavior

- Harassment, discrimination, or hate speech of any kind
- Trolling, inflammatory comments, or personal attacks
- Publishing others' private information
- Any conduct that could be considered unprofessional

---

## How Can I Contribute?

### 🐛 Reporting Bugs

**Before submitting a bug report:**
1. Check existing issues to avoid duplicates
2. Collect relevant information (OS, Rust version, error messages)
3. Create a minimal reproduction case

**Bug Report Template:**
```markdown
**Description:**
A clear description of the bug.

**Steps to Reproduce:**
1. Run command `...`
2. Observe error `...`

**Expected Behavior:**
What should happen.

**Actual Behavior:**
What actually happens.

**Environment:**
- OS: [e.g., macOS 14.0, Ubuntu 22.04]
- Rust version: [output of `rustc --version`]
- OpenSyria version: [output of `git rev-parse HEAD`]

**Error Output:**
```
Paste error messages here
```
```

### 💡 Suggesting Features

**Feature Request Template:**
```markdown
**Feature Description:**
Clear description of the proposed feature.

**Use Case:**
Why is this feature needed? Who benefits?

**Proposed Implementation:**
Technical approach (optional).

**Alternatives Considered:**
Other solutions you've thought about.
```

### 📝 Improving Documentation

Documentation improvements are always welcome:
- Fix typos or unclear explanations
- Add examples and tutorials
- Improve Arabic translations
- Document undocumented features
- Create video tutorials or diagrams

### 🔧 Code Contributions

Areas where we need help:
- **Core Blockchain**: Performance optimizations, validation logic
- **Networking**: P2P protocol implementation, peer discovery
- **Cultural Identity**: Heritage verification, IPFS integration
- **Governance**: Voting mechanisms, proposal types
- **Wallet**: GUI development, hardware wallet support
- **Explorer**: UI/UX improvements, real-time updates
- **Testing**: Unit tests, integration tests, fuzzing
- **Documentation**: API docs, tutorials, translations

---

## Getting Started

### 1. Fork the Repository

```bash
# Fork on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/opensyria.git
cd opensyria

# Add upstream remote
git remote add upstream https://github.com/OpenSyria/blockchain.git
```

### 2. Set Up Development Environment

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
rustup update stable
rustup component add rustfmt clippy

# Build the project
cargo build --all

# Run tests to verify setup
cargo test --all
```

### 3. Create a Feature Branch

```bash
# Update your fork
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/my-new-feature
```

---

## Development Workflow

### 1. Make Your Changes

**Best Practices:**
- Write clear, self-documenting code
- Add comments for complex logic
- Follow existing code style
- Keep commits focused and atomic
- Write descriptive commit messages

**Commit Message Format:**
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Build process, dependencies, tooling

**Examples:**
```bash
git commit -m "feat(identity): add IPFS metadata retrieval"
git commit -m "fix(consensus): correct difficulty adjustment calculation"
git commit -m "docs(governance): add voting examples"
```

### 2. Keep Your Branch Updated

```bash
# Regularly sync with upstream
git fetch upstream
git rebase upstream/main
```

### 3. Build and Test

```bash
# Build all crates
cargo build --all

# Run tests
cargo test --all

# Run specific crate tests
cargo test -p opensyria-core

# Check code formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all -- -D warnings
```

---

## Coding Standards

### Rust Style Guide

We follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/).

**Key Points:**
- Use `rustfmt` for automatic formatting
- Maximum line length: 100 characters
- Use descriptive variable names
- Prefer explicit over implicit
- Document public APIs with `///` comments

**Example:**
```rust
/// Creates a new cultural identity token.
///
/// # Arguments
/// * `id` - Unique token identifier
/// * `owner` - Public key of the token owner
/// * `token_type` - Type of heritage being represented
///
/// # Returns
/// A new `IdentityToken` instance
///
/// # Example
/// ```
/// let token = IdentityToken::new(
///     "damascus-steel-001".to_string(),
///     owner_pubkey,
///     TokenType::TraditionalCraft,
/// );
/// ```
pub fn new(id: String, owner: PublicKey, token_type: TokenType) -> Self {
    // Implementation
}
```

### Error Handling

- Use `Result<T, E>` for fallible operations
- Create custom error types for domain-specific errors
- Provide helpful error messages
- Use `?` operator for error propagation

```rust
// Good
pub fn validate_transaction(&self) -> Result<(), TransactionError> {
    if self.amount == 0 {
        return Err(TransactionError::InvalidAmount("Amount must be positive".into()));
    }
    Ok(())
}

// Avoid
pub fn validate_transaction(&self) -> bool {
    self.amount != 0  // No context about why validation failed
}
```

### Testing

- Write unit tests for individual functions
- Write integration tests for cross-module behavior
- Test edge cases and error conditions
- Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_identity_token_with_valid_data() {
        let token = IdentityToken::new(
            "test-001".to_string(),
            test_pubkey(),
            TokenType::HeritageSite,
        );
        assert_eq!(token.id, "test-001");
    }

    #[test]
    fn test_reject_empty_token_id() {
        let result = IdentityToken::validate_id("");
        assert!(result.is_err());
    }
}
```

---

## Testing Guidelines

### Running Tests

```bash
# Run all tests
cargo test --all

# Run with output
cargo test --all -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific crate
cargo test -p opensyria-core
```

### Integration Tests

```bash
# Run integration test scripts
./scripts/test-network.sh      # Multi-node P2P testing
./scripts/test-multisig.sh     # Multi-signature accounts
./scripts/test-pool.sh         # Mining pool operations
./scripts/test-ipfs.sh         # Heritage content storage
./scripts/test-wallet-api.sh   # REST API endpoints
./scripts/test-daemon.sh       # Network daemon mode
```

### Test Coverage

We aim for:
- **Core modules**: 80%+ coverage
- **Storage layer**: 90%+ coverage
- **Consensus logic**: 95%+ coverage
- **Critical security code**: 100% coverage

---

## Documentation

### Code Documentation

- Document all public APIs with `///` doc comments
- Include examples in documentation
- Keep documentation up to date with code changes
- Run `cargo doc` to verify documentation builds

```bash
# Generate and view documentation
cargo doc --all --no-deps --open
```

### User Documentation

Located in `docs/`:
- **ARCHITECTURE.md**: System design and architecture
- **DEPLOYMENT.md**: Installation and deployment
- **identity/**: Cultural identity system docs
- **network/**: P2P networking documentation
- **governance/**: Governance system guide
- **api/**: REST API reference

### Bilingual Support

- Core documentation should include Arabic translations where appropriate
- CLI help text should be bilingual (Arabic/English)
- Error messages for end users should be bilingual
- Technical documentation can be primarily English

---

## Pull Request Process

### 1. Prepare Your PR

**Before submitting:**
- [ ] All tests pass (`cargo test --all`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] No clippy warnings (`cargo clippy --all -- -D warnings`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (if applicable)
- [ ] Commits are clean and well-organized

### 2. Create Pull Request

**PR Title Format:**
```
<type>(<scope>): <description>
```

**PR Description Template:**
```markdown
## Description
Brief description of changes.

## Motivation
Why is this change needed?

## Changes Made
- Added feature X
- Fixed bug Y
- Updated documentation Z

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] CHANGELOG.md updated
- [ ] All tests passing
- [ ] No clippy warnings

## Related Issues
Fixes #123
Relates to #456
```

### 3. Review Process

- Maintainers will review your PR within 48-72 hours
- Address review comments by pushing new commits
- Once approved, maintainers will merge your PR
- Your contribution will be credited in CHANGELOG.md

### 4. After Merge

- Delete your feature branch
- Update your fork
- Check the project roadmap for next contributions

---

## Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Discord** (coming soon): Real-time chat
- **Telegram** (coming soon): Arabic-speaking community

### Getting Help

**For development questions:**
1. Check existing documentation
2. Search GitHub issues
3. Ask in GitHub Discussions
4. Join community chat

**For security issues:**
- **DO NOT** open public issues
- Email: opensyria.net@gmail.com
- Use GPG encryption if possible

---

## Recognition

### Contributors

All contributors will be:
- Listed in CHANGELOG.md for their contributions
- Credited in release notes
- Added to CONTRIBUTORS.md (coming soon)
- Recognized in community channels

### Types of Contributions

We value all contributions equally:
- Code contributions
- Documentation improvements
- Bug reports and testing
- Community support
- Translations
- Design and UX
- Outreach and education

---

## Additional Resources

### Learning Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)

### Blockchain Development
- [Mastering Bitcoin](https://github.com/bitcoinbook/bitcoinbook)
- [Mastering Ethereum](https://github.com/ethereumbook/ethereumbook)
- [Substrate Documentation](https://docs.substrate.io/)

### Project Documentation
- [Architecture Guide](docs/ARCHITECTURE.md)
- [Deployment Guide](docs/DEPLOYMENT.md)
- [API Documentation](docs/api/)
- [Testing Guide](docs/tests/INTEGRATION_TESTS.md)

---

## License

By contributing to OpenSyria, you agree that your contributions will be licensed under the project's MIT OR Apache-2.0 dual license.

---

## Thank You! شكراً جزيلاً!

Your contributions help preserve Syrian culture and build a sovereign digital future. Every line of code, every bug report, every documentation improvement makes a difference.

**Together, we're building more than a blockchain — we're preserving a civilization.**

**معاً، نبني أكثر من مجرد بلوكتشين — نحن نحافظ على حضارة**
