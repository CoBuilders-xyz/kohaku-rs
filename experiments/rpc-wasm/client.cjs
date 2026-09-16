'use strict';

const assert = require('node:assert/strict');
Error.stackTraceLimit = 40;
const { sync_fixture } = require(process.argv[2]);
const port = Number(process.argv[3]);
const scenario = Number(process.argv[4]);
let ticks = 0;
const heartbeat = setInterval(() => ticks++, 10);
const deadline = setTimeout(() => {
  console.error('[timeout] SDK promise did not settle');
  process.exit(2);
}, 10000);

(async () => {
  try {
    const pending = sync_fixture(port, scenario);
    assert.ok(pending instanceof Promise);
    console.log('[promise] Rust async export returned a JavaScript Promise');
    const events = JSON.parse(await pending);
    assert.equal(scenario, 0, 'an error scenario unexpectedly succeeded');
    assert.ok(ticks >= 5, 'JS heartbeat did not progress during synchronization');
    console.log(JSON.stringify({ kind: 'resolved', events, ticks }));
  } catch (error) {
    if (scenario === 0) throw error;
    assert.ok(error instanceof Error);
    assert.ok(!(error instanceof WebAssembly.RuntimeError));
    assert.match(error.message, /fixture RPC failure/);
    console.log(JSON.stringify({ kind: 'rejected', message: error.message }));
  } finally {
    clearInterval(heartbeat);
    clearTimeout(deadline);
  }
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
