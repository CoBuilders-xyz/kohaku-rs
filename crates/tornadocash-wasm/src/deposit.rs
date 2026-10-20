use alloy::primitives::TxKind;
use kohaku_tornadocash::Deposit as CoreDeposit;
use serde::Serialize;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::{bigint::JsU256, hex::Hex, note::Note, pool::Pool};

/// Plain target, calldata and optional value from a core approval request.
#[derive(Serialize, Tsify)]
pub struct TransactionData {
    to: Hex,
    data: Hex,
    #[serde(
        with = "tsify::serde_wasm_bindgen::preserve",
        skip_serializing_if = "JsValue::is_undefined"
    )]
    #[tsify(optional, type = "bigint")]
    value: JsValue,
}

/// A Tornado Cash deposit.
///
/// See [`kohaku_tornadocash::Deposit`] for more details.
#[wasm_bindgen]
pub struct Deposit {
    inner: CoreDeposit,
}

#[wasm_bindgen]
impl Deposit {
    /// Construct an independently owned deposit without consuming the pool or note.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(pool: &Pool, note: &Note) -> Self {
        Self {
            inner: CoreDeposit::new(&pool.inner, note.inner.clone()),
        }
    }

    /// Return the deposit transaction's target address as hex.
    pub fn target(&self) -> Result<Ts<Hex>, JsError> {
        Ok(Hex::from(self.inner.target()).into_ts()?)
    }

    /// Return the core-encoded deposit calldata as hex.
    pub fn input(&self) -> Result<Ts<Hex>, JsError> {
        Ok(Hex::from(self.inner.input().as_slice()).into_ts()?)
    }

    /// Return the exact native value required by the core, or zero for ERC20 pools.
    #[must_use]
    pub fn value(&self) -> JsU256 {
        self.inner.value().into()
    }

    /// Return the core's ERC20 approval as a plain snapshot, or undefined for native pools.
    ///
    /// An absent core value remains absent. This does not check existing allowance.
    pub fn approval(&self) -> Result<Option<Ts<TransactionData>>, JsError> {
        let Some(approval) = self.inner.approval() else {
            return Ok(None);
        };
        let Some(TxKind::Call(to)) = approval.to else {
            return Err(JsError::new("Core approval must contain a target address"));
        };
        let Some(data) = approval.input.into_input() else {
            return Err(JsError::new("Core approval must contain calldata"));
        };

        Ok(Some(
            TransactionData {
                to: Hex::from(to),
                data: Hex::from(data.as_ref()),
                value: approval
                    .value
                    .map_or(JsValue::UNDEFINED, |value| JsU256::from(value).into()),
            }
            .into_ts()?,
        ))
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use alloy::primitives::{TxKind, U256};
    use js_sys::{Object, Reflect};
    use kohaku_tornadocash::{Note as CoreNote, Pool as CorePool};
    use tsify::Tsify;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::{CoreDeposit, Deposit};
    use crate::{hex::Hex, note::Note, pool::Pool};

    #[wasm_bindgen_test]
    fn deposit_reads_map_core_values_for_native_and_erc20() {
        for core_pool in [CorePool::ETHEREUM_ETHER_01, CorePool::ETHEREUM_DAI_100] {
            let core_note = CoreNote::new([1_u8; 31], [2_u8; 31]);
            let expected = CoreDeposit::new(&core_pool, core_note.clone());
            let pool = Pool { inner: core_pool };
            let note = Note { inner: core_note };
            let deposit = Deposit::new(&pool, &note);

            assert_eq!(
                deposit
                    .target()
                    .unwrap()
                    .to_rust()
                    .unwrap()
                    .to_bytes()
                    .unwrap(),
                expected.target().as_slice()
            );
            assert_eq!(
                deposit
                    .input()
                    .unwrap()
                    .to_rust()
                    .unwrap()
                    .to_bytes()
                    .unwrap(),
                expected.input()
            );
            assert_eq!(U256::try_from(deposit.value()).unwrap(), expected.value());
        }
    }

    #[wasm_bindgen_test]
    fn native_approval_is_undefined() {
        let (deposit, _) = deposit_fixture(CorePool::ETHEREUM_ETHER_01);
        let approval = deposit.approval().unwrap();
        let approval: JsValue = approval.into();

        assert!(approval.is_undefined());
    }

    #[wasm_bindgen_test]
    fn erc20_approval_maps_core_request_and_preserves_absent_value() {
        let (deposit, core) = deposit_fixture(CorePool::ETHEREUM_DAI_100);
        let expected = core.approval().unwrap();

        let snapshot: JsValue = deposit.approval().unwrap().unwrap().into();
        let mut keys = Object::keys(&Object::from(snapshot.clone()))
            .iter()
            .map(|key| key.as_string().unwrap())
            .collect::<Vec<_>>();
        keys.sort();
        assert_eq!(keys, ["data", "to"]);

        let Some(TxKind::Call(address)) = expected.to else {
            panic!("expected a core transaction target address");
        };
        let expected_to: JsValue = Hex::from(address).into_ts().unwrap().into();
        let to = Reflect::get(&snapshot, &JsValue::from_str("to")).unwrap();
        assert_eq!(to, expected_to);
        let data = Reflect::get(&snapshot, &JsValue::from_str("data")).unwrap();
        assert_eq!(
            data.as_string().unwrap(),
            format!("0x{}", hex::encode(expected.input.input().unwrap()))
        );
        assert!(
            Reflect::get(&snapshot, &JsValue::from_str("value"))
                .unwrap()
                .is_undefined()
        );
    }

    fn deposit_fixture(core_pool: CorePool) -> (Deposit, CoreDeposit) {
        let core_note = CoreNote::new([1_u8; 31], [2_u8; 31]);
        let core = CoreDeposit::new(&core_pool, core_note.clone());
        let pool = Pool { inner: core_pool };
        let note = Note { inner: core_note };

        (Deposit::new(&pool, &note), core)
    }
}
