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

### Typed note parsing

`parse_note` uses the same core parser and returns all five fields of the note
as a plain JavaScript object. The Rust `ParsedNote` struct lives in `src/note.rs`.
`tsify` generates this declaration together with the binding's `.d.ts`:

```ts
export interface ParsedNote {
  symbol: string;
  amount: string;
  chainId: bigint;
  nullifier: Uint8Array;
  secret: Uint8Array;
}

export function parse_note(note: string): ParsedNote;
```

The amount remains the original text. Both byte arrays have length 31 and
preserve the core's byte order. They contain the note's original secrets, not
their hashes. The function throws a JavaScript `Error` if parsing or conversion
to JavaScript fails.

The Rust annotations control the generated type and the runtime conversion:

- `Serialize` converts the struct's fields to JavaScript values.
- `Tsify` generates the TypeScript declaration from the Rust struct.
- `#[serde(rename_all = "camelCase")]` maps `chain_id` to `chainId`.
- `#[tsify(large_number_types_as_bigints)]` makes `u64` a JS/TS `bigint`,
  preserving values above JavaScript's safe integer range.
- `#[serde(with = "serde_bytes")]` serializes each byte array as a `Uint8Array`;
  `#[tsify(type = "Uint8Array")]` describes that representation in TypeScript.
- `Result<Ts<ParsedNote>, JsError>` and `parsed.into_ts()?` perform the fallible
  conversion inside the adapter and expose `ParsedNote` as the return type in TS.

The dependency enables tsify's `js` feature, using `serde-wasm-bindgen` for the
conversion. See [tsify 0.5.8](https://docs.rs/tsify/0.5.8/tsify/). This is an
output type, so it only needs serialization; no structured JS input is accepted.

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
fixed-width hex output, JavaScript exceptions, nullifier-hash behavior, parsed
fields and byte order, and exact `bigint` values across the `u64` range. It does
not establish parity with the TS SDK or validate browser execution.

After generating the bindings, check a TypeScript consumer against the generated
declarations locally:

```sh
npm exec --yes --package=typescript@7.0.2 -- tsc --noEmit --strict \
  --target ES2020 --module Node16 crates/tornadocash-wasm/tests/parse-note.types.ts
```

This compile-only fixture verifies the result type and rejects `number` chain
IDs, ordinary arrays for secret bytes, and the Rust `chain_id` spelling.

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

`wasm-bindgen` generates the JS loader and `.d.ts`, including the `ParsedNote`
declaration supplied by `tsify`. A public SDK wrapper and Worker are future work.
