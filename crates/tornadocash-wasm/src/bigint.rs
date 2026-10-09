use alloy_primitives::U256;
use js_sys::BigInt;
use wasm_bindgen::{JsCast, JsError, JsValue};

/// Convert an unsigned 256-bit value to an exact JavaScript bigint.
#[allow(dead_code, reason = "shared helper for deposit and event bindings")]
pub(crate) fn u256_to_bigint(value: U256) -> BigInt {
    JsValue::bigint_from_str(&value.to_string()).unchecked_into()
}

/// Decode an optional log block number without coercion or truncation.
#[allow(dead_code, reason = "shared helper for event bindings")]
pub(crate) fn optional_block_number(value: &JsValue) -> Result<Option<u64>, JsError> {
    if value.is_undefined() {
        return Ok(None);
    }

    u64::try_from(value.clone())
        .map(Some)
        .map_err(|_| JsError::new("blockNumber must be a bigint in the u64 range"))
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use alloy_primitives::U256;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::{optional_block_number, u256_to_bigint};

    #[wasm_bindgen_test]
    fn converts_u256_without_truncating_to_128_bits() {
        let value = u256_to_bigint(U256::MAX);

        assert_eq!(
            String::from(value.to_string(10).unwrap()),
            "115792089237316195423570985008687907853269984665640564039457584007913129639935"
        );
    }

    #[wasm_bindgen_test]
    fn accepts_absent_or_exact_block_number() {
        assert_eq!(optional_block_number(&JsValue::UNDEFINED).unwrap(), None);
        assert_eq!(
            optional_block_number(&JsValue::from(u64::MAX)).unwrap(),
            Some(u64::MAX)
        );
    }

    #[wasm_bindgen_test]
    fn rejects_invalid_present_block_numbers() {
        for value in [
            JsValue::NULL,
            JsValue::from_f64(1.0),
            JsValue::bigint_from_str("18446744073709551616"),
        ] {
            assert!(optional_block_number(&value).is_err());
        }
    }
}
