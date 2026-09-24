# kohaku-tornadocash-wasm

WebAssembly binding crate for `kohaku-tornadocash`.

This crate depends on the Rust Tornadocash core and `wasm-bindgen`, and builds
both an `rlib` and a `cdylib`.

## Note bindings

Note bindings live in `src/note.rs`; `src/lib.rs` declares the module and
reexports the Rust functions.

```ts
export function note_commitment(note: string): string;
export function note_nullifier_hash(note: string): string;
```

Both functions parse a legacy note with the core's `Note` parser and call
`Note::commitment` or `Note::nullifier_hash`. Each returns `0x` followed by 64
lowercase hexadecimal digits (32 bytes, most significant byte first). They
throw a JavaScript `Error` with the core parser's message for invalid input
and run synchronously.

The commitment depends on the nullifier and secret. The nullifier hash depends
only on the nullifier; neither hash depends on the symbol, amount, or chain ID.
Computing the nullifier hash does not check whether the note has been spent.

Parsing follows the Rust core's rules; it does not trim whitespace or add
validation for the symbol or denomination. Computing a commitment does not
check whether a deposit exists on-chain.

Each adapter consists of:

- `#[wasm_bindgen]`: export the function to JavaScript.
- `note: &str`: accept a JavaScript string as Rust text.
- `Result<String, JsError>`: return a string on success or throw an error in JS.
- `let note: Note = note.parse()?`: use the core parser; `?` converts a parser
  error into `JsError` and returns early.
- `format!("0x{:064x}", note.commitment())`: call the core and format its integer
  as lowercase hex, padded with leading zeros to 64 digits. The nullifier hash
  adapter calls `note.nullifier_hash()` instead.

## Build and run in Node

Prerequisites: the `wasm32-unknown-unknown` Rust target, Node.js with `node:test`,
and the `wasm-bindgen` CLI matching the crate's pinned version, `0.2.108`.
If needed, install the CLI with:

```sh
cargo install wasm-bindgen-cli --version 0.2.108 --locked
```

Run from the repository root:

```sh
cargo build --locked --manifest-path crates/Cargo.toml \
  -p kohaku-tornadocash-wasm --target wasm32-unknown-unknown --target-dir crates/target

wasm-bindgen crates/target/wasm32-unknown-unknown/debug/kohaku_tornadocash_wasm.wasm \
  --target nodejs --out-dir crates/target/tornadocash-wasm-node

TORNADOCASH_WASM_MODULE=crates/target/tornadocash-wasm-node/kohaku_tornadocash_wasm.js \
  node --test crates/tornadocash-wasm/tests/*.cjs
```

The last command checks the generated JS/WASM boundary with synthetic notes:
fixed-width hex output, JavaScript exceptions for invalid input, and nullifier
hash dependence on the nullifier rather than the secret or metadata. It does
not establish parity with the TS SDK or validate browser execution.

The test uses `.cjs` because `--target nodejs` generates a CommonJS module.
It exercises the generated JavaScript API, including thrown `Error` objects.
The core already has Rust tests for note encoding/decoding and a Pedersen hash
vector. New Rust conversion logic should receive Rust tests when introduced;
tests that construct JavaScript values or errors need a WASM runtime, for
example through `wasm-bindgen-test`, rather than ordinary native `cargo test`.

## CI

The `WASM` workflow runs on pull requests targeting `master`, pushes to `master`,
and manual dispatch. It builds the WASM library, generates the Node bindings,
and runs every `tests/*.cjs` file with Node's test runner. A build, generation,
or test failure fails the job. This check runs separately from the native Rust CI.

CI uses Rust 1.98.1, Node 26.8.1, and wasm-bindgen CLI 0.2.108. When updating
the wasm-bindgen dependency, update the CLI version in the workflow as well.

`wasm-bindgen` generates the JS loader and `.d.ts` for these primitive signatures.
This crate does not yet include `tsify`, a public SDK wrapper, or a Worker.
