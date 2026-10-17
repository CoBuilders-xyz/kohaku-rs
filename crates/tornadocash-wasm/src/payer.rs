use alloy_primitives::Address;
use js_sys::BigInt;
use kohaku_tornadocash::Payer as CorePayer;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::{
    bigint::{bigint_to_u256, u256_to_bigint},
    hex::Hex,
};

/// The payer of a withdrawal transaction, with exact fee and refund amounts.
///
/// See [`kohaku_tornadocash::Payer`] for more details.
#[wasm_bindgen]
pub struct Payer {
    pub(crate) inner: CorePayer,
}

#[wasm_bindgen]
impl Payer {
    /// Construct an independently owned payer from its canonical fields.
    ///
    /// # Errors
    /// Throws if address is not a 20-byte Hex or either amount is not a bigint
    /// in `0..=U256::MAX`.
    #[wasm_bindgen(constructor)]
    pub fn new(
        address: &Ts<Hex>,
        #[wasm_bindgen(unchecked_param_type = "bigint")] fee: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "bigint")] refund: &JsValue,
    ) -> Result<Self, JsError> {
        Ok(Self {
            inner: CorePayer {
                address: Address::try_from(address.to_rust()?)?,
                fee: bigint_to_u256(fee)?,
                refund: bigint_to_u256(refund)?,
            },
        })
    }

    /// Return the payer address as checksummed hex.
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> Result<Ts<Hex>, JsError> {
        Ok(Hex::from(self.inner.address).into_ts()?)
    }

    /// Return the fee in the pool asset's base units.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn fee(&self) -> BigInt {
        u256_to_bigint(self.inner.fee)
    }

    /// Return the native-asset refund in base units.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn refund(&self) -> BigInt {
        u256_to_bigint(self.inner.refund)
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use alloy_primitives::U256;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[wasm_bindgen_test]
    fn payer_maps_address_fee_and_refund() {
        let address = Address::with_last_byte(0xab);
        let fee = U256::from(1) << 200;
        let refund = U256::from(7);
        let payer = Payer::new(
            &Hex::from(address).into_ts().unwrap(),
            &u256_to_bigint(fee).into(),
            &u256_to_bigint(refund).into(),
        )
        .unwrap();
        assert_eq!(bigint_to_u256(&payer.fee().into()).unwrap(), fee);
        assert_eq!(bigint_to_u256(&payer.refund().into()).unwrap(), refund);
        let output: JsValue = payer.address().unwrap().into();
        assert_eq!(output.as_string().unwrap(), address.to_string());
    }
}
