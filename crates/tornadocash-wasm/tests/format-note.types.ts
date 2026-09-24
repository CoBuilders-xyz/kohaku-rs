import { format_note, parse_note, type ParsedNote } from '../../target/tornadocash-wasm-node/kohaku_tornadocash_wasm';

const fields: ParsedNote = {
  symbol: 'eth',
  amount: '0.10',
  chainId: 1n,
  nullifier: new Uint8Array(31),
  secret: new Uint8Array(31),
};

const text: string = format_note(fields);
const roundTrip: string = format_note(parse_note(text));

// @ts-expect-error Formatting accepts structured data, not a note string.
format_note(text);
// @ts-expect-error Chain IDs are bigint in the public TypeScript contract.
format_note({ ...fields, chainId: 1 });
// @ts-expect-error Secret bytes are Uint8Array in the public TypeScript contract.
format_note({ ...fields, secret: [1, 2, 3] });
// @ts-expect-error All note fields are required.
format_note({ symbol: 'eth' });
// @ts-expect-error Formatting returns a string, not ParsedNote or any.
const object: ParsedNote = format_note(fields);
