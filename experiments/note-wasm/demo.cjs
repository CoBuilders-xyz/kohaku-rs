'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

// This generated loader instantiates the actual .wasm module in Node.
const { note_commitment } = require('./output/pkg/kohaku_note.js');
const native = JSON.parse(fs.readFileSync(path.join(__dirname, 'output/native-reference.json'), 'utf8'));

console.log('LAB 10 — Real Tornado note operation through WASM in Node.');
console.log('Public deterministic fixtures only. Never use them with funds.');
assert.equal(typeof note_commitment, 'function');
console.log('\n[1] Generated JS loader loaded the WASM module: PASS');

// Match Lab 01's public fixture; this is data construction, not JS cryptography.
const prefix = 'tornado-ETH-1-31337-0x';
const original = prefix + '01'.repeat(31) + '02'.repeat(31);
assert.equal(original, native.original.note);

const commitment = note_commitment(original);
assert.match(commitment, /^0x[0-9a-f]{64}$/);
assert.equal(commitment, native.original.commitment);
assert.equal(commitment, '0x16530cfcf96f2ba56688084e9d9f55f49d6f73874bc2d7a1768b732335080504');
console.log('\n[2] JS passed a note string; real Rust SDK returned its commitment:');
console.log(commitment);
console.log('Matches same-revision native execution and the Lab 01 public fixture: PASS');

const changed = prefix + '01'.repeat(31) + '03' + '02'.repeat(30);
assert.equal(changed, native.changed.note);
const changedCommitment = note_commitment(changed);
assert.equal(changedCommitment, native.changed.commitment);
assert.equal(changedCommitment, '0x2bf0def8520d4f2d4ce0b28bb172a08f100d93f3bb7e44017de0f5409aa90c47');
assert.notEqual(changedCommitment, commitment);
console.log('\n[3] Changing one secret bit changes the commitment: PASS');
console.log(changedCommitment);

// These are SDK parsing errors translated into JS Error objects, not WASM traps.
for (const [label, encoded, message] of [
    ['invalid format', 'not-a-note', /^invalid note format$/],
    ['invalid chain id', original.replace('-31337-', '-abc-'), /^invalid chain id$/],
    ['invalid hex', prefix + 'zz'.repeat(62), /^invalid hex:/],
    ['short preimage', prefix + '01', /^invalid note format$/],
]) {
    assert.throws(() => note_commitment(encoded), (error) => {
        assert.ok(error instanceof Error);
        assert.ok(!(error instanceof WebAssembly.RuntimeError));
        assert.match(error.message, message);
        return true;
    });
    console.log(`\n[4] ${label}: caught expected JS Error — PASS`);
}

assert.equal(note_commitment(original), commitment);
console.log('\n[5] Valid call still works after the errors: PASS');
console.log('\nDone: one real synchronous SDK operation executed in Node through WASM.');
console.log('No browser, randomness, RPC, timers, workers, proofs, deposits, or withdrawals tested.');
