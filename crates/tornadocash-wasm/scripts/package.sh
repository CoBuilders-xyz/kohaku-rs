#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
crate_dir=$(cd -- "$script_dir/.." && pwd)
repo_dir=$(cd -- "$crate_dir/../.." && pwd)
target_dir="$repo_dir/crates/target"
package_dir="$target_dir/npm/tornadocash-wasm"
tarball_dir="$target_dir/npm"
bindgen=${WASM_BINDGEN:-wasm-bindgen}
cd -- "$repo_dir"

command -v jq >/dev/null
expected=$(sed -n 's/^wasm-bindgen *= *"=\([^"]*\)".*/\1/p' crates/Cargo.toml)
actual=$("$bindgen" --version | awk '{print $2}')
if [[ -z "$expected" || "$actual" != "$expected" ]]; then
    printf 'wasm-bindgen CLI must be %s; found %s\n' "$expected" "$actual" >&2
    exit 1
fi

# Rust inputs must match the recorded commit while packaging files can be edited.
git diff --quiet HEAD -- crates .cargo \
    ':!crates/tornadocash-wasm/npm' \
    ':!crates/tornadocash-wasm/scripts/package.sh'
source_commit=$(git rev-parse HEAD)

cargo build --locked --release --manifest-path crates/Cargo.toml \
    -p kohaku-tornadocash-wasm --target wasm32-unknown-unknown \
    --target-dir "$target_dir"

rm -rf -- "$package_dir"
mkdir -p -- "$package_dir"
wasm="$target_dir/wasm32-unknown-unknown/release/kohaku_tornadocash_wasm.wasm"
"$bindgen" "$wasm" --target nodejs --out-dir "$package_dir/node"
"$bindgen" "$wasm" --target web --out-dir "$package_dir/web"
printf '{"type":"commonjs"}\n' > "$package_dir/node/package.json"
printf '{"type":"module"}\n' > "$package_dir/web/package.json"
cp -- "$crate_dir/npm/package.json" "$crate_dir/npm/README.md" "$package_dir/"

jq -n \
    --arg sourceCommit "$source_commit" \
    --arg rustTree "$(git rev-parse HEAD:crates)" \
    --arg cargoLockSha256 "$(sha256sum crates/Cargo.lock | awk '{print $1}')" \
    --arg rustc "$(rustc --version)" \
    --arg wasmBindgen "$actual" \
    --arg node "$(node --version)" \
    '{sourceRepository: "https://github.com/CoBuilders-xyz/kohaku-rs",
      sourceCommit: $sourceCommit, rustTree: $rustTree,
      cargoLockSha256: $cargoLockSha256, rustc: $rustc,
      wasmBindgen: $wasmBindgen, node: $node}' > "$package_dir/build-info.json"

cd -- "$package_dir"
npm pack --ignore-scripts --pack-destination "$tarball_dir"
printf '\nPackage directory: %s\nTarball directory: %s\n' "$package_dir" "$tarball_dir"
