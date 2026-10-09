use kohaku_tornadocash::Pool as CorePool;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::hex::Hex;

/// A known Tornado pool stored in WASM memory until its JavaScript wrapper is freed.
#[wasm_bindgen]
pub struct Pool {
    inner: CorePool,
}

#[wasm_bindgen]
impl Pool {
    /// Return independently owned wrappers for every entry in the core pool catalog.
    #[must_use]
    pub fn known() -> Vec<Self> {
        CorePool::POOLS
            .iter()
            .cloned()
            .map(|inner| Self { inner })
            .collect()
    }

    /// Return the pool's chain ID.
    #[wasm_bindgen(getter, js_name = chainId)]
    #[must_use]
    pub fn chain_id(&self) -> u64 {
        self.inner.chain_id
    }

    /// Return the pool contract address as hex.
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> Result<Ts<Hex>, JsError> {
        Ok(Hex::from(self.inner.address).into_ts()?)
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::{CorePool, Pool};

    #[wasm_bindgen_test]
    fn pool_address_is_lowercase_hex() {
        let pool = Pool {
            inner: CorePool::ETHEREUM_ETHER_01,
        };
        let expected = pool.inner.address.to_string().to_ascii_lowercase();

        let address: JsValue = pool.address().unwrap().into();
        assert_eq!(address.as_string().unwrap(), expected);
    }
}
