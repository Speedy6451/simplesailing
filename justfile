build-rust:
    cargo build --manifest-path pirates/Cargo.toml --target wasm32-unknown-unknown --release

minify-rust: build-rust
    wasm-strip pirates/target/wasm32-unknown-unknown/release/pirates.wasm
    wasm-opt -o pirates/target/wasm32-unknown-unknown/release/pirates-opt.wasm -Oz pirates/target/wasm32-unknown-unknown/release/pirates.wasm
    
