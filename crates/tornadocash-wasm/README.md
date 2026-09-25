# kohaku-tornadocash-wasm

WebAssembly binding crate for `kohaku-tornadocash`.

This crate depends on the Rust Tornadocash core and `wasm-bindgen`, and builds
both an `rlib` and a `cdylib`.

## Note class

Bindings live in `src/note.rs`; `src/lib.rs` declares the module and reexports
`Note` and `NoteData`. The core type is imported locally as `CoreNote` to
distinguish it from the exported `Note` wrapper.
Each wrapper owns a core Rust `Note` in the loaded WASM instance. Multiple notes
share that instance; creating a note does not start another WASM instance or Worker.

The generated API includes:

```ts
export class Note {
  constructor(data: NoteData);
  static random(symbol: string, amount: string, chainId: bigint): Note;
  static parse(text: string): Note;
  toObject(): NoteData;
  toString(): string;
  preimage(): Uint8Array;
  commitment(): string;
  nullifierHash(): string;
  free(): void;
}

export interface NoteData {
  symbol: string;
  amount: string;
  chainId: bigint;
  nullifier: Uint8Array;
  secret: Uint8Array;
}
```

Use the same object for successive operations, then release it:

```ts
const note = Note.parse(originalText);
try {
  const commitment = note.commitment();
  const nullifierHash = note.nullifierHash();
  const canonicalText = note.toString();
  const preimage = note.preimage();
  const fields = note.toObject();

  const copy = new Note(fields);
  try {
    console.log(copy.toString() === canonicalText);
  } finally {
    copy.free();
  }
} finally {
  note.free();
}
```

All methods run synchronously. `parse` calls the core parser once; the constructor
converts structured input once and calls the core's `Note::new`. Methods borrow
that stored note with `&self`. `free()` releases the Rust object; methods must
not be called afterward. Releasing one note leaves other notes usable.
`wasm-bindgen` also supplies automatic cleanup through `FinalizationRegistry`
where available, but explicit cleanup makes the lifetime predictable.

`toObject()` returns a plain object with independent copies of the fields,
including the original nullifier and secret bytes. Modifying the constructor's
input or an exported object does not mutate the stored note. Each byte array
has length 31 and preserves the core's byte order. The amount remains text;
chain IDs use `bigint` to preserve the entire `u64` range.

`toString()` delegates to the core's `Display` implementation and emits
`tornado-{symbol}-{amount}-{chainId}-0x{preimage}`: lowercase hex containing the
nullifier's 31 bytes followed by the secret's 31 bytes. Parsing and formatting
normalize the hex prefix/case and decimal chain ID spelling.

`preimage()` delegates to the core and returns an independent `Uint8Array` of
62 bytes: nullifier first, then secret. The adapter converts the core's
`[u8; 62]` to `Vec<u8>`, which wasm-bindgen converts to `Uint8Array` and declares
in TypeScript without tsify. Modifying the array does not mutate the note,
and the array remains usable after freeing the note.

Both hash methods return `0x` followed by 64 lowercase hexadecimal digits
(32 bytes, most significant byte first). Commitment depends on the nullifier
and secret; nullifier hash depends only on the nullifier. Neither uses metadata.
These operations do not check on-chain deposits or spent status.

Parsing retains the core's rules, including optional `0x` and no whitespace
trimming. The constructor checks representable Rust types, including 31-byte
arrays and chain IDs in the `u64` range. Conversion and parsing failures throw
JavaScript `Error` objects. No symbol or amount validation is added: arbitrary
strings supplied to the constructor may produce text the parser cannot read back.

### Generating a random note

```ts
const note = Note.random('eth', '0.1', 1n);
try {
  const text = note.toString();
} finally {
  note.free();
}
```

The adapter seeds Rust's `StdRng` from `SysRng` with
`StdRng::try_from_rng(&mut SysRng)?`, then passes that generator to
`CoreNote::random`. The core generates the nullifier and secret; the JS caller
supplies only metadata. The generator is local to each call.

