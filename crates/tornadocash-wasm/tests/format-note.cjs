const assert = require('node:assert/strict');
const { resolve } = require('node:path');
const { test } = require('node:test');

if (!process.env.TORNADOCASH_WASM_MODULE) {
  throw new Error('Set TORNADOCASH_WASM_MODULE to the generated wasm-bindgen JavaScript module');
}
const { format_note, parse_note } = require(resolve(process.env.TORNADOCASH_WASM_MODULE));
const fields = {
  symbol: 'eth',
  amount: '0.10',
  chainId: 1n,
  nullifier: Uint8Array.from({ length: 31 }, (_, i) => i),
  secret: Uint8Array.from({ length: 31 }, (_, i) => 255 - i),
};
const preimage = Buffer.concat([fields.nullifier, fields.secret]).toString('hex');

test('format_note produces the core legacy format from structured data', () => {
  const text = format_note(fields);
  assert.equal(text, `tornado-eth-0.10-1-0x${preimage}`);
  assert.deepEqual(parse_note(text), fields);
});

test('format_note canonicalizes hex while preserving the amount text', () => {
  const input = `tornado-eth-0.10-1-${preimage.toUpperCase()}`;
  assert.equal(format_note(parse_note(input)), `tornado-eth-0.10-1-0x${preimage}`);
});

test('format_note preserves chain IDs across the u64 range', () => {
  for (const chainId of [0n, 9007199254740993n, 18446744073709551615n]) {
    const note = { ...fields, chainId };
    assert.equal(format_note(note), `tornado-eth-0.10-${chainId}-0x${preimage}`);
    assert.deepEqual(parse_note(format_note(note)), note);
  }
});

test('format_note retains the core constructor and formatter metadata behavior', () => {
  assert.equal(
    format_note({ ...fields, symbol: 'TOKEN', amount: 'custom' }),
    `tornado-TOKEN-custom-1-0x${preimage}`,
  );
});

test('format_note rejects invalid structured inputs with JavaScript Errors', () => {
  const { secret, ...missingSecret } = fields;
  const invalid = [
    undefined,
    null,
    'not an object',
    missingSecret,
    { ...fields, symbol: 1 },
    { ...fields, amount: 0.1 },
    { ...fields, chainId: -1n },
    { ...fields, chainId: 18446744073709551616n },
    { ...fields, chainId: 1.5 },
    { ...fields, nullifier: new Uint8Array(30) },
    { ...fields, nullifier: new Uint8Array(32) },
    { ...fields, secret: new Uint8Array(30) },
    { ...fields, secret: new Uint8Array(32) },
  ];
  for (const note of invalid) {
    assert.throws(() => format_note(note), Error);
  }
  // A failed conversion must leave the instance usable.
  assert.equal(format_note(fields), `tornado-eth-0.10-1-0x${preimage}`);
});
