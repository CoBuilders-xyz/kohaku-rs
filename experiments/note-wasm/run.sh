#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
lab_dir="$PWD"
repo_dir=$(git rev-parse --show-toplevel)
if [[ $# != 0 ]]; then printf 'Usage: bash experiments/note-wasm/run.sh\n' >&2; exit 2; fi
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_INCREMENTAL=false
bindgen="${WASM_BINDGEN:-$repo_dir/target/tools/bin/wasm-bindgen}"
mkdir -p "$lab_dir/output"
section() { printf '\n--- %s ---\n' "$1"; }
run() { printf '\n$ %s\n' "$*" >&2; "$@"; }
(
    date -u '+Started: %Y-%m-%dT%H:%M:%SZ'
    run git rev-parse HEAD
    run git status --short
    run rustc --version
    run node --version
    section '1. Matching binding generator (shared across experiments)'
    if [[ ! -e "$bindgen" && -z "${WASM_BINDGEN:-}" ]]; then
        run cargo install wasm-bindgen-cli --version 0.2.108 --locked \
            --bin wasm-bindgen --root "$repo_dir/target/tools" \
            --target-dir "$repo_dir/target/tool-build"
    fi
    [[ "$("$bindgen" --version)" == 'wasm-bindgen 0.2.108' ]] || exit 1
    section '2. Build the adapter using this checkout of Tornado'
    run cargo build --locked --lib --target wasm32-unknown-unknown --target-dir "$repo_dir/target"
    run "$bindgen" --target nodejs --out-dir "$lab_dir/output/pkg" \
        --out-name kohaku_note "$repo_dir/target/wasm32-unknown-unknown/debug/kohaku_note_wasm_lab.wasm"
    section '3. Execute native reference and real Node/WASM assertions'
    run cargo run --locked --bin native-reference --target-dir "$repo_dir/target" \
        | tee "$lab_dir/output/native-reference.json"
    run node "$lab_dir/demo.cjs"
    date -u '+Finished: %Y-%m-%dT%H:%M:%SZ'
) 2>&1 | tee "$lab_dir/output/run.log"
