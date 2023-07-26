alias c := clippy
alias f := fmt

default: fmt clippy build_native

fmt:
    cargo fmt

clippy:
    cargo clippy

build_native:
    RUSTFLAGS='-C target-cpu=native' cargo build --release
