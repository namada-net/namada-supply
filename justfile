fmt:
    cargo fmt

fmt-check:
    cargo fmt --all --check
    
clippy:
    cargo clippy

clippy-fix:
    cargo clippy --all --fix --allow-dirty --allow-staged

check:
    cargo check

run RPC:
    cargo run --release -- --rpc {{RPC}}
