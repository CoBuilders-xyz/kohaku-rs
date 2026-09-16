use std::str::FromStr;

use kohaku_tornadocash::note::Note;
use wasm_bindgen::prelude::*;

// Export one operation; parsing and hashing stay in the real SDK.
#[wasm_bindgen]
pub fn note_commitment(encoded: &str) -> Result<String, JsError> {
    let note = Note::from_str(encoded).map_err(|error| JsError::new(&error.to_string()))?;
    Ok(format!("{:#066x}", note.commitment()))
}
