#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
lab_dir="$PWD"
repo_dir=$(git rev-parse --show-toplevel)
mode="${1:-adapted}"
if [[ $# -gt 1 ]]; then printf 'Expected at most one mode.\n' >&2; exit 2; fi
case "$mode" in
    baseline|adapted|native-tests) ;;
    *) printf 'Usage: bash experiments/rpc-wasm/run.sh [baseline|adapted|native-tests]\n' >&2; exit 2 ;;
esac
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_INCREMENTAL=false
export CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_TEST_INCREMENTAL=false
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
    if [[ "$mode" != native-tests ]]; then
        section '1. Matching shared binding generator'
        if [[ ! -e "$bindgen" && -z "${WASM_BINDGEN:-}" ]]; then
            run cargo install wasm-bindgen-cli --version 0.2.108 --locked \
                --bin wasm-bindgen --root "$repo_dir/target/tools" \
                --target-dir "$repo_dir/target/tool-build"
        fi
        [[ "$("$bindgen" --version)" == 'wasm-bindgen 0.2.108' ]] || exit 1
        section '2. Build this checkout and generate Node bindings'
        run cargo build --locked --lib --target wasm32-unknown-unknown --target-dir "$repo_dir/target"
        run "$bindgen" --target nodejs --out-dir "$lab_dir/output/$mode/pkg" \
            --out-name kohaku_rpc "$repo_dir/target/wasm32-unknown-unknown/debug/kohaku_rpc_wasm_lab.wasm"
        section "3. Execute $mode assertions against loopback HTTP"
        run node "$lab_dir/demo.cjs" "$mode"
    fi
    if [[ "$mode" != baseline ]]; then
        section '4. Default native workspace suite (ignored tests stay ignored)'
        run cargo test --locked --manifest-path "$repo_dir/crates/Cargo.toml" \
            --workspace --target-dir "$repo_dir/target"
    fi
    date -u '+Finished: %Y-%m-%dT%H:%M:%SZ'
) 2>&1 | tee "$lab_dir/output/$mode.log"
