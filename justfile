set positional-arguments

# List available commands.
default:
    @just --list

# Start the desktop editor.
gui:
    nix develop --command cargo run --features gui --bin offimg-gui

# Build the static browser app into dist/.
web:
    nix develop --command trunk build --release --public-url /

# Convert an image; extra options are passed through, e.g. --fit crop --force.
convert input output *options:
    nix develop --command cargo run --bin offimg -- convert "$@"

# Run the CLI with any arguments, e.g. just cli convert --help.
cli *args:
    nix develop --command cargo run --bin offimg -- "$@"

# Build the standalone CLI at target/debug/offimg, without GUI dependencies.
build:
    nix develop --command cargo build --bin offimg --no-default-features

# Run every test, including the GUI's headless checks.
test:
    nix develop --command cargo test --all-features

# Format the Rust source files.
fmt:
    nix develop --command cargo fmt

# Check formatting without changing files.
fmt-check:
    nix develop --command cargo fmt --check

# Check Rust code for common mistakes, treating warnings as errors.
clippy:
    nix develop --command cargo clippy --all-targets --all-features -- -D warnings

# Run all checks before committing.
check: fmt-check clippy test
