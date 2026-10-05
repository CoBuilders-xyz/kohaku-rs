use crate::{hex::Hex, note::Note};
use kohaku_tornadocash::NoteString as CoreNoteString;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

/// A Tornado note with asset and chain metadata, supporting parsing and
/// formatting in the standard tornado format.
#[wasm_bindgen]
pub struct NoteString {
    inner: CoreNoteString,
}

#[wasm_bindgen]
impl NoteString {
    // NOTE CREATION

    /// Construct a displayable note, taking ownership of the supplied note.
    ///
    /// The supplied note is consumed even if chain ID validation fails.
    ///
    /// # Errors
    ///
    /// Throws if the chain ID is outside the u64 range.
    #[wasm_bindgen(constructor)]
    pub fn new(
        note: Note,
        symbol: &str,
        amount: &str,
        chain_id: js_sys::BigInt,
    ) -> Result<Self, JsError> {
        let chain_id = u64::try_from(chain_id)
            .map_err(|_| JsError::new("chainId must be in the u64 range"))?;

        Ok(Self {
            inner: CoreNoteString::new(note.inner, symbol, amount, chain_id),
        })
    }

    /// Parse a note in the standard Tornado note format.
    ///
    /// # Errors
    ///
    /// Throws if the core parser rejects the input.
    pub fn parse(text: &str) -> Result<Self, JsError> {
        Ok(Self {
            inner: text.parse()?,
        })
    }

    // GETTERS

    /// Return the asset symbol.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn symbol(&self) -> String {
        self.inner.symbol.clone()
    }

    /// Return the amount as text.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn amount(&self) -> String {
        self.inner.amount.clone()
    }

    /// Return the chain Id
    #[wasm_bindgen(getter, js_name = chainId)]
    #[must_use]
    pub fn chain_id(&self) -> u64 {
        self.inner.chain_id
    }

    /// Return the contained note's nullifier encoded as hex.
    #[wasm_bindgen(getter)]
    pub fn nullifier(&self) -> Result<Ts<Hex>, JsError> {
        let hex = Hex::from(self.inner.note.nullifier);

        Ok(hex.into_ts()?)
    }

    /// Return the contained note's secret encoded as hex.
    #[wasm_bindgen(getter)]
    pub fn secret(&self) -> Result<Ts<Hex>, JsError> {
        let hex = Hex::from(self.inner.note.secret);

        Ok(hex.into_ts()?)
    }

    // FORMAT

    /// Format the note using the standard Tornado note representation.
    #[wasm_bindgen(js_name= toString)]
    #[must_use]
    pub fn format(&self) -> String {
        self.inner.to_string()
    }

    // NOTE OPERATIONS

    /// Return the contained note's 62-byte preimage encoded as hex.
    pub fn preimage(&self) -> Result<Ts<Hex>, JsError> {
        let bytes = self.inner.preimage();
        let hex = Hex::from(bytes.as_slice());

        Ok(hex.into_ts()?)
    }

    /// Compute the contained note's commitment encoded as hex.
    pub fn commitment(&self) -> Result<Ts<Hex>, JsError> {
        let hex = Hex::from(self.inner.commitment());

        Ok(hex.into_ts()?)
    }

    /// Compute the contained note's nullifier hash encoded as hex.
    ///
    /// This does not check whether the note has been spent.
    #[wasm_bindgen(js_name= nullifierHash)]
    pub fn nullifier_hash(&self) -> Result<Ts<Hex>, JsError> {
        let hex = Hex::from(self.inner.nullifier_hash());

        Ok(hex.into_ts()?)
    }
}
