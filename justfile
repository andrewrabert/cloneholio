set default-list := true

# Build the crates
build:
    @cargo build --manifest-path {{justfile_directory()}}/Cargo.toml

# Run the tests
test:
    @cargo test --manifest-path {{justfile_directory()}}/Cargo.toml --workspace

# Format the sources
fmt:
    @cargo fmt --all --manifest-path {{justfile_directory()}}/Cargo.toml

# Verify formatting without writing
fmt-check:
    @cargo fmt --all --manifest-path {{justfile_directory()}}/Cargo.toml --check

# Lint with clippy (warnings are errors)
lint:
    @cargo clippy --manifest-path {{justfile_directory()}}/Cargo.toml --workspace --all-targets -- -D warnings

# Run all static checks + tests
check: fmt-check lint test

# List outdated dependencies
outdated:
    @cargo outdated --manifest-path {{justfile_directory()}}/Cargo.toml --workspace --root-deps-only

# Update dependencies
[positional-arguments]
update *args:
    @cargo update --manifest-path {{justfile_directory()}}/Cargo.toml "$@"

# Run the binary
[no-exit-message]
[positional-arguments]
run *args:
    @cargo run --manifest-path {{justfile_directory()}}/Cargo.toml --quiet -- "$@"
