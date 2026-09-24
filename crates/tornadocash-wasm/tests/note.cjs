const assert = require('node:assert/strict');
const { resolve } = require('node:path');
const { test } = require('node:test');

if (!process.env.TORNADOCASH_WASM_MODULE) {
  throw new Error('Set TORNADOCASH_WASM_MODULE to the generated wasm-bindgen JavaScript module');
}
const { Note } = require(resolve(process.env.TORNADOCASH_WASM_MODULE));
const preimage = '01'.repeat(31) + '02'.repeat(31);

for (const method of ['commitment', 'nullifierHash']) {
  test(`Note.${method} returns a 32-byte lowercase hex string`, (t) => {
    const note = Note.parse(`tornado-eth-1-1-0x${preimage}`);
    t.after(() => note.free());
    const hash = note[method]();
    assert.equal(typeof hash, 'string');
    assert.match(hash, /^0x[0-9a-f]{64}$/);
    assert.equal(note[method](), hash);
  });
}

test('nullifier hash depends on the nullifier, independently of the secret and metadata', (t) => {
  const notes = [
    Note.parse(`tornado-eth-1-1-0x${preimage}`),
    Note.parse(`tornado-eth-1-1-0x${'01'.repeat(31)}${'03'.repeat(31)}`),
    Note.parse(`tornado-dai-100-5-0x${preimage}`),
    Note.parse(`tornado-eth-1-1-0x${'03'.repeat(31)}${'02'.repeat(31)}`),
  ];
  for (const note of notes) t.after(() => note.free());
  const hash = notes[0].nullifierHash();
  assert.equal(hash, notes[1].nullifierHash());
  assert.equal(hash, notes[2].nullifierHash());
  assert.notEqual(hash, notes[3].nullifierHash());
});

test('two notes coexist in one WASM module and can be freed independently', (t) => {
  const first = Note.parse(`tornado-eth-1-1-0x${preimage}`);
  const second = Note.parse(`tornado-eth-1-1-0x${'03'.repeat(62)}`);
  t.after(() => second.free());
  const firstHash = first.commitment();
  const secondHash = second.commitment();
  const snapshot = first.toObject();
  assert.notEqual(firstHash, secondHash);
  first.free();
  for (const method of ['toObject', 'toString', 'commitment', 'nullifierHash']) {
    assert.throws(() => first[method](), Error);
  }
  assert.equal(second.commitment(), secondHash);
  assert.deepEqual(snapshot.nullifier, new Uint8Array(31).fill(1));
  assert.deepEqual(snapshot.secret, new Uint8Array(31).fill(2));
});
