use kohaku_tornadocash::note::Note;
use wasm_bindgen::prelude::*;

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
