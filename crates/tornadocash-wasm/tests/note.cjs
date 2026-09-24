const assert = require('node:assert/strict');
const { resolve } = require('node:path');
const { test } = require('node:test');

if (!process.env.TORNADOCASH_WASM_MODULE) {
  throw new Error('Set TORNADOCASH_WASM_MODULE to the generated wasm-bindgen JavaScript module');
}
const { note_commitment, note_nullifier_hash } = require(resolve(process.env.TORNADOCASH_WASM_MODULE));
const preimage = '01'.repeat(31) + '02'.repeat(31);

for (const [name, hashNote] of Object.entries({ note_commitment, note_nullifier_hash })) {
  test(`${name} returns a 32-byte lowercase hex string`, () => {
    const hash = hashNote(`tornado-eth-1-1-0x${preimage}`);
    assert.equal(typeof hash, 'string');
    assert.match(hash, /^0x[0-9a-f]{64}$/);
  });

  test(`${name} throws JavaScript Errors for parse failures`, () => {
    const cases = [
      ['', 'invalid note format'],
      [`tornado-eth-1-invalid-0x${preimage}`, 'invalid chain id'],
      [`tornado-eth-1-18446744073709551616-0x${preimage}`, 'invalid chain id'],
      ['tornado-eth-1-1-0x00', 'invalid note format'],
      [`tornado-eth-1-1-0x${'zz'.repeat(62)}`, /invalid hex/],
    ];
    for (const [note, message] of cases) {
      assert.throws(() => hashNote(note), { name: 'Error', message });
    }
  });
}

test('nullifier hash depends on the nullifier, independently of the secret and metadata', () => {
  const hash = note_nullifier_hash(`tornado-eth-1-1-0x${preimage}`);
  const differentSecret = '01'.repeat(31) + '03'.repeat(31);
  const differentNullifier = '03'.repeat(31) + '02'.repeat(31);

  assert.equal(hash, note_nullifier_hash(`tornado-eth-1-1-0x${differentSecret}`));
  assert.equal(hash, note_nullifier_hash(`tornado-dai-100-5-0x${preimage}`));
  assert.notEqual(hash, note_nullifier_hash(`tornado-eth-1-1-0x${differentNullifier}`));
});
