# Contributing to gwatch

First off, thank you for considering contributing to gwatch! It's people like you that make gwatch a great tool.

## Code of Conduct

This project and everyone participating in it is governed by the [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How Can I Contribute?

### Reporting Bugs

- Check if the bug has already been reported in the Issues.
- If not, open a new issue with a clear title and description.
- Include as much relevant information as possible, and a code sample or executable test case demonstrating the expected behavior that is not occurring.

### Suggesting Enhancements

- Open a new issue with the "enhancement" label.
- Describe the feature you'd like to see and why it would be useful.

### Pull Requests

1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. If you've changed APIs, update the documentation.
4. Ensure the test suite passes (`cargo test`).
5. Make sure your code is formatted (`cargo fmt`).
6. Run clippy to ensure no warnings (`cargo clippy -- -D warnings`).

## Development Setup

### Requirements

- Rust (latest stable version recommended)
- Git

### Build and Run

```bash
git clone https://github.com/csaltos/gwatch.git
cd gwatch
cargo build
cargo run -- --help
```

### Running Tests

```bash
cargo test
```

## License

By contributing to gwatch, you agree that your contributions will be licensed under its [MIT License](LICENSE).
