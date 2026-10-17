# Offline relayer quotes

`RelayerStatus.parse(json)` uses the canonical Rust serde schema. Pass the original
JSON response text so JavaScript does not round integers before Rust receives them.
The status exposes exact prices as `Map<string, bigint>`, typed instance maps and
health metadata (`undefined` for absent values). Returned metadata is detached.

`supports(pool)` and `quote(pool, gasPrice, refund)` are synchronous core operations
that borrow the pool. Gas price is a `u128` bigint in native base units per gas;
refund is a `U256` bigint in native base units. Invalid inputs and core quote errors
throw, including unsupported pools, zero ERC20 prices and arithmetic overflow.
An unrelated zero price does not prevent parsing or quoting another asset.

After [generating Node bindings](../README.md#build), supply your application's
original response text as `rawStatusJson` and the pool address as `poolAddress`:

```js
const { RelayerStatus, Pool } = require(
  './crates/target/tornadocash-wasm-node/kohaku_tornadocash_wasm.js',
);
const status = RelayerStatus.parse(rawStatusJson);
let pool;
let payer;
try {
  pool = Pool.fromAddress(poolAddress);
  if (pool === undefined || !status.supports(pool)) {
    throw new Error('Relayer does not support this pool');
  }
  payer = status.quote(pool, 1_000_000_000n, 100n);
  console.log(payer.address, payer.fee, payer.refund);
} finally {
  payer?.free();
  pool?.free();
  status.free();
}
```

The returned `Payer` is independently owned and remains valid after its status and
pool are freed. Its fee is in pool-asset base units; its refund is in native base
units. `new Payer(address, fee, refund)` also accepts a 20-byte Hex address and two
U256 bigints. Call `free()` on every owned wrapper when finished; metadata snapshots
need no cleanup. These bindings perform no relayer requests or withdrawal submission.
