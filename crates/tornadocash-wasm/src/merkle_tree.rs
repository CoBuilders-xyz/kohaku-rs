use kohaku_tornadocash::{Field, merkle_tree::MerkleTree as CoreMerkleTree};
use serde::{Deserialize, Serialize};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::{hex::Hex, index::tree_index, merkle_proof::MerkleProof};

/// An ordered JavaScript array of commitment hex strings, with no WASM ownership.
#[derive(Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct Commitments(Vec<Hex>);

/// A canonical depth-20 binary Tornado tree stored in WASM memory until freed.
///
/// See [`kohaku_tornadocash::merkle_tree::MerkleTree`] for the canonical tree type.
#[wasm_bindgen]
#[derive(Default)]
pub struct MerkleTree {
    inner: CoreMerkleTree,
}

#[wasm_bindgen]
impl MerkleTree {
    /// Construct an independently owned empty Tornado tree.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: CoreMerkleTree::new(),
        }
    }

    /// Construct a tree from commitments in their supplied leaf order.
    ///
    /// # Errors
    ///
    /// Throws if any commitment is not an exact 32-byte canonical field element,
    /// or if the core rejects the leaf count.
    #[wasm_bindgen(js_name = fromLeaves)]
    pub fn from_leaves(commitments: &Ts<Commitments>) -> Result<Self, JsError> {
        let leaves = decode_commitments(commitments)?;

        Ok(Self {
            inner: CoreMerkleTree::from_leaves(&leaves)?,
        })
    }

    /// Replace existing leaves or append commitments contiguously from startIndex.
    ///
    /// An empty batch leaves the tree unchanged, including at an index beyond its leaves.
    /// The complete batch is validated before any mutation.
    ///
    /// # Errors
    ///
    /// Throws if startIndex is not a finite integer JS number in the u32 range,
    /// a commitment is invalid, or the core rejects the bounds or capacity.
    pub fn splice(
        &mut self,
        #[wasm_bindgen(js_name = startIndex, unchecked_param_type = "number")]
        start_index: &JsValue,
        commitments: &Ts<Commitments>,
    ) -> Result<(), JsError> {
        let start_index = tree_index(start_index)? as usize;
        let leaves = decode_commitments(commitments)?;
        self.inner.splice(start_index, &leaves)?;
        Ok(())
    }

    /// Return the tree's root as lowercase, 32-byte hex.
    pub fn root(&self) -> Result<Ts<Hex>, JsError> {
        Ok(Hex::from(self.inner.root()?).into_ts()?)
    }

    /// Return an independently owned inclusion proof capturing the current root.
    ///
    /// # Errors
    ///
    /// Throws if index is not a finite integer JS number in the u32 range,
    /// or if the core rejects the leaf index.
    pub fn proof(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] index: &JsValue,
    ) -> Result<MerkleProof, JsError> {
        let index = tree_index(index)? as usize;
        Ok(self.inner.proof(index)?.into())
    }
}

