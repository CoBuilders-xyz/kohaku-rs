# Tornado Merkle inclusion

`MerkleTree` wraps the core's fixed depth-20 binary Tornado tree. It accepts an
ordered array of commitment hex strings directly, independently of event decoding.
Hashing, tree updates and proof verification delegate to the canonical Rust core.
All operations are synchronous and local.

## API

| Surface | Result | Behavior |
| --- | --- | --- |
| `new MerkleTree()` | Owned `MerkleTree` | Construct an empty tree. |
| `MerkleTree.fromLeaves(commitments)` | Owned `MerkleTree` | Construct from commitments in leaf order. |
| `tree.splice(startIndex, commitments)` | `void` | Replace existing leaves and append any remaining leaves contiguously. |
| `tree.root()` | `Hex` | Read the current root. |
| `tree.proof(index)` | Owned `MerkleProof` | Capture inclusion of an existing leaf against the current root. |
| `proof.leaf`, `proof.root` | Read-only `Hex` | Read the proof's captured leaf and root. |
| `proof.verify()` | `boolean` | Verify the canonical proof with `TornadoHasher`. |

`Commitments` is a TypeScript alias for `Hex[]`, not a WASM wrapper. Each input
must decode to exactly 32 bytes and lie in the canonical BN254 scalar-field range;
out-of-range inputs are rejected rather than reduced. Leaf and root outputs are
lowercase, 32-byte, `0x`-prefixed hex strings.

Indices must be actual finite integer JavaScript numbers in `0..=4294967295`.
Strings, bigint values and fractional or out-of-range numbers are rejected before
conversion. The core checks leaf bounds and the tree's capacity of `2 ** 20` leaves.
Invalid boundary inputs and core errors throw JavaScript `Error`s.

`splice` does not insert gaps or shift later leaves. A nonempty batch can start at
an existing leaf or immediately after the last leaf. Existing entries are replaced;
the batch can also extend the tree. The entire batch is converted before mutation,
so a malformed later commitment cannot partially apply an update. An empty batch
does nothing, including at a start index beyond the current leaves, provided the
index passes the numeric validation above. `proof(index)` requires an existing leaf.

## Ownership and meaning

Trees and proofs are independently owned WASM wrappers. Call `free()` on each when
finished; do not reuse a freed wrapper. Arrays and hex strings are ordinary JS
values and do not require `free()`. Reading a root or creating a proof borrows the
tree; neither operation consumes it.

A proof retains its captured root when the tree changes or is freed. Its `verify()`
checks that captured root, not automatically the current tree root. Proof data stay
private: JavaScript has no arbitrary proof constructor or path import/export API.

Local inclusion does not establish that the root is accepted on-chain, that a note
is unspent, or that supplied logs are complete or canonical. Callers select the
correct chain and pool, check event order and contiguity, and handle removals or
reorganizations. An inclusion proof is a hash path, not the zero-knowledge proof
needed for withdrawal.

## Node example

Run from the repository root after generating the Node bindings using the
[build instructions](../README.md#build). These fixed field values illustrate the
local API without fetching logs or asserting that they represent real deposits.

```js
const assert = require('node:assert/strict');
const { MerkleTree } = require(
  './crates/target/tornadocash-wasm-node/kohaku_tornadocash_wasm.js',
);

const commitments = [1n, 2n, 3n].map(
  value => `0x${value.toString(16).padStart(64, '0')}`,
);
const tree = MerkleTree.fromLeaves(commitments.slice(0, 2));
let proof;
try {
  try {
    proof = tree.proof(1);
    assert.equal(proof.leaf, commitments[1]);
    assert.equal(proof.root, tree.root());
    assert.equal(proof.verify(), true);

    // Replace leaf 1 and append leaf 2. The earlier proof keeps its root.
    tree.splice(1, [commitments[2], commitments[0]]);
    assert.notEqual(tree.root(), proof.root);
  } finally {
    tree.free();
  }

  // The proof remains usable after its source tree is freed.
  assert.equal(proof.verify(), true);
  console.log('Captured leaf:', proof.leaf);
  console.log('Captured root:', proof.root);
} finally {
  proof?.free();
}
```

The same API is generated for the web target; initialize that module before use.
Generating web bindings is not browser runtime verification.
