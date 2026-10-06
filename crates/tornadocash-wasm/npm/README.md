# Tornado Cash WASM bindings

JavaScript/TypeScript bindings for Tornado Cash notes, backed by `kohaku-tornadocash`.
Includes compiled WASM for Node and browsers; no Rust toolchain is needed to use it.

## Install

```sh
npm install @cobuilders/kohaku-tornadocash-wasm@dev
```

## Node

WASM initializes automatically. Examples use fixed inputs for demonstration.

```js
import { Note, NoteString } from '@cobuilders/kohaku-tornadocash-wasm/node';

const note = new Note(`0x${'01'.repeat(31)}`, `0x${'02'.repeat(31)}`);
const noteString = new NoteString(note, 'eth', '0.1', 1n);
const parsed = NoteString.parse(noteString.toString());

console.log(parsed.commitment());
parsed.free();
noteString.free();
```

## Browser

```js
import init, { Note } from '@cobuilders/kohaku-tornadocash-wasm/web';

await init();
const note = new Note(`0x${'01'.repeat(31)}`, `0x${'02'.repeat(31)}`);
console.log(note.commitment());
note.free();
```

Use a bundler that serves the generated WASM URL, or an import map and HTTP server.
Call `init()` before using the classes. Use the same entry point for all note objects.

## Notes API

- `new Note(nullifier, secret)` accepts two 31-byte, `0x`-prefixed hex strings.
- `new NoteString(note, symbol, amount, chainId)` adds asset and chain metadata.
- `NoteString.parse(text)` reads a Tornado note; `toString()` formats it.
- Both classes expose read-only `nullifier` and `secret` getters and the methods
  `preimage()`, `commitment()` and `nullifierHash()`.
- `NoteString` also exposes read-only `symbol`, `amount` and `chainId` getters.

Byte values use the TypeScript type ``Hex = `0x${string}` ``. Outputs are lowercase.
`chainId` is a bigint in the u64 range. Invalid inputs throw JavaScript errors.
Note operations are synchronous after initialization.

`NoteString` consumes its input `Note`, even if chain ID validation fails.
Use the resulting `NoteString` afterward. Call `free()` when finished with a wrapper;
consumed or freed wrappers must not be reused.

## TypeScript

TypeScript projects need `ESNext.Disposable` in their `compilerOptions.lib`
(alongside their usual libraries) for the generated `Symbol.dispose` declarations.
