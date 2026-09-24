# kohaku-tornadocash-wasm

WebAssembly binding crate for `kohaku-tornadocash`.

This is a workspace scaffold. It depends on the Rust Tornadocash core and
`wasm-bindgen`, and is configured to build both an `rlib` and a `cdylib`.
No functions or types are exported to JavaScript yet.

From the `kohaku-rs` repository root, validate the scaffold with:

```sh
cargo check --locked --offline --manifest-path crates/Cargo.toml \
  -p kohaku-tornadocash-wasm --target wasm32-unknown-unknown --target-dir target
```

This checks the workspace member and its dependencies. A WASM build also
succeeds, but the resulting module does not contain an application API yet.
