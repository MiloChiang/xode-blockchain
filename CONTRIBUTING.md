# Contributing to Xode Blockchain

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch
4. Make your changes
5. Submit a PR

## Development Setup

```bash
# Install dependencies
rustup target add wasm32-unknown-unknown

# Build
cargo build --release

# Test
cargo test
```

## Pull Request Process

1. Update documentation
2. Add tests for new features
3. Ensure CI passes
4. Request review

## Code Style

- Follow Rust naming conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` to check for issues
