const assert = require('node:assert/strict');
const { resolve } = require('node:path');
const { test } = require('node:test');

if (!process.env.TORNADOCASH_WASM_MODULE) {
  throw new Error('Set TORNADOCASH_WASM_MODULE to the generated wasm-bindgen JavaScript module');
}
const { parse_note } = require(resolve(process.env.TORNADOCASH_WASM_MODULE));
const nullifier = Uint8Array.from({ length: 31 }, (_, i) => i);
const secret = Uint8Array.from({ length: 31 }, (_, i) => i + 31);
const preimage = Buffer.concat([nullifier, secret]).toString('hex');

test('parse_note returns a plain object with exact metadata and ordered secret bytes', () => {
  const note = parse_note(`tornado-eth-0.10-1-0x${preimage}`);

  assert.equal(Object.getPrototypeOf(note), Object.prototype);
  assert.deepEqual(note, {
    symbol: 'eth',
    amount: '0.10',
    chainId: 1n,
    nullifier,
    secret,
  });
  assert.ok(note.nullifier instanceof Uint8Array);
  assert.ok(note.secret instanceof Uint8Array);
});

test('parse_note preserves chain IDs as bigint across the u64 range', () => {
  for (const chainId of [0n, 9007199254740993n, 18446744073709551615n]) {
    const note = parse_note(`tornado-eth-1-${chainId}-0x${preimage}`);
    assert.equal(note.chainId, chainId);
  }
});

test('parse_note keeps core parsing behavior for optional hex prefix and metadata', () => {
  const note = parse_note(`tornado-TOKEN-custom-1-${preimage}`);
  assert.equal(note.symbol, 'TOKEN');
  assert.equal(note.amount, 'custom');
  assert.deepEqual(note.nullifier, nullifier);
  assert.deepEqual(note.secret, secret);
});

test('parse_note exposes core parsing failures as JavaScript Errors', () => {
  const cases = [
    ['', 'invalid note format'],
    [` tornado-eth-1-1-0x${preimage}`, 'invalid note format'],
    [`tornado-eth-1-invalid-0x${preimage}`, 'invalid chain id'],
    [`tornado-eth-1-18446744073709551616-0x${preimage}`, 'invalid chain id'],
    ['tornado-eth-1-1-0x00', 'invalid note format'],
    [`tornado-eth-1-1-0x${'ff'.repeat(63)}`, 'invalid note format'],
    [`tornado-eth-1-1-0x${'zz'.repeat(62)}`, /invalid hex/],
  ];
  for (const [note, message] of cases) {
    assert.throws(() => parse_note(note), { name: 'Error', message });
  }
});
