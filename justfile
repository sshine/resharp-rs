# Default recipe
default:
    @just --list

# Format all code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
test:
    cargo test --all-features

# Run tests with verbose output
test-verbose:
    cargo test --all-features -- --nocapture

# Update snapshot tests
snap:
    cargo insta test --review

# Build release
build:
    cargo build --release --all-features

# Generate documentation
doc:
    cargo doc --no-deps --all-features --open

# Run CI checks locally
ci: fmt-check lint test doc build
    @echo "All CI checks passed!"

# Watch for changes and run tests
watch:
    cargo watch -x test

# Clean build artifacts
clean:
    cargo clean
