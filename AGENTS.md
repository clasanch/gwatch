# AGENTS.md - Developer Workflow Guide

> This file documents commands and workflows for developers and AI agents.

## Quick Reference

| Action | Command |
|--------|---------|
| Format code | `cargo fmt --all` |
| Check formatting | `cargo fmt --all -- --check` |
| Lint code | `cargo clippy --all-targets -- -D warnings` |
| Run tests | `cargo test --all-features` |
| Check coverage | `cargo llvm-cov --all-features` |
| Security audit | `cargo audit` |
| Dependency check | `cargo deny check` |
| Build release | `cargo build --release` |

## Pre-requisites

Install required tools (one-time setup):

```bash
# Git hook manager
brew install lefthook

# Coverage tool
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview

# Security tools
cargo install cargo-audit
cargo install cargo-deny --locked
```

## Git Hooks Setup

After cloning the repository:

```bash
lefthook install
```

This installs:
- **Pre-commit hook:** Format and lint checks (<5 seconds)
- **Pre-push hook:** Full test suite, 85% coverage, security scans (<5 minutes)

## Development Workflow

### Before Committing
Hooks run automatically, but you can run manually:

```bash
lefthook run pre-commit
```

### Before Pushing
Hooks run automatically, but you can run manually:

```bash
lefthook run pre-push
```

### Coverage Requirements

Minimum coverage: **85%**

Check current coverage:
```bash
cargo llvm-cov --all-features
```

Generate HTML report:
```bash
cargo llvm-cov --all-features --html
open target/llvm-cov/html/index.html
```

## CI/CD Pipeline

The CI pipeline runs on every push and PR:

1. **Format** - `cargo fmt --all --check`
2. **Clippy** - `cargo clippy --all-targets -- -D warnings`
3. **Test** - `cargo test --all-features` (Linux + macOS)
4. **Build** - `cargo build --release` (Linux + macOS)
5. **MSRV** - Check Rust 1.70 compatibility
6. **Audit** - Security vulnerability scan
7. **Coverage** - 85% minimum enforcement

## Code Style

- Maximum line width: 100 characters
- Imports grouped: std → external → crate
- MSRV: Rust 1.70
- No `unsafe` code without justification

## Troubleshooting

### Hook not running
```bash
lefthook install
```

### Coverage below threshold
Add more tests to reach 85% coverage. Check uncovered lines:
```bash
cargo llvm-cov --all-features --html
```

### Clippy warnings
Fix all warnings. Do not use `#[allow(...)]` without justification.
