use std::collections::HashMap;

use js_sys::Map;
use kohaku_tornadocash::relayer::RelayerStatus as CoreRelayerStatus;
use serde::Serialize;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::{
    bigint::{bigint_to_u256, u256_to_bigint},
    hex::Hex,
    payer::Payer,
    pool::Pool,
};

/// A caller-supplied relayer status, used for offline pool support and fee quotes.
///
/// See [`kohaku_tornadocash::relayer::RelayerStatus`] for more details.
#[wasm_bindgen]
pub struct RelayerStatus {
    inner: CoreRelayerStatus,
}

/// A snapshot of the pool instances supported for one asset.
#[derive(Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[expect(
    clippy::struct_field_names,
    reason = "preserves the canonical Instance field name"
)]
pub struct Instance {
    instance_address: HashMap<String, Hex>,
    symbol: String,
    decimals: u8,
}

/// A snapshot of a relayer's supported instances, keyed by the core's asset key.
#[derive(Serialize, Tsify)]
pub struct Instances(HashMap<String, Instance>);

/// A snapshot of the relayer's reported health, preserving absent values.
#[derive(Serialize, Tsify)]
pub struct Health {
    status: Option<String>,
    error: Option<String>,
}

#[wasm_bindgen]
impl RelayerStatus {
    /// Parse the original JSON response body using the core's serde contract.
    ///
    /// Supply raw text to preserve integers before JavaScript number rounding.
    /// # Errors
    /// Throws if the body cannot be deserialized as a canonical relayer status.
    pub fn parse(json: &str) -> Result<Self, JsError> {
        Ok(Self {
            inner: serde_json::from_str(json)?,
        })
    }

    /// Return the reward account as checksummed hex.
    #[wasm_bindgen(getter, js_name = rewardAccount)]
    pub fn reward_account(&self) -> Result<Ts<Hex>, JsError> {
        Ok(Hex::from(self.inner.reward_account).into_ts()?)
    }

