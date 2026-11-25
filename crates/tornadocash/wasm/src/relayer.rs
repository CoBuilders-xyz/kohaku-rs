use std::{rc::Rc, time::Duration};

use js_sys::Promise;
use kohaku_tornadocash_relayer::Relayer as CoreRelayer;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::{
    Payer, Pool, RelayerStatus,
    bigint::{JsU128, JsU256},
};

/// HTTP client for a Tornado Cash relayer.
#[wasm_bindgen]
pub struct Relayer {
    pub(crate) inner: Rc<CoreRelayer>,
}

#[wasm_bindgen]
impl Relayer {
    #[wasm_bindgen(constructor)]
    pub fn new(
        url: &str,
        #[wasm_bindgen(js_name = timeoutMs)] timeout_ms: Option<crate::bigint::JsU64>,
        #[wasm_bindgen(js_name = pollIntervalMs)] poll_interval_ms: Option<crate::bigint::JsU64>,
    ) -> Result<Self, JsError> {
        let mut client = CoreRelayer::new(url);
        if let Some(ms) = timeout_ms {
            client = client.with_timeout(Duration::from_millis(ms.try_into()?));
        }
        if let Some(ms) = poll_interval_ms {
            client = client.with_poll_interval(Duration::from_millis(ms.try_into()?));
        }
        Ok(Self {
            inner: Rc::new(client),
        })
    }

    /// Fetch the relayer's current status.
    #[wasm_bindgen(unchecked_return_type = "Promise<RelayerStatus>")]
    pub fn status(&self) -> Promise {
        let client = self.inner.clone();
        future_to_promise(async move {
            let inner = client.status().await.map_err(JsError::from)?;
            Ok(RelayerStatus { inner }.into())
        })
    }

    /// Fetch a quote using the relayer's current status.
    #[wasm_bindgen(unchecked_return_type = "Promise<Payer>")]
    pub fn quote(
        &self,
        pool: &Pool,
        #[wasm_bindgen(js_name = gasPrice)] gas_price: JsU128,
        refund: JsU256,
    ) -> Promise {
        let client = self.inner.clone();
        let pool = pool.inner.clone();
        future_to_promise(async move {
            let payer = client
                .quote(&pool, gas_price.try_into()?, refund.try_into()?)
                .await
                .map_err(JsError::from)?;
            Ok(Payer::from(payer).into())
        })
    }
}