On WASM, `getrandom` uses `globalThis.crypto.getRandomValues()` to obtain the
seed. The crate explicitly enables its `wasm_js` feature. This requires Web
Crypto in the host (browsers and Node.js 19+); if it is unavailable or fails,
the adapter throws a JavaScript `Error`. It does not fall back to an insecure
source. The generator is discarded after constructing the note.

The chain ID crosses the boundary as `js_sys::BigInt` and is checked with
`u64::try_from`: negative values, values above `u64::MAX`, and non-bigint inputs
throw rather than being truncated. The generated TypeScript parameter is
`bigint`. Symbol and amount keep the core's existing behavior.

Runtime tests cover generation, formatting round trips, exact chain IDs, and
missing/failing Web Crypto with recovery afterward. These are integration
checks, not a statistical assessment of the random generator.

### Generated types and conversions

`wasm-bindgen` generates the class, its methods and their declarations.
`NoteData` is the data transfer type generated by tsify, used by the constructor
and `toObject()`:

- `#[wasm_bindgen]` on the `Note` struct and its `impl` exports the class
  and its methods under the same name in Rust and JS.
- `#[wasm_bindgen(constructor)]` exposes Rust `new` as the JS constructor.
- `#[wasm_bindgen(js_name = ...)]` gives methods their JS camelCase names.
- `Serialize` and `Deserialize` support Rust-to-JS and JS-to-Rust data conversion.
- `Tsify` generates the `NoteData` TypeScript interface.
- `#[serde(rename_all = "camelCase")]` maps `chain_id` to `chainId`.
- `#[tsify(large_number_types_as_bigints)]` represents `u64` as `bigint`.
- `#[serde(with = "serde_bytes")]` serializes bytes as `Uint8Array`;
  `#[tsify(type = "Uint8Array")]` declares that representation in TypeScript.
- `Ts<NoteData>` carries typed JS data across the boundary. `to_rust()?` and
  `into_ts()?` perform fallible conversions inside the adapter.

This replaces the earlier free functions (`parse_note`, `format_note`,
`note_commitment`, `note_nullifier_hash`). The Rust core remains unchanged.

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
fields and byte order, exact `bigint` values across the `u64` range, formatting
round trips, invalid structured inputs, independent snapshots, and object lifetimes. It does not establish parity with
the TS SDK or validate browser execution.

After generating the bindings, check a TypeScript consumer against the generated
declarations locally:

```sh
npm exec --yes --package=typescript@7.0.2 -- tsc --noEmit --strict \
  --target ES2020 --lib ES2020,ESNext.Disposable --module Node16 crates/tornadocash-wasm/tests/*.types.ts
```

`ESNext.Disposable` supplies the types for the `[Symbol.dispose]()` method
that wasm-bindgen also generates for the class.

These compile-only fixtures verify result and input types, and reject `number`
chain IDs, ordinary arrays for secret bytes, missing fields, and the Rust
`chain_id` spelling.

The test uses `.cjs` because `--target nodejs` generates a `CommonJS` module.
It exercises the generated JavaScript API, including thrown `Error` objects.
The core already has Rust tests for note encoding/decoding and a Pedersen hash
vector. New Rust conversion logic should receive Rust tests when introduced;
tests that construct JavaScript values or errors need a WASM runtime, for
example through `wasm-bindgen-test`, rather than ordinary native `cargo test`.

## CI

The `WASM` workflow runs on pull requests targeting `master`, pushes to `master`,
and manual dispatch. It builds the WASM library, generates the Node bindings,
and runs every `tests/*.cjs` file with Node's test runner. It then checks all
`tests/*.types.ts` consumers against the generated declarations with
`tsc --noEmit --strict`. A build, generation, runtime test, or type-check failure
fails the job. This check runs separately from the native Rust CI.

CI uses Rust 1.98.1, Node 26.8.1, wasm-bindgen CLI 0.2.108, and TypeScript 7.0.2.
When updating the wasm-bindgen dependency, update the CLI version in the workflow
as well.

`wasm-bindgen` generates the JS loader and `.d.ts`, including the `NoteData`
declaration supplied by `tsify`. A public SDK wrapper and Worker are future work.
