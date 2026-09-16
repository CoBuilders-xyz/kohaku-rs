# Lab 10 replay: a note through Node/WASM

Executable experiment. Source base and compile adaptations are documented in
`../README.md`. This branch adds an adapter, not another SDK implementation.

## Prerequisites and execution

Rust/Cargo with `wasm32-unknown-unknown`, Node with CommonJS support, Bash, Git.
All runs recorded during migration used Rust 1.98.1 and Node 26.8.1.

```sh
bash experiments/note-wasm/run.sh
```

The runner reuses `target/tools/bin/wasm-bindgen` (exactly 0.2.108), or installs
that version there if missing. `WASM_BINDGEN` may point to an existing matching
binary. Dependency/tool downloads need network access; the fixture execution
does not. SDK dependency paths refer to this checkout; lockfiles pin resolved
dependencies. No per-lab clone or tool compilation is needed if already cached.

## Read the experiment

1. `src/lib.rs` parses a note using the real SDK and exports its commitment.
2. wasm-bindgen generates a Node loader and `.d.ts` in ignored `output/pkg/`.
3. `src/bin/native-reference.rs` runs the same SDK directly with public fixtures.
4. `demo.cjs` compares both hashes, exercises four malformed notes, and calls
   the function again after errors.

The migration changes the old `provider::note::Note` import to `note::Note`,
matching current upstream. Assertions and public fixture values are unchanged.

Exit criterion: both commitments match native execution and fixed public
fixtures; invalid inputs produce JS Error objects rather than WASM traps.
Limits: one synchronous operation, not all Tornado, tsify, a browser, worker,
randomness, proof generation, transactions, or a published SDK. Execution logs
are in `output/run.log`; the migration report records actual outcomes.
