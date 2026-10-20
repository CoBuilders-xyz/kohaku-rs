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

## Usage

Given an existing `Pool` and `Note`, construct the deposit and read its prepared
values. The caller can pass these to its transaction tooling:

```js
const deposit = new Deposit(pool, note);
const request = {
  to: deposit.target(),
  data: deposit.input(),
  value: deposit.value(),
};
const approval = deposit.approval(); // undefined for a native pool

deposit.free();
```
