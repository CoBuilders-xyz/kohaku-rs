use kohaku_tornadocash::note::Note;
use serde::Serialize;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

/// Parsed legacy note data, including the original nullifier and secret bytes.
#[derive(Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(large_number_types_as_bigints)]
pub struct ParsedNote {
    pub symbol: String,
    pub amount: String,
    pub chain_id: u64,
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    pub nullifier: [u8; 31],
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    pub secret: [u8; 31],
}

/// Parse a legacy note into a plain JavaScript object.
///
/// # Errors
///
/// Throws a JavaScript `Error` if parsing or conversion to JavaScript fails.
#[wasm_bindgen]
pub fn parse_note(note: &str) -> Result<Ts<ParsedNote>, JsError> {
    let note: Note = note.parse()?;
    let parsed = ParsedNote {
        symbol: note.symbol,
        amount: note.amount,
        chain_id: note.chain_id,
        nullifier: note.nullifier.into_bytes(),
        secret: note.secret.into_bytes(),
    };
    Ok(parsed.into_ts()?)
}

/// Compute a note's commitment as `0x` followed by 64 lowercase hex digits.
///
/// # Errors
///
/// Throws a JavaScript `Error` if the core cannot parse the note.
#[wasm_bindgen]
pub fn note_commitment(note: &str) -> Result<String, JsError> {
    let note: Note = note.parse()?;
    Ok(format!("0x{:064x}", note.commitment()))
}

/// Compute a note's nullifier hash as `0x` followed by 64 lowercase hex digits.
///
/// This does not check whether the note has been spent.
///
/// # Errors
///
/// Throws a JavaScript `Error` if the core cannot parse the note.
#[wasm_bindgen]
pub fn note_nullifier_hash(note: &str) -> Result<String, JsError> {
    let note: Note = note.parse()?;
    Ok(format!("0x{:064x}", note.nullifier_hash()))
}
