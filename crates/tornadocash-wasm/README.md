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

From the repository root, enter the same Nix environment used by CI. It provides
Rust with the WASM target, Node/npm, wasm-bindgen and the WASM test runner.

```sh
nix --extra-experimental-features 'nix-command flakes' develop .#ci
```

Run the following commands inside that shell:

```sh
cargo build --locked --manifest-path crates/Cargo.toml \
  -p kohaku-tornadocash-wasm --target wasm32-unknown-unknown --target-dir crates/target

wasm-bindgen crates/target/wasm32-unknown-unknown/debug/kohaku_tornadocash_wasm.wasm \
  --target nodejs --out-dir crates/target/tornadocash-wasm-node

wasm-bindgen crates/target/wasm32-unknown-unknown/debug/kohaku_tornadocash_wasm.wasm \
  --target web --out-dir crates/target/tornadocash-wasm-web
```

Both targets generate TypeScript declarations. For the browser target, call the
module's default `init()` export before using the classes.

## Package for npm

`npm/package.json` defines the package and its `/node` and `/web` entry points.
`npm/README.md` is the consumer documentation included in the package.
`scripts/package.sh` builds both targets and creates the installable tarball.

From the repository root:

```sh
nix --extra-experimental-features 'nix-command flakes' develop .#ci \
  --command bash crates/tornadocash-wasm/scripts/package.sh
```

The script uses a locked release build and checks that Rust inputs match HEAD.
Outside Nix, it requires Rust with the WASM target, Node/npm, jq and a matching
wasm-bindgen CLI on PATH. `WASM_BINDGEN` can select a CLI executable.

The package directory and `.tgz` are written to `crates/target/npm/`.
`build-info.json` records the Rust source commit, lockfile hash and tool versions.
Install the tarball in a separate project and check Node imports, browser WASM
loading and TypeScript resolution before publishing.

## Validation

Inside the Nix shell above, run the same lint and test commands as CI:

```sh
cargo clippy --locked --manifest-path crates/Cargo.toml \
  -p kohaku-tornadocash-wasm --target wasm32-unknown-unknown \
  --target-dir crates/target --no-deps -- -D warnings

cargo test --locked --manifest-path crates/Cargo.toml \
  -p kohaku-tornadocash-wasm --lib --target-dir crates/target

cargo test --locked --manifest-path crates/Cargo.toml \
  -p kohaku-tornadocash-wasm --lib --target wasm32-unknown-unknown \
  --target-dir crates/target
```

The repository's Cargo configuration selects `wasm-bindgen-test-runner` for WASM
tests. Use `exit` to leave the Nix shell when finished.
