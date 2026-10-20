# Decode supplied Tornado logs

`decodeEvent(log: LogInput): DecodedEvent | undefined` synchronously delegates to
the core's `SyncEvent::try_from_log`. Supply a plain object with `topics: Hex[]`,
`data: Hex` and optional `blockNumber: bigint`. Each topic must be exactly 32
bytes, with at most four topics; data may contain arbitrary bytes, including `0x`.

Omitting `blockNumber` or setting it to `undefined` means absent. A present block
must be an unsigned u64 bigint; `null` is rejected. Malformed structural input
throws a JavaScript `Error`. Structurally valid input preserves the core's
`undefined` result for unknown/undecodable events or absent block numbers.

The result is a plain discriminated union:

| `kind` | Fields |
| --- | --- |
| `"deposit"` | `commitment: Hex`, `leafIndex: number`, `blockNumber: bigint` |
| `"withdrawal"` | `to: Hex`, `nullifierHash: Hex`, `relayer: Hex`, `fee: bigint`, `blockNumber: bigint` |

`leafIndex` preserves u32, block numbers preserve u64 and fees preserve the full
U256 value. Commitments and nullifier hashes use lowercase hex; addresses use
canonical Alloy checksum formatting. Snapshots need no `free()` call.

With the generated module loaded and a caller-supplied `log`:

```js
const event = decodeEvent(log);
if (event?.kind === 'deposit') {
  console.log(event.commitment, event.leafIndex, event.blockNumber);
}
```

The caller selects logs for the correct pool and chain and handles removals,
reorgs, ordering and completeness. The input deliberately omits emitter and
removed metadata; decoding does not establish provenance or whether a note is
unspent. The binding neither fetches logs nor repairs history.
