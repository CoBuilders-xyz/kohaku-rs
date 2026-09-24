import { parse_note, type ParsedNote } from '../../target/tornadocash-wasm-node/kohaku_tornadocash_wasm';

// Compile-only consumer of the generated declarations; this file is not executed.
const note = parse_note('synthetic note supplied at runtime');
const parsed: ParsedNote = note;
const symbol: string = parsed.symbol;
const amount: string = parsed.amount;
const chainId: bigint = note.chainId;
const nullifier: Uint8Array = note.nullifier;
const secret: Uint8Array = note.secret;

// @ts-expect-error Chain IDs must not silently become numbers or any.
const numericChainId: number = note.chainId;
// @ts-expect-error Secret bytes are typed arrays, not ordinary number arrays.
const secretArray: number[] = note.secret;
// @ts-expect-error Rust snake_case is converted to camelCase in the public object.
note.chain_id;