fn decode_commitments(commitments: &Ts<Commitments>) -> Result<Vec<Field>, JsError> {
    Ok(commitments
        .to_rust()?
        .0
        .into_iter()
        .map(Field::try_from)
        .collect::<Result<Vec<_>, _>>()?)
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use tsify::Tsify;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::{Commitments, CoreMerkleTree, Field, Hex, MerkleTree};

    #[wasm_bindgen_test]
    fn empty_construction_returns_the_core_root_as_hex() {
        let expected = CoreMerkleTree::new().root().unwrap().to_string();
        let empty_leaves = Commitments(Vec::new()).into_ts().unwrap();

        for tree in [
            MerkleTree::new(),
            MerkleTree::from_leaves(&empty_leaves).unwrap(),
        ] {
            let root: JsValue = tree.root().unwrap().into();
            assert_eq!(root.as_string().unwrap(), expected);
        }
    }

    #[wasm_bindgen_test]
    fn from_leaves_converts_commitments_in_order_and_delegates_to_core() {
        let leaves = [Field::from(0xab_usize), Field::from(1_usize)];
        let commitments = Commitments(leaves.into_iter().map(Hex::from).collect())
            .into_ts()
            .unwrap();

        let tree = MerkleTree::from_leaves(&commitments).unwrap();

        assert_eq!(tree.inner.leaves(), leaves);
        let root: JsValue = tree.root().unwrap().into();
        assert_eq!(
            root.as_string().unwrap(),
            CoreMerkleTree::from_leaves(&leaves)
                .unwrap()
                .root()
                .unwrap()
                .to_string()
        );
    }

    #[wasm_bindgen_test]
    fn from_leaves_rejects_a_malformed_later_field() {
        let commitments = Commitments(vec![
            Hex::from(Field::from(1_usize)),
            Hex::from([0xab].as_slice()),
        ])
        .into_ts()
        .unwrap();

        assert!(MerkleTree::from_leaves(&commitments).is_err());
    }

    #[wasm_bindgen_test]
    fn splice_converts_the_whole_batch_before_mutating() {
        let initial = Commitments(vec![Hex::from(Field::from(1_usize))])
            .into_ts()
            .unwrap();
        let mut tree = MerkleTree::from_leaves(&initial).unwrap();
        let before = tree.inner.root().unwrap();
        let malformed = Commitments(vec![
            Hex::from(Field::from(2_usize)),
            Hex::from([0xff; 32].as_slice()),
        ])
        .into_ts()
        .unwrap();

        assert!(tree.splice(&JsValue::from(0), &malformed).is_err());
        assert_eq!(tree.inner.root().unwrap(), before);
        assert_eq!(tree.inner.leaves(), &[Field::from(1_usize)]);
    }

    #[wasm_bindgen_test]
    fn splice_converts_and_delegates_replacement_and_append() {
        let initial = Commitments(vec![Hex::from(Field::from(1_usize))])
            .into_ts()
            .unwrap();
        let mut tree = MerkleTree::from_leaves(&initial).unwrap();
        let leaves = [Field::from(2_usize), Field::from(3_usize)];
        let batch = Commitments(leaves.into_iter().map(Hex::from).collect())
            .into_ts()
            .unwrap();

        tree.splice(&JsValue::from(0), &batch).unwrap();

        assert_eq!(tree.inner.leaves(), leaves);
        assert_eq!(
            tree.inner.root().unwrap(),
            CoreMerkleTree::from_leaves(&leaves)
                .unwrap()
                .root()
                .unwrap()
        );
    }

    #[wasm_bindgen_test]
    fn splice_preserves_core_empty_batch_semantics() {
        let mut tree = MerkleTree::new();
        let before = tree.inner.root().unwrap();
        let empty = Commitments(Vec::new()).into_ts().unwrap();

        tree.splice(&JsValue::from(u32::MAX), &empty).unwrap();

        assert_eq!(tree.inner.root().unwrap(), before);
    }

    #[wasm_bindgen_test]
    fn tree_operations_reuse_index_validation_and_core_bounds() {
        let mut tree = MerkleTree::new();
        let batch = Commitments(vec![Hex::from(Field::from(1_usize))])
            .into_ts()
            .unwrap();

        let fractional = JsValue::from_f64(0.5);
        assert!(tree.splice(&fractional, &batch).is_err());
        assert!(tree.proof(&fractional).is_err());

        let missing = JsValue::from(1);
        assert!(tree.splice(&missing, &batch).is_err());
        assert!(tree.proof(&missing).is_err());
    }

    #[wasm_bindgen_test]
    fn proof_delegates_the_selected_leaf_and_encodes_metadata() {
        let leaves = [Field::from(1_usize), Field::from(0xab_usize)];
        let batch = Commitments(leaves.into_iter().map(Hex::from).collect())
            .into_ts()
            .unwrap();
        let tree = MerkleTree::from_leaves(&batch).unwrap();

        let proof = tree.proof(&JsValue::from(1)).unwrap();

        let leaf: JsValue = proof.leaf().unwrap().into();
        let root: JsValue = proof.root().unwrap().into();
        assert_eq!(leaf.as_string().unwrap(), leaves[1].to_string());
        assert_eq!(
            root.as_string().unwrap(),
            tree.inner.root().unwrap().to_string()
        );
        assert!(proof.verify());
    }
}
