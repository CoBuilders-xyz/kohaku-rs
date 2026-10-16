use kohaku_tornadocash::merkle_tree::{MerkleProof as CoreMerkleProof, TornadoHasher};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::hex::Hex;

/// An owned canonical inclusion proof, valid against its captured root.
///
/// Obtain this wrapper from `MerkleTree.proof()` and free it when finished.
/// Updating or freeing the source tree does not change this proof.
#[wasm_bindgen]
pub struct MerkleProof {
    inner: CoreMerkleProof,
}

impl From<CoreMerkleProof> for MerkleProof {
    fn from(inner: CoreMerkleProof) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
impl MerkleProof {
    /// Return the captured leaf as lowercase, 32-byte hex.
    #[wasm_bindgen(getter)]
    pub fn leaf(&self) -> Result<Ts<Hex>, JsError> {
        Ok(Hex::from(self.inner.leaf).into_ts()?)
    }

    /// Return the captured root as lowercase, 32-byte hex.
    #[wasm_bindgen(getter)]
    pub fn root(&self) -> Result<Ts<Hex>, JsError> {
        Ok(Hex::from(self.inner.root).into_ts()?)
    }

    /// Verify inclusion against the captured root using the canonical Tornado hasher.
    ///
    /// This does not establish on-chain root acceptance or whether a note is unspent.
    #[must_use]
    pub fn verify(&self) -> bool {
        self.inner.verify::<TornadoHasher>()
    }
}
