use js_sys::BigInt;
use kohaku_tornadocash::Deposit as CoreDeposit;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::{bigint::u256_to_bigint, hex::Hex, note::Note, pool::Pool};

/// A Tornado deposit stored in WASM memory until its JavaScript wrapper is freed.
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
    pub fn value(&self) -> BigInt {
        u256_to_bigint(self.inner.value())
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use kohaku_tornadocash::{Note as CoreNote, Pool as CorePool};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::{CoreDeposit, Deposit};
    use crate::{note::Note, pool::Pool};

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
            assert_eq!(
                String::from(deposit.value().to_string(10).unwrap()),
                expected.value().to_string()
            );
        }
    }
}
