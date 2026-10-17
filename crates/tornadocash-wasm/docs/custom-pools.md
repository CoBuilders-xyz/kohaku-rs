# Custom pools and paymaster metadata

`new Pool(chainId, address, asset, amountWei, deployedBlock, paymaster?)`
constructs an owned canonical Rust Pool from caller-supplied metadata. It is
synchronous and performs no network requests or catalog registration.

| Parameter | Type | Boundary |
| --- | --- | --- |
| `chainId` | `bigint` | `0n` through `18446744073709551615n` (u64). |
| `address` | `Hex` | Pool contract address, exactly 20 bytes. |
| `asset` | `Asset` | Borrowed; its canonical metadata is cloned once. |
| `amountWei` | `bigint` | `0n` through `340282366920938463463374607431768211455n` (u128), in asset base units. |
| `deployedBlock` | `bigint` | `0n` through `18446744073709551615n` (u64). |
| `paymaster` | `PaymasterInfo \| null` (optional) | Plain object `{ address: Hex, adapter: Hex }`, both exactly 20 bytes. |

Hex strings require the `0x` prefix. Inputs need not use checksums; address
outputs use the existing Alloy checksummed representation. Invalid metadata,
negative or overflowing integers, and non-bigint numeric inputs throw JavaScript
`Error`s. Numbers and decimal strings are not coerced into bigints. The
constructor preserves valid metadata without discovering chains, checking deployed
contracts, or imposing additional domain rules.

`pool.paymaster` returns a plain `PaymasterInfo` object or `undefined`. This also
works for pools returned by `known()`, `fromAddress()` and `fromNote()`. Omit the
constructor argument or pass `undefined` for a pool without paymaster metadata.
The generated optional argument also accepts `null` as absence; the getter returns
`undefined` in each case.
This is metadata only; it does not execute paymaster operations.

The existing read-only getters and `amount()`, `id()`, `symbol()` and `toString()`
delegate to the canonical core. Decimal formatting supports the Asset's complete
`0..=255` decimals range without overflowing, returns `"0"` for zero, and removes
fractional trailing zeros. The denomination remains an exact bigint in
`pool.amountWei`; decimal text is never converted through a JavaScript number.

The constructor borrows the Asset, including when validation fails. Callers may
reuse or free that Asset independently of the Pool. Each `pool.asset` read creates
another owned Asset wrapper that survives freeing the Pool. Call `free()` once on
each owned WASM wrapper when finished and do not reuse freed wrappers. Paymaster
objects are copied at the boundary, have no `free()` method, and can outlive the
Pool; mutating an input or returned object does not modify the core Pool.

## Node usage example

Run from the repository root after building and generating Node bindings with
the [crate's build commands](../README.md#build). No RPC service or secrets are
required. This usage example constructs a pool, reads its metadata and frees the
owned wrappers. Boundary and regression tests live in Rust beside the bindings.

```sh
node <<'JS'
const { Asset, Pool } = require(
  './crates/target/tornadocash-wasm-node/kohaku_tornadocash_wasm.js',
);

const asset = Asset.native('XYZ', 6);
let pool;
try {
  pool = new Pool(
    1n,
    '0x00000000000000000000000000000000000000ef',
    asset, 1230000n, 19000000n,
    {
      address: '0x00000000000000000000000000000000000000ab',
      adapter: '0x00000000000000000000000000000000000000cd',
    },
  );
  console.log(pool.symbol(), pool.amount()); // XYZ 1.23
  console.log(pool.paymaster);
  // { address: '0x00000000000000000000000000000000000000AB',
  //   adapter: '0x00000000000000000000000000000000000000cD' }
} finally {
  pool?.free();
  asset.free(); // The constructor borrows the Asset; we still own it.
}
JS
```

For browser bindings, initialize the generated web module with its default
`init()` export first. The constructor and ownership rules are the same.
