'use strict';

const assert = require('node:assert/strict');
const http = require('node:http');
const path = require('node:path');
const { spawn } = require('node:child_process');
const { performance } = require('node:perf_hooks');

const variant = process.argv[2] ?? 'adapted';
assert.ok(['baseline', 'adapted'].includes(variant), 'expected baseline or adapted');
const pkg = path.join(__dirname, 'output', variant, 'pkg', 'kohaku_rpc.js');
const { deposit_topic } = require(pkg);
const topic = deposit_topic();
const address = '0x' + '11'.repeat(20);
const ranges = [[100, 101], [102, 103], [104, 105]];
const requests = [];
const serverErrors = [];
const word = (n) => BigInt(n).toString(16).padStart(64, '0');
const hex = (n) => '0x' + n.toString(16);
const commitment = (index) => '0x' + ['11', '22', '33'][index].repeat(32);

function fixtureLog(index) {
  return {
    address,
    topics: [topic, commitment(index)],
    data: '0x' + word(index) + word(1700000000 + index),
    blockNumber: hex(ranges[index][0]),
    blockHash: '0x' + word(1000 + index),
    transactionHash: '0x' + word(2000 + index),
    transactionIndex: '0x0',
    logIndex: '0x0',
    removed: false,
  };
}

const server = http.createServer(async (req, res) => {
  try {
    assert.equal(req.method, 'POST');
    assert.ok(['/success', '/fail-first', '/fail-second'].includes(req.url));
    let text = '';
    for await (const chunk of req) {
      text += chunk;
      assert.ok(text.length < 65536, 'oversized local fixture request');
    }
    const rpc = JSON.parse(text);
    assert.equal(rpc.jsonrpc, '2.0');
    assert.equal(rpc.method, 'eth_getLogs', 'unexpected RPC method');
    const filter = rpc.params[0];
    const index = requests.filter((entry) => entry.route === req.url).length;
    assert.ok(index < 3, 'unexpected extra request');
    const addresses = Array.isArray(filter.address) ? filter.address : [filter.address];
    assert.deepEqual(addresses, [address]);
    assert.deepEqual([Number(BigInt(filter.fromBlock)), Number(BigInt(filter.toBlock))], ranges[index]);
    requests.push({ route: req.url, index, at: performance.now(), range: ranges[index] });
    console.log(`[rpc] ${req.url} eth_getLogs ${ranges[index].join('..')}`);
    const fail = req.url === '/fail-first' || (req.url === '/fail-second' && index === 1);
    res.setHeader('content-type', 'application/json');
    res.end(JSON.stringify({ jsonrpc: '2.0', id: rpc.id,
      ...(fail ? { error: { code: -32602, message: 'fixture RPC failure' } }
        : { result: [fixtureLog(index)] }),
    }));
  } catch (error) {
    serverErrors.push(error);
    res.writeHead(500).end('local fixture assertion failed');
  }
});

function runClient(port, scenario) {
  return new Promise((resolve, reject) => {
    // Separate processes isolate the baseline's fatal WASM panic.
    const child = spawn(process.execPath, [path.join(__dirname, 'client.cjs'), pkg, String(port), String(scenario)],
      { stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '';
    let stderr = '';
    const timeout = setTimeout(() => child.kill('SIGTERM'), 12000);
    child.stdout.on('data', (data) => { stdout += data; process.stdout.write(data); });
    child.stderr.on('data', (data) => { stderr += data; process.stderr.write(data); });
    child.on('error', (error) => { clearTimeout(timeout); reject(error); });
    child.on('close', (code, signal) => {
      clearTimeout(timeout);
      resolve({ code, signal, stdout, stderr });
    });
  });
}

function lastResult(run) {
  return JSON.parse(run.stdout.trim().split('\n').at(-1));
}

(async () => {
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  const port = server.address().port;
  console.log(`LAB 11 — ${variant}; loopback fixture server on port ${port}`);
  console.log('Real Rust RpcSyncer and HTTP transport; fabricated logs, no blockchain or funds.');
  try {
    const failure = await runClient(port, 1);
    assert.equal(failure.signal, null);
    assert.equal(failure.code, 0, 'early RPC error was not handled');
    assert.equal(lastResult(failure).kind, 'rejected');
    assert.equal(requests.filter((r) => r.route === '/fail-first').length, 1);
    console.log('[check] RPC error before any SDK sleep rejects the Promise: PASS');

    const success = await runClient(port, 0);
    assert.equal(success.signal, null, 'client timed out or was killed');
    const batches = requests.filter((r) => r.route === '/success');
    if (variant === 'baseline') {
      assert.equal(success.code, 1, 'expected a fatal Tokio runtime failure');
      assert.match(success.stderr, /time not implemented on this platform/);
      assert.match(success.stderr, /tokio.*::time::instant/);
      assert.match(success.stderr, /tokio.*::time::sleep/);
      assert.equal(batches.length, 1, 'baseline must reach the first HTTP response');
      console.log('[check] Expected baseline runtime panic at the first SDK sleep: CONFIRMED');
    } else {
      assert.equal(success.code, 0, 'synchronization failed');
      assert.deepEqual(batches.map((r) => r.range), ranges);
      const result = lastResult(success);
      assert.equal(result.kind, 'resolved');
      assert.deepEqual(result.events, ranges.map((_, index) => ({
        kind: 'deposit', commitment: commitment(index), leafIndex: index,
        timestamp: String(1700000000 + index),
      })));
      const gaps = batches.slice(1).map((batch, i) => batch.at - batches[i].at);
      assert.ok(gaps.every((gap) => gap >= 80), '100ms SDK batch delay was not observed');
      console.log(`[check] Three decoded deposits; gaps ${gaps.map((n) => n.toFixed(1)).join(', ')} ms; heartbeat ${result.ticks}: PASS`);

      const lateFailure = await runClient(port, 2);
      assert.equal(lateFailure.signal, null);
      assert.equal(lateFailure.code, 0);
      assert.equal(lastResult(lateFailure).kind, 'rejected');
      assert.equal(requests.filter((r) => r.route === '/fail-second').length, 2);
      console.log('[check] RPC error after one successful batch/sleep rejects without returning partial data: PASS');
    }
    assert.deepEqual(serverErrors, []);
    console.log(`[result] ${variant} experiment: PASS`);
  } finally {
    server.closeAllConnections();
    await new Promise((resolve) => server.close(resolve));
  }
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
