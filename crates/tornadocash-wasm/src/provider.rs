use alloy::{providers::RootProvider, transports::http::reqwest::Url};
use js_sys::{Array, Promise};
use kohaku_tornadocash::{Field, TornadoProviderExt};
use tsify::Ts;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::{
    Pool,
    bigint::{bigint_to_u256, u256_to_bigint},
    hex::Hex,
};

/// Read-only Tornado pool queries through an HTTP(S) RPC endpoint.
///
/// See [`kohaku_tornadocash::TornadoProviderExt`] for canonical query semantics.
/// Calls snapshot their arguments and retain a provider handle before returning
/// a Promise, so wrappers may be freed while a request is pending.
#[wasm_bindgen]
pub struct TornadoProvider {
    inner: RootProvider,
}

#[wasm_bindgen]
impl TornadoProvider {
    /// Construct an Alloy HTTP provider without making a network request.
    ///
    /// # Errors
    ///
    /// Throws if `url` is invalid or does not use HTTP(S).
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Result<Self, JsError> {
        let url = Url::parse(url)?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(JsError::new("RPC URL must use http or https"));
        }
        Ok(Self {
            inner: RootProvider::new_http(url),
        })
    }

    /// Query whether a nullifier hash has been spent.
    ///
    /// Rejects on invalid field hex or an RPC failure.
    #[wasm_bindgen(js_name = isSpent, unchecked_return_type = "Promise<boolean>")]
    pub fn is_spent(&self, pool: &Pool, hash: &Ts<Hex>) -> Promise {
        let hash = decode_field(hash);
        let pool = pool.inner.clone();
        let provider = self.inner.clone();
        future_to_promise(async move {
            let result = provider
                .is_spent(&pool, hash?)
                .await
                .map_err(JsError::from)?;
            Ok(JsValue::from(result))
        })
    }

    /// Query spent status in the same length and order as the supplied hashes.
    ///
    /// Rejects on invalid field hex or an RPC failure.
    #[wasm_bindgen(js_name = isSpentArray, unchecked_return_type = "Promise<boolean[]>")]
    pub fn is_spent_array(&self, pool: &Pool, hashes: Vec<Ts<Hex>>) -> Promise {
        let hashes: Result<Vec<_>, _> = hashes
            .into_iter()
            .map(|value| decode_field(&value))
            .collect();
        let pool = pool.inner.clone();
        let provider = self.inner.clone();
        future_to_promise(async move {
            let result = provider
                .is_spent_array(&pool, &hashes?)
                .await
                .map_err(JsError::from)?;
            Ok(result
                .into_iter()
                .map(JsValue::from)
                .collect::<Array>()
                .into())
        })
    }

    /// Query whether a Merkle root is known to the pool.
    ///
    /// Rejects on invalid field hex or an RPC failure.
    #[wasm_bindgen(js_name = isKnownRoot, unchecked_return_type = "Promise<boolean>")]
    pub fn is_known_root(&self, pool: &Pool, root: &Ts<Hex>) -> Promise {
        let root = decode_field(root);
        let pool = pool.inner.clone();
        let provider = self.inner.clone();
        future_to_promise(async move {
            let result = provider
                .is_known_root(&pool, root?)
                .await
                .map_err(JsError::from)?;
            Ok(JsValue::from(result))
        })
    }

    /// Quote an exact wei amount in the pool's fee token.
    ///
    /// Native assets return the input without RPC. Rejects unless `weiAmount` is
    /// a bigint in `0..=U256::MAX`, or if the ERC20 RPC query fails.
    #[wasm_bindgen(js_name = quoteWeiInFeeToken, unchecked_return_type = "Promise<bigint>")]
    pub fn quote_wei_in_fee_token(
        &self,
        pool: &Pool,
        #[wasm_bindgen(js_name = weiAmount, unchecked_param_type = "bigint")] wei_amount: &JsValue,
    ) -> Promise {
        let amount = bigint_to_u256(wei_amount);
        let pool = pool.inner.clone();
        let provider = self.inner.clone();
        future_to_promise(async move {
            let result = provider
                .quote_wei_in_fee_token(&pool, amount?)
                .await
                .map_err(JsError::from)?;
            Ok(JsValue::from(u256_to_bigint(result)))
        })
    }
}

fn decode_field(value: &Ts<Hex>) -> Result<Field, JsError> {
    Ok(value.to_rust()?.try_into()?)
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use kohaku_tornadocash::Pool as CorePool;
    use tsify::Tsify;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    fn pool() -> Pool {
        Pool {
            inner: CorePool::ETHEREUM_ETHER_01,
        }
    }

    #[wasm_bindgen_test]
    fn restricts_transport_to_http_schemes() {
        for url in ["http://127.0.0.1:1", "https://example.invalid/rpc"] {
            assert!(TornadoProvider::new(url).is_ok());
        }
        assert!(TornadoProvider::new("ws://localhost:8545").is_err());
    }

    #[wasm_bindgen_test]
    async fn query_rejects_invalid_array_element_before_rpc() {
        let provider = TornadoProvider::new("http://127.0.0.1:1").unwrap();
        let valid = Hex::from(Field::from(1_usize)).into_ts().unwrap();
        let short = Hex::from([0; 31].as_slice()).into_ts().unwrap();
        let error = JsFuture::from(provider.is_spent_array(&pool(), vec![valid, short]))
            .await
            .unwrap_err();
        assert!(
            js_sys::Error::from(error)
                .message()
                .includes("Invalid byte length", 0)
        );
    }

    #[wasm_bindgen_test]
    async fn quote_rejects_number_instead_of_coercing_it() {
        let provider = TornadoProvider::new("http://127.0.0.1:1").unwrap();
        let error =
            JsFuture::from(provider.quote_wei_in_fee_token(&pool(), &JsValue::from_f64(1.0)))
                .await
                .unwrap_err();
        assert!(js_sys::Error::from(error).message().includes("bigint", 0));
    }
}
