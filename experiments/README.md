# Cumulative WASM learning experiments

These local branches replay the experiments from `lwa-kohaku` on upstream
`master` at `fb071b3162459be9a5cf9f71ee1ec40a32b76f82` (checked 2026-09-15).
They are research experiments, not a complete solution to upstream issue #7.
Historical lab results at `00d69fa583aeba5b68b5828542557ffdeb8f994d` remain in
the learning repository; do not attribute those results to the new revision.

Use one checkout and its ignored `target/` across branch switches. The Bash
runners log real commands, disable dev debug info/incremental artifacts to
reduce disk use, and keep generated bindings under each experiment's ignored
`output/`. They never reset source files, switch branches, or push changes.
Builds may download Cargo dependencies. Runtime fixtures use no funds.

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
