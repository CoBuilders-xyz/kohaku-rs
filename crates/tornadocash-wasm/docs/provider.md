# Read-only pool queries

`TornadoProvider` holds a canonical Alloy HTTP provider and delegates pool queries
to Rust's `TornadoProviderExt`. It accepts an absolute HTTP(S) RPC URL. Constructing
it does not connect, discover a chain, or verify that the endpoint matches a pool's
`chainId`; choose the correct endpoint for that pool.

```ts
import { Pool, TornadoProvider } from './kohaku_tornadocash_wasm.js';

const provider = new TornadoProvider(rpcUrl);
const pool = Pool.fromAddress('0x12D66f87A04A9E220743712cE6d9bB1B5616B8Fc');
if (!pool) throw new Error('Unknown pool');

try {
    const hash = `0x${'00'.repeat(31)}01` as const;
    const spent = await provider.isSpent(pool, hash);
    const ordered = await provider.isSpentArray(pool, [hash, hash]);
    const known = await provider.isKnownRoot(pool, hash);
    const feeTokenAmount = await provider.quoteWeiInFeeToken(pool, 10n ** 18n);
    console.log({ spent, ordered, known, feeTokenAmount });
} finally {
    pool.free();
    provider.free();
}
```

For web bindings, call the generated module's default `init()` before constructing
wrappers. Node bindings need no explicit initialization. The runtime must provide
`fetch`, `Request`, `Response`, `Headers` and `AbortController` (as in modern Node
and browsers). Browser endpoints must permit the page's origin through CORS;
an HTTPS page also needs an endpoint permitted by its mixed-content policy.

| Method | Input | Promise result |
| --- | --- | --- |
| `isSpent(pool, hash)` | A nullifier hash | `boolean` |
| `isSpentArray(pool, hashes)` | An ordered array of nullifier hashes | `boolean[]`, with core length and order |
| `isKnownRoot(pool, root)` | A Merkle root | `boolean` |
| `quoteWeiInFeeToken(pool, weiAmount)` | Exact unsigned wei amount | Exact `bigint` in the pool's fee token base units |

Hashes and roots use the shared `Hex` representation: `0x` followed by exactly
32 bytes, within the canonical BN254 field range. Leading zero bytes matter.
Amounts must be actual JavaScript `bigint` values in `0..=(2n ** 256n - 1n)`;
numbers, decimal strings, negative values and overflow reject without coercion.
Native-asset quotes return the input exactly and do not contact the endpoint.
ERC20 quotes call the core's `quoteWeiInToken` using the pool and token addresses.

Query methods return Promises. Invalid field/amount conversions and RPC, HTTP or
contract decoding failures reject with an `Error`; failures never become `false`,
an empty array, or a zero quote. Invalid constructor URLs throw synchronously.
Calling methods with freed or incompatible wrapper objects can also throw in the
generated binding glue.

Each method borrows the wrappers only during the synchronous call, copies the
canonical pool and field/amount data, and retains a cloned Alloy provider handle
for the future. Callers may mutate their original hash array or `free()` the pool
and provider immediately after obtaining a Promise; the pending request keeps
its original inputs and remains valid. `free()` releases that wrapper, does not
cancel pending work, and the wrapper must not be reused afterward.

The API exposes no generic Alloy methods, signing, transaction submission,
injected-wallet adapter, relayer client or synchronization service.

## Focused tests and coverage limits

The Rust tests beside `provider.rs` check the wrapper's HTTP(S) scheme restriction,
conversion of every supplied array element, and use of the strict bigint input
boundary. Shared Hex/Field and U256 conversion coverage is reused. Core query
semantics, Alloy/reqwest transport, Promise scheduling and generated borrow/free
behavior rely on their documented library/toolchain contracts. The wrapper keeps
explicit owned snapshots and a cloned provider handle in each future.

Run the provider tests with the existing tooling from the repository root:

```sh
CARGO_BUILD_JOBS=2 nix --extra-experimental-features 'nix-command flakes' develop .#ci --command cargo test --locked --manifest-path crates/Cargo.toml -p kohaku-tornadocash-wasm --lib --target wasm32-unknown-unknown --target-dir crates/target provider::wasm_tests
```

The simulated in-flight RPC regression and custom browser/network runner have
been removed and are no longer maintained. Earlier deferred-response tests and
Node/Chromium loopback HTTP runs remain historical evidence for their original
snapshots. No equivalent runtime coverage is claimed for the current tests.
Real HTTPS has never been validated in this delivery; accepting an HTTPS URL is
only a constructor check. Current checks add no browser, CORS or TLS coverage.
