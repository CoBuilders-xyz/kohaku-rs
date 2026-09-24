use kohaku_tornadocash::note::Note;
use serde::{Deserialize, Serialize};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

/// Parsed legacy note data, including the original nullifier and secret bytes.
#[derive(Serialize, Deserialize, Tsify)]
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

/// A Tornado note stored in WASM memory until its JavaScript wrapper is freed.
#[wasm_bindgen(js_name = Note)]
pub struct WasmNote {
    inner: Note,
}

#[wasm_bindgen(js_class = Note)]
impl WasmNote {
    /// Construct a note from structured data using the core constructor.
    ///
    /// # Errors
    ///
    /// Throws a JavaScript `Error` if fields cannot be converted to Rust,
    /// including secret lengths other than 31 or a chain ID outside `u64`.
    #[wasm_bindgen(constructor)]
    pub fn new(data: &Ts<ParsedNote>) -> Result<WasmNote, JsError> {
        let data = data.to_rust()?;
        Ok(Self {
            inner: Note::new(
                data.nullifier,
                data.secret,
                data.symbol,
                data.amount,
                data.chain_id,
            ),
        })
    }

    /// Parse a legacy note and retain the resulting core instance.
    ///
    /// # Errors
    ///
    /// Throws a JavaScript `Error` with the core parser's message on failure.
    pub fn parse(text: &str) -> Result<WasmNote, JsError> {
        Ok(Self {
            inner: text.parse()?,
        })
    }

    /// Export an independent copy of the note's fields, including its secrets.
    ///
    /// # Errors
    ///
    /// Throws a JavaScript `Error` if conversion to JavaScript fails.
    #[wasm_bindgen(js_name = toObject)]
    pub fn to_object(&self) -> Result<Ts<ParsedNote>, JsError> {
        let data = ParsedNote {
            symbol: self.inner.symbol.clone(),
            amount: self.inner.amount.clone(),
            chain_id: self.inner.chain_id,
            nullifier: self.inner.nullifier.into_bytes(),
            secret: self.inner.secret.into_bytes(),
        };
        Ok(data.into_ts()?)
    }

    /// Format the note using the core's legacy `Display` representation.
    #[wasm_bindgen(js_name = toString)]
    #[must_use]
    pub fn format(&self) -> String {
        self.inner.to_string()
    }

    /// Compute the commitment as `0x` followed by 64 lowercase hex digits.
    #[must_use]
    pub fn commitment(&self) -> String {
        format!("0x{:064x}", self.inner.commitment())
    }

    /// Compute the nullifier hash as `0x` followed by 64 lowercase hex digits.
    ///
    /// This does not check whether the note has been spent.
    #[wasm_bindgen(js_name = nullifierHash)]
    #[must_use]
    pub fn nullifier_hash(&self) -> String {
        format!("0x{:064x}", self.inner.nullifier_hash())
    }
}
