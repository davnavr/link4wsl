alias c := clippy
alias f := fmt

default: fmt clippy

fmt:
    cargo fmt

clippy:
    cargo clippy
