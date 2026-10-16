# Decode supplied Tornado logs

`decodeEvent(log: LogInput): DecodedEvent | undefined` is synchronous and delegates
to the core's `SyncEvent::try_from_log`. It decodes one caller-supplied log; it
does not fetch logs or contact a provider.

The input is a plain object:

| Property | Type | Validation |
| --- | --- | --- |
| `topics` | `Hex[]` | At most four entries, each exactly 32 bytes. |
| `data` | `Hex` | Arbitrary bytes, including empty `0x`; need not be ABI-aligned. |
| `blockNumber` | optional `bigint` | Omitted or `undefined` means absent. A present value must be in `0n..=18446744073709551615n`. |

`Hex` is the shared TypeScript type `` `0x${string}` ``. Malformed objects, invalid
hex, topic sizes/counts, or invalid present block numbers throw JavaScript `Error`s.
Numbers, strings and `null` are not accepted as block numbers. Inputs are not coerced
or reduced into range.

The result is a plain discriminated union:

| `kind` | Fields |
| --- | --- |
| `"deposit"` | `commitment: Hex`, `leafIndex: number`, `blockNumber: bigint` |
| `"withdrawal"` | `to: Hex`, `nullifierHash: Hex`, `relayer: Hex`, `fee: bigint`, `blockNumber: bigint` |

`leafIndex` preserves the core's `u32`. Block numbers and the full `u256` fee stay
exact as JavaScript bigint. Commitments and nullifier hashes use lowercase hex;
addresses use the shared conversion's canonical Alloy checksum formatting.
Snapshots live in JavaScript and need no `free()` call.

Every core `None` becomes `undefined`. This includes unknown signatures, undecodable
ABI or out-of-range canonical fields, and an absent block number. For example,
valid hex data with an invalid event ABI can yield `undefined` rather than throw.
This API does not distinguish the reasons for a core no-result.

The input deliberately omits emitter and removed metadata. The caller must select
logs for the correct pool and chain and handle reorgs, removals and canonicality.
A decoded event does not establish provenance, completeness, ordering, on-chain
root acceptance or whether a note is unspent. The binding does not sort events or
fill history gaps.

## Deterministic Node example

Generate the Node bindings using the [build instructions](../README.md#build),
then run this JavaScript from the repository root with Node (available in
`nix develop .#ci`). No RPC or live-chain fixture is needed.

These synthetic fixtures were generated offline with `cast abi-encode-event` from
the event declarations in the [core ABI](../../tornadocash/src/abis/tornado.rs).
The deposit encodes commitment `0xab`, leaf index `7` and timestamp `123`. The
withdrawal encodes recipient `0xab`, nullifier hash `0xcd`, the Ethereum DAI address
as relayer and a fee of `2^200 + 17`. They are examples of encoding, not on-chain
activity. The same fixtures are exercised by the Rust-authored WASM tests.

```js
const assert = require('node:assert/strict');
const { decodeEvent } = require(
  './crates/target/tornadocash-wasm-node/kohaku_tornadocash_wasm.js',
);

const depositLog = {
  topics: [
    '0xa945e51eec50ab98c161376f0db4cf2aeba3ec92755fe2fcd388bdbbb80ff196',
    '0x00000000000000000000000000000000000000000000000000000000000000ab',
  ],
  data: '0x' + [
    '0000000000000000000000000000000000000000000000000000000000000007',
    '000000000000000000000000000000000000000000000000000000000000007b',
  ].join(''),
  blockNumber: 9007199254740993n,
};
const deposit = decodeEvent(depositLog);
assert.deepEqual(deposit, {
  kind: 'deposit',
  commitment: depositLog.topics[1],
  leafIndex: 7,
  blockNumber: 9007199254740993n,
});

const withdrawal = decodeEvent({
  topics: [
    '0xe9e508bad6d4c3227e881ca19068f099da81b5164dd6d62b2eaf1e8bc6c34931',
    '0x0000000000000000000000006b175474e89094c44da98b954eedeac495271d0f',
  ],
  data: '0x' + [
    '00000000000000000000000000000000000000000000000000000000000000ab',
    '00000000000000000000000000000000000000000000000000000000000000cd',
    '0000000000000100000000000000000000000000000000000000000000000011',
  ].join(''),
  blockNumber: 18446744073709551615n,
});
assert.deepEqual(withdrawal, {
  kind: 'withdrawal',
  to: '0x00000000000000000000000000000000000000AB',
  nullifierHash: '0x00000000000000000000000000000000000000000000000000000000000000cd',
  relayer: '0x6B175474E89094C44Da98b954EedeAC495271d0F',
  fee: (1n << 200n) + 17n,
  blockNumber: 18446744073709551615n,
});

// A missing block or undecodable ABI preserves the core no-result.
assert.equal(decodeEvent({ ...depositLog, blockNumber: undefined }), undefined);
assert.equal(decodeEvent({ ...depositLog, data: '0x' }), undefined);

// Structurally invalid input throws, including a number instead of bigint.
assert.throws(() => decodeEvent({ ...depositLog, blockNumber: 1 }), Error);
console.log(deposit.kind, withdrawal.kind, withdrawal.fee);
// No free(): both values are ordinary JavaScript snapshots.
```

For a browser build, initialize the generated web module before calling
`decodeEvent`. Generating web bindings alone does not verify browser execution.
