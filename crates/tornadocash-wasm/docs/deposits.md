# Deposit preparation

`Deposit` wraps the core Tornado deposit. Construct it from an existing `Pool`
and `Note`; both inputs are borrowed and remain usable. The deposit owns its
canonical core values independently.

The bindings expose the core's preparation methods:

| Method | JavaScript result |
| --- | --- |
| `target()` | Pool contract address as checksummed `Hex`. |
| `input()` | Core-encoded `deposit(commitment)` calldata as `Hex`. |
| `value()` | Exact native amount as `bigint`; `0n` for ERC20 pools. |
| `approval()` | Plain `TransactionData` object for ERC20 pools; `undefined` for native pools. |

Approval data uses this TypeScript shape:

```ts
interface TransactionData {
  to: Hex;
  data: Hex;
  value?: bigint;
}
```

The core builds the ERC20 `approve(pool, amount)` request. Its target is the token
contract, while the deposit target is the pool contract. The binding adapts the
request using the shared Hex and bigint conversions. An absent core value stays
absent; the current core approval omits `value`. Addresses use checksum formatting,
and calldata uses lowercase hex. Conversion errors throw JavaScript `Error`s.

These synchronous methods prepare unsigned data locally. The caller handles
existing allowance, signing, submission and confirmation. An executed ERC20
approval grants spending permission; the deposit is a separate transaction.

Call `free()` on owned `Deposit`, `Pool` and `Note` wrappers when finished.
Approval data is an ordinary JavaScript object managed by JavaScript's garbage
collector.

## Node example

Generate the Node bindings with the commands in the [bindings README](../README.md).
Run the following from the repository root. The secrets are fixed example inputs.
The example assembles deposit data in caller code from the three core getters.

```js
const { Deposit, Note, Pool } = require(
  './crates/target/tornadocash-wasm-node/kohaku_tornadocash_wasm.js',
);

const note = new Note(`0x${'01'.repeat(31)}`, `0x${'02'.repeat(31)}`);
try {
  for (const [label, address] of [
    ['0.1 ETH', '0x12D66f87A04A9E220743712cE6d9bB1B5616B8Fc'],
    ['100 DAI', '0xD4B88Df4D29F5CedD6857912842cff3b20C8Cfa3'],
  ]) {
    const pool = Pool.fromAddress(address);
    if (pool === undefined) throw new Error(`Unknown pool: ${address}`);

    let deposit;
    try {
      deposit = new Deposit(pool, note);
      const request = {
        to: deposit.target(),
        data: deposit.input(),
        value: deposit.value(),
      };

      console.log(label, request);
      console.log('approval', deposit.approval());
    } finally {
      deposit?.free();
      pool.free();
    }
  }
} finally {
  note.free();
}
```

The ETH request has value `100000000000000000n` and no approval. The DAI request
has value `0n`; its approval targets the DAI token and contains calldata without
a `value` property.
