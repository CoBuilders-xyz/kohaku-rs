# Cumulative WASM learning experiments

These local branches replay the experiments from `lwa-kohaku` on upstream
`master` at `fb071b3162459be9a5cf9f71ee1ec40a32b76f82` (checked 2026-09-15).
They are research experiments, not a complete solution to upstream issue #7.
See [executed results and limits](RESULTS.md) on the final timer branch.
Historical lab results at `00d69fa583aeba5b68b5828542557ffdeb8f994d` remain in
the learning repository; do not attribute those results to the new revision.

Use one checkout and its ignored `target/` across branch switches. The Bash
runners log real commands, disable dev debug info/incremental artifacts to
reduce disk use, and keep generated bindings under each experiment's ignored
`output/`. They never reset source files, switch branches, or push changes.
Builds may download Cargo dependencies. Runtime fixtures use no funds.

All work is local to the organization fork. The original upstream is a read-only
reference; its local push URL was disabled. Do not push or contact maintainers
without an explicit user instruction.

## Lab 07: entropy backend

Enable `getrandom` 0.2.17's `js` feature and record its direct dependency in the
SDK lockfile without upgrading packages. This selects a JS entropy backend;
it does not, by itself, make Tornado WASM-compatible or test random generation.

Prerequisites: Rust/Cargo, `wasm32-unknown-unknown`, Bash, Git, and ripgrep.
From this repository's root:

```sh
bash experiments/compile-wasm/run.sh getrandom
```

Exit criterion: both compiler passes fail at Mio, with the getrandom platform
guard absent. The runner exits successfully only for the expected outcome.
Logs are in `target/lab-logs/`. Later branches add further adaptations.

## Lab 08: target-specific Tokio features

Keep `time` available to the library and restrict `process`/`rt-multi-thread`
to non-WASM targets. Native behavior is retained; this is not a replacement
for every Tokio runtime operation on WASM.

```sh
bash experiments/compile-wasm/run.sh tokio
```

Exit criterion: both passes reach the SDK and fail on non-Send async futures.
This branch replays the target-split variant; the historical process-only
variant and its explanation remain in the original Lab 08.

## Lab 09: local futures on WASM

Use `async_trait(?Send)` on wasm32 for the two backend traits and their five
implementations. Native async futures still require Send, and explicit trait
Send/Sync bounds are unchanged. This is a target-specific compile-time contract,
not a promise that every runtime operation works in JavaScript.

```sh
bash experiments/compile-wasm/run.sh futures
```

Exit criterion: locked WASM library check and build succeed. The product is
a Rust library artifact, not yet a JavaScript-callable adapter.

## Lab 10: execute a note operation

The [note adapter](note-wasm/README.md) adds a generated JS/WASM boundary and
native fixture comparison. Run `bash experiments/note-wasm/run.sh`. It uses
current upstream's `note::Note` path rather than the historical provider path.

## Lab 11: RPC runtime boundary

The [RPC adapter](rpc-wasm/README.md) first records the runtime clock failure on
`lab/11-rpc-baseline`. Its succeeding timer branch keeps the same fixture
assertions and selects a WASM-compatible sleep. The baseline uses current
`pool::{Asset, Pool}` imports and does not add another SDK patch.

On `lab/11-rpc-timers`, only the RPC syncer's sleep implementation changes:
native uses Tokio and wasm32 uses wasmtimer 0.4.3. The existing dependency
version is reused; SDK and both adapter lockfiles record the new direct edge.
The default RPC runner now expects successful synchronization and includes
native regression tests. This adaptation does not address every SDK timer.
