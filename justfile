[private]
default:
    @just --list

# Run cloneholio with the given arguments
run *args:
    uv run cloneholio {{args}}

# Install dependencies
[group('dev')]
setup:
    uv sync

# Build distribution packages
[group('dev')]
build: setup
    uv build

# Remove build artifacts
[group('dev')]
clean:
    rm -rf .venv dist

# Check linting and formatting without modifying files
[group('check')]
lint: setup
    uv run ruff check
    uv run ruff format --check

# Fix lint issues and format the code
[group('check')]
format: setup
    uv run ruff check --fix
    uv run ruff format

# Run the test suite
[group('check')]
test: setup
    uv run pytest
