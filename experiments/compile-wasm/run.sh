#!/usr/bin/env bash
# Compare the compiler boundary at each cumulative lab branch.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.."
repo_dir="$PWD"
stage="${1:-getrandom}"
if [[ $# -gt 1 ]]; then printf 'Expected at most one stage argument.\n' >&2; exit 2; fi
case "$stage" in
    baseline|getrandom|tokio|futures) ;;
    *) printf 'Usage: bash experiments/compile-wasm/run.sh [baseline|getrandom|tokio|futures]\n' >&2; exit 2 ;;
esac
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_INCREMENTAL=false
mkdir -p target/lab-logs
section() { printf '\n--- %s ---\n' "$1"; }
run() { printf '\n$ %s\n' "$*"; "$@"; }

for pass in check build; do
    log="$repo_dir/target/lab-logs/$stage-$pass.log"
    section "$stage: $pass"
    git rev-parse HEAD
    options=()
    if [[ "$pass" == check ]]; then options+=(--keep-going); else options+=(--jobs 1); fi
    set +e
    run cargo "$pass" --locked --manifest-path crates/Cargo.toml \
        -p kohaku-tornadocash --lib --target wasm32-unknown-unknown \
        --target-dir "$repo_dir/target" "${options[@]}" 2>&1 | tee "$log"
    statuses=("${PIPESTATUS[@]}")
    result=${statuses[0]}
    set -e
    [[ "${statuses[1]}" == 0 ]] || { printf 'Could not write the build log.\n' >&2; exit 1; }
    if [[ "$stage" == futures ]]; then
        [[ "$result" == 0 ]] || exit "$result"
    else
        [[ "$result" == 101 ]] || { printf 'Expected Cargo compilation failure (101), got %s\n' "$result"; exit 1; }
        case "$stage" in
            baseline) rg -q 'targets are not supported by default' "$log" ;;
            getrandom) rg -q 'mio' "$log"; ! rg -q 'targets are not supported by default' "$log" ;;
            tokio) rg -q 'future cannot be sent between threads safely' "$log" ;;
        esac
    fi
    printf '[check] Expected %s outcome confirmed (Cargo exit %s).\n' "$stage" "$result"
done