    /// Return a detached Map of assets to typed instance metadata.
    #[wasm_bindgen(getter)]
    pub fn instances(&self) -> Result<Ts<Instances>, JsError> {
        Ok(Instances(
            self.inner
                .instances
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        Instance {
                            instance_address: value
                                .instance_address
                                .iter()
                                .map(|(amount, address)| (amount.clone(), Hex::from(*address)))
                                .collect(),
                            symbol: value.symbol.clone(),
                            decimals: value.decimals,
                        },
                    )
                })
                .collect(),
        )
        .into_ts()?)
    }

    /// Return the network ID as an exact bigint.
    #[wasm_bindgen(getter, js_name = netId)]
    #[must_use]
    pub fn net_id(&self) -> u64 {
        self.inner.net_id
    }

    /// Return a detached Map of exact token prices, keyed by asset symbol.
    #[wasm_bindgen(getter, js_name = ethPrices, unchecked_return_type = "Map<string, bigint>")]
    #[must_use]
    pub fn eth_prices(&self) -> Map {
        let prices = Map::new();
        for (symbol, price) in &self.inner.eth_prices {
            prices.set(&JsValue::from_str(symbol), &u256_to_bigint(*price));
        }
        prices
    }

    /// Return the reported Tornado service fee percentage.
    #[wasm_bindgen(getter, js_name = tornadoServiceFee)]
    #[must_use]
    pub fn tornado_service_fee(&self) -> f64 {
        self.inner.tornado_service_fee
    }

    /// Return the reported mining service fee percentage.
    #[wasm_bindgen(getter, js_name = miningServiceFee)]
    #[must_use]
    pub fn mining_service_fee(&self) -> f64 {
        self.inner.mining_service_fee
    }

    /// Return the reported relayer version.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn version(&self) -> String {
        self.inner.version.clone()
    }

    /// Return detached health metadata with undefined for absent values.
    #[wasm_bindgen(getter)]
    pub fn health(&self) -> Result<Ts<Health>, JsError> {
        Ok(Health {
            status: self.inner.health.status.clone(),
            error: self.inner.health.error.clone(),
        }
        .into_ts()?)
    }

    /// Return the reported queue size.
    #[wasm_bindgen(getter, js_name = currentQueue)]
    #[must_use]
    pub fn current_queue(&self) -> u32 {
        self.inner.current_queue
    }

    /// Check pool support using the core, borrowing the pool without consuming it.
    #[must_use]
    pub fn supports(&self, pool: &Pool) -> bool {
        self.inner.supports(&pool.inner)
    }

    /// Return an independently owned canonical payer, borrowing the pool.
    ///
    /// # Errors
    /// Throws for invalid bigint widths, unsupported pools, or core quote errors
    /// (a zero ERC20 price or arithmetic overflow).
    pub fn quote(
        &self,
        pool: &Pool,
        #[wasm_bindgen(js_name = gasPrice, unchecked_param_type = "bigint")] gas_price: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "bigint")] refund: &JsValue,
    ) -> Result<Payer, JsError> {
        let gas_price = u128::try_from(gas_price.clone())
            .map_err(|_| JsError::new("gasPrice must be a bigint in the u128 range"))?;
        let refund = bigint_to_u256(refund)?;
        Ok(Payer {
            inner: self.inner.quote(&pool.inner, gas_price, refund)?,
        })
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use alloy_primitives::{Address, U256};
    use js_sys::Reflect;
    use kohaku_tornadocash::{
        Pool as CorePool,
        relayer::{Health as CoreHealth, Instance as CoreInstance},
    };
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[wasm_bindgen_test]
    fn metadata_maps_prices_instances_and_optional_health() {
        let address = Address::with_last_byte(0xab);
        let price = U256::from(1) << 200;
        let status = RelayerStatus {
            inner: CoreRelayerStatus {
                eth_prices: HashMap::from([("token".to_owned(), price)]),
                health: CoreHealth {
                    status: Some("ok".to_owned()),
                    error: None,
                },
                instances: HashMap::from([(
                    "token".to_owned(),
                    CoreInstance {
                        instance_address: HashMap::from([("7".to_owned(), address)]),
                        symbol: "TKN".to_owned(),
                        decimals: 6,
                    },
                )]),
                ..Default::default()
            },
        };
        let key = JsValue::from_str("token");
        let prices = status.eth_prices();
        assert_eq!(bigint_to_u256(&prices.get(&key)).unwrap(), price);
        prices.clear();
        assert_eq!(status.eth_prices().size(), 1);

        let instances: JsValue = status.instances().unwrap().into();
        let instance = Map::from(instances).get(&key);
        assert_eq!(
            Reflect::get(&instance, &JsValue::from_str("symbol")).unwrap(),
            JsValue::from_str("TKN")
        );
        assert_eq!(
            Reflect::get(&instance, &JsValue::from_str("decimals")).unwrap(),
            JsValue::from_f64(6.0)
        );
        let addresses = Reflect::get(&instance, &JsValue::from_str("instanceAddress")).unwrap();
        assert_eq!(
            Map::from(addresses)
                .get(&JsValue::from_str("7"))
                .as_string()
                .unwrap(),
            address.to_string()
        );
        let health: JsValue = status.health().unwrap().into();
        assert_eq!(
            Reflect::get(&health, &JsValue::from_str("status")).unwrap(),
            JsValue::from_str("ok")
        );
        assert!(
            Reflect::get(&health, &JsValue::from_str("error"))
                .unwrap()
                .is_undefined()
        );
    }

    #[wasm_bindgen_test]
    fn quote_checks_gas_price_width() {
        let pool = Pool {
            inner: CorePool::ETHEREUM_ETHER_1,
        };
        // Only the support metadata needed to reach our successful narrowing path.
        let status = RelayerStatus {
            inner: CoreRelayerStatus {
                net_id: pool.inner.chain_id,
                instances: HashMap::from([(
                    pool.inner.symbol().to_owned(),
                    CoreInstance {
                        instance_address: HashMap::from([(
                            pool.inner.amount(),
                            pool.inner.address,
                        )]),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            },
        };
        let zero = JsValue::from(0_u128);
        assert!(
            status
                .quote(&pool, &JsValue::from(u128::MAX), &zero)
                .is_ok()
        );
        assert!(
            status
                .quote(
                    &pool,
                    &JsValue::bigint_from_str("340282366920938463463374607431768211456"),
                    &zero
                )
                .is_err()
        );
    }
}
