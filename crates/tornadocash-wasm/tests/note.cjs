const assert = require('node:assert/strict');
const { resolve } = require('node:path');
const { test } = require('node:test');

if (!process.env.TORNADOCASH_WASM_MODULE) {
  throw new Error('Set TORNADOCASH_WASM_MODULE to the generated wasm-bindgen JavaScript module');
}
const { note_commitment } = require(resolve(process.env.TORNADOCASH_WASM_MODULE));
const preimage = '01'.repeat(31) + '02'.repeat(31);

test('a note returns a 32-byte lowercase hex string', () => {
  const commitment = note_commitment(`tornado-eth-1-1-0x${preimage}`);
  assert.equal(typeof commitment, 'string');
  assert.match(commitment, /^0x[0-9a-f]{64}$/);
});

test('parse failures cross the WASM boundary as JavaScript Errors', () => {
  const cases = [
    ['', 'invalid note format'],
    [`tornado-eth-1-invalid-0x${preimage}`, 'invalid chain id'],
    [`tornado-eth-1-18446744073709551616-0x${preimage}`, 'invalid chain id'],
    ['tornado-eth-1-1-0x00', 'invalid note format'],
    [`tornado-eth-1-1-0x${'zz'.repeat(62)}`, /invalid hex/],
  ];
  for (const [note, message] of cases) {
    assert.throws(() => note_commitment(note), { name: 'Error', message });
  }
});
