# Current-base replay results

Version 1 — September 16, 2026 UTC (September 15 in Argentina).
Status: completed local replay, no publication. Not a complete SDK port.

## Inputs

- Organization fork: `https://github.com/CoBuilders-xyz/kohaku-rs`.
- Starting upstream/fork `master`: `fb071b3162459be9a5cf9f71ee1ec40a32b76f82`.
- Earlier lab pin: `00d69fa583aeba5b68b5828542557ffdeb8f994d` (nine commits behind).
- Rust 1.98.1, Cargo 1.98.1, Node 26.8.1, wasm-bindgen CLI/crate 0.2.108.
- WASM target: wasm32-unknown-unknown; native: x86_64-unknown-linux-gnu.
- Dev debug info/incremental artifacts disabled; native test profile likewise.
- No dependency versions upgraded by the replay; lockfile changes only add
  direct getrandom/wasmtimer dependency edges. Adapter locks were seeded from
  the original labs; their initial hashes remained identical on the new base.

## Executed outcomes

| Stage | Actual command | Outcome |
| --- | --- | --- |
| Unmodified master | `cargo check --locked --manifest-path crates/Cargo.toml -p kohaku-tornadocash --lib --target wasm32-unknown-unknown --keep-going --target-dir target` with the dev profile environment above | Exit 101: getrandom JS backend guard and Mio platform errors |
| Lab 07 | `bash experiments/compile-wasm/run.sh getrandom` | Both Cargo passes exit 101 at Mio, no getrandom platform guard; runner exit 0 |
| Lab 08 | `bash experiments/compile-wasm/run.sh tokio` | Both Cargo passes exit 101 with 17 Send-related SDK errors; runner exit 0 |
| Lab 09 | `bash experiments/compile-wasm/run.sh futures` | Locked check and build both exit 0 |
| Lab 10 | `bash experiments/note-wasm/run.sh` | Two commitments match native/fixed fixtures; four invalid inputs become JS Errors; valid call after errors passes |
| Lab 11 baseline | `bash experiments/rpc-wasm/run.sh baseline` | Normal early error rejection; first success response reaches clock panic in Tokio; expected-failure controller exit 0 |
| Lab 11 adapted | `bash experiments/rpc-wasm/run.sh adapted` | Three batches decoded; early/late errors reject; native regression suite succeeds |
| Final note regression | `bash experiments/note-wasm/run.sh` after timer adaptation | Same assertions pass on the final cumulative SDK |

Compilation runner uses `check --keep-going` and `build --jobs 1`; both use
`--locked` and target only the Tornado library, not the native fork-kit crate.
No unmodified-master full build or native test run is claimed. Lab 08's
process-only intermediate variant was not replayed.

Initial note run: 00:26:53–00:27:42 UTC. Baseline RPC: 00:29:23–00:29:24.
Adapted RPC plus native suite: 00:30:27–00:31:13. Final note regression:
00:31:30–00:31:34. Runners preserve command logs under ignored
`target/lab-logs/` and experiment `output/`; later runs can overwrite them.
New imports use `note::Note` and `pool::{Asset, Pool}` following the merged
API moves. Public fixtures/assertions were not weakened.

The successful RPC run measured 142.5/110.5 ms between request arrivals for a
configured 100 ms delay, with 32 heartbeat ticks. These are fixture timings,
not performance guarantees. Queries were 100..101, 102..103, 104..105.
Generated RPC types still expose `Promise<string>`, not tsify structured data.

Native default suite: **29 unit tests passed, 0 failed**, across kv-store (6),
merkle-tree (8), Tornado (12), userop-kit (3), and fork-kit (0).
**Two doctests passed**: kv-store executed, Tornado's example was compile-only.
**Seven integration tests remained ignored** (external_sync: 2, pool_provider:
2, provider: 1, sync: 1, tornadocash_paymaster: 1). Their binaries compiled;
their bodies did not run. No Anvil, Alto, circuit download, or funds were used.

## Checks and corrections

Bash syntax, Node syntax, Rust formatting checks, and `git diff --check` passed.
Rustfmt warned that upstream's nightly-only configuration options were ignored
by the stable toolchain; the check returned success. An initial compile
runner invocation returned exit 2 after the runner was edited while executing;
the final file passed `bash -n` and its complete getrandom run was repeated
successfully before committing. Cargo failures in that stage are intentional.

## Final lockfile SHA-256

| File | SHA-256 |
| --- | --- |
| `crates/Cargo.lock` | `9b619b154a2c1d84659774f2fe6e1f7bcf4214e42f93917e0a5a787580a12019` |
| `experiments/note-wasm/Cargo.lock` | `f3721469823b76ce54435ef5aae139945ea9c9fcc2292de2771dec3749da6487` |
| `experiments/rpc-wasm/Cargo.lock` | `0a77bad06320d61b8fe3f9d5cde25a3e6b6a43515293b1642f0b8e3c313bc35c` |

## Limits and next step

No browser, Web Worker, tsify, full Tornado API, storage/reorg runtime path,
live RPC, randomness, proof generation, deposit, or withdrawal coverage is
claimed. Async waiting and a heartbeat are not CPU work offloading. The native
suite is regression evidence, not a cryptographic audit. The open upstream
API-refactor PR #13 was inspected but neither merged nor compiled here.
Issue #7 is not solved or assigned to us by these experiments.

Next: inspect incremental branch diffs with the learner, then choose one
additional wrapper boundary. Do not automatically implement the rest of #7.
Nothing was pushed, no upstream issue was changed, and old lab caches were not
deleted. The single new target cache measured about 2.0 GB after these runs.
