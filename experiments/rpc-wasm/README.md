# Lab 11 replay: real async RPC in Node/WASM

Executable experiment on the cumulative fork; uses public fabricated events,
loopback HTTP only, no blockchain or funds. The Rust adapter calls the real
`RpcSyncer` via `SyncerBackend`, not a JavaScript syncer implementation.

Prerequisites: Rust/Cargo, wasm32-unknown-unknown, Node 26.8.1 (tested), Bash,
Git, and wasm-bindgen CLI 0.2.108. The runner shares the fork's `target/` and
matching CLI; it can install that version if absent or use `WASM_BINDGEN`.
Cargo/tool downloads may need network access. Resolved dependencies are locked.

## Baseline branch

On `lab/11-rpc-baseline` (before the timer adaptation):

```sh
bash experiments/rpc-wasm/run.sh baseline
```

The adapter returns a Promise. An early fixture RPC error must reject normally;
the first successful HTTP response must lead to the known unsupported clock
panic in Tokio's sleep. The test uses an isolated Node child process to contain
that fatal panic. Exit criterion: the expected failure is detected, not that
baseline synchronization works. A child process is not a Web Worker SDK API.

## Adapted branch

On the subsequent `lab/11-rpc-timers` branch:

```sh
bash experiments/rpc-wasm/run.sh adapted
```

This mode requires three decoded Deposit projections for ranges 100..101,
102..103, and 104..105, a progressing JS heartbeat, observed batch delays,
and normal Promise rejections for errors before/after the first successful
batch. It then runs the SDK's default native workspace tests without `--ignored`.
`native-tests` runs just that regression suite. Running the wrong expectation
against a different SDK state should fail; no mode silently patches the SDK.

The only adapter API-path migration is `provider::pool` to `pool`. Read
`src/lib.rs` for the Promise export, `demo.cjs` for the server/assertions, and
`client.cjs` for isolated callers. Generated code/logs stay under `output/`.

Limits: Node only, no browser/CORS, tsify, Web Workers, cancellation, real RPC,
proofs, withdrawal events, storage/reorg runtime paths, userop polling, full
SDK compatibility, or Rust/TypeScript protocol parity. Neither the watchdog
nor asynchronous I/O demonstrates offloading CPU-heavy work to a worker.
