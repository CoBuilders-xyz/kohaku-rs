# kohaku-tornadocash-wasm

JavaScript/TypeScript bindings for `kohaku-tornadocash`. This crate houses the
bindings for the core's public API, currently covering notes. The wrappers follow
the Rust API; wasm-bindgen and tsify stay in this crate.

## Available bindings

### Notes

- `new Note(nullifier, secret)` creates a note from two 31-byte hex strings.
- `new NoteString(note, symbol, amount, chainId)` adds metadata.
- `NoteString.parse(text)` parses a Tornado note; `toString()` formats it.
- Both classes expose read-only `nullifier` and `secret` getters, plus
  `preimage()`, `commitment()` and `nullifierHash()`.
- `NoteString` also exposes read-only `symbol`, `amount` and `chainId` getters.

Bytes use `0x`-prefixed hex strings, typed as `Hex` in TypeScript. Outputs use lowercase hex.
`chainId` is a `bigint` in the `u64` range. Conversion and parsing errors throw
JavaScript `Error`s. All operations are synchronous.

`NoteString` consumes its input `Note`, even if chain ID validation fails.
Use the resulting `NoteString` afterward. Call `free()` on owned wrappers when
finished; consumed or freed wrappers must not be reused.

Example with fixed inputs, after generating the Node bindings:

```js
const { Note, NoteString } = require(
  './crates/target/tornadocash-wasm-node/kohaku_tornadocash_wasm.js',
);

const note = new Note(`0x${'01'.repeat(31)}`, `0x${'02'.repeat(31)}`);
const noteString = new NoteString(note, 'eth', '0.1', 1n);
const parsed = NoteString.parse(noteString.toString());

console.log(parsed.commitment());
parsed.free();
noteString.free();
```

## Build

Run from the repository root. The wasm-bindgen CLI version must match the Rust dependency.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.108 --locked --bin wasm-bindgen

cargo build --locked --manifest-path crates/Cargo.toml \
  -p kohaku-tornadocash-wasm --target wasm32-unknown-unknown --target-dir crates/target

wasm-bindgen crates/target/wasm32-unknown-unknown/debug/kohaku_tornadocash_wasm.wasm \
  --target nodejs --out-dir crates/target/tornadocash-wasm-node

wasm-bindgen crates/target/wasm32-unknown-unknown/debug/kohaku_tornadocash_wasm.wasm \
  --target web --out-dir crates/target/tornadocash-wasm-web
```

Both targets generate TypeScript declarations. For the browser target, call the
module's default `init()` export before using the classes.

## Validation

CI builds WASM, runs strict Clippy, tests native conversions and WASM bindings,
and generates Node/browser bindings.

```sh
cargo clippy --locked --manifest-path crates/Cargo.toml \
  -p kohaku-tornadocash-wasm --target wasm32-unknown-unknown \
  --target-dir crates/target --no-deps -- -D warnings
```
