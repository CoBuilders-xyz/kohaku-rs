use kohaku_tornadocash::Asset as CoreAsset;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::hex::Hex;

/// A known Tornado asset stored in WASM memory until its JavaScript wrapper is freed.
#[wasm_bindgen]
pub struct Asset {
    pub(crate) inner: CoreAsset,
}

#[wasm_bindgen]
impl Asset {
    /// Return an independently owned wrapper for the core's native ETH asset.
    #[must_use]
    pub fn eth() -> Self {
        Self {
            inner: CoreAsset::ETH,
        }
    }

    /// Return an independently owned wrapper for the core's native MATIC asset.
    #[must_use]
    pub fn matic() -> Self {
        Self {
            inner: CoreAsset::MATIC,
        }
    }

    /// Return an independently owned wrapper for the core's Ethereum DAI token.
    #[wasm_bindgen(js_name = ethereumDai)]
    #[must_use]
    pub fn ethereum_dai() -> Self {
        Self {
            inner: CoreAsset::ETHEREUM_DAI,
        }
    }

    /// Return the asset variant: `native` or `erc20`.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn kind(&self) -> String {
        match &self.inner {
            CoreAsset::Native { .. } => "native",
            CoreAsset::Erc20 { .. } => "erc20",
        }
        .to_owned()
    }

    /// Return the asset symbol.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn symbol(&self) -> String {
        self.inner.symbol().to_owned()
    }

    /// Return the number of decimal places used by the asset.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn decimals(&self) -> u8 {
        self.inner.decimals()
    }

    /// Return the ERC20 token address as hex, or undefined for a native asset.
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> Result<Option<Ts<Hex>>, JsError> {
        match &self.inner {
            CoreAsset::Native { .. } => Ok(None),
            CoreAsset::Erc20 { address, .. } => Ok(Some(Hex::from(*address).into_ts()?)),
        }
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::Asset;

    #[wasm_bindgen_test]
    fn native_assets_have_no_token_address() {
        for asset in [Asset::eth(), Asset::matic()] {
            assert_eq!(asset.kind(), "native");
            let address: JsValue = asset.address().unwrap().into();
            assert!(address.is_undefined());
        }
    }

    #[wasm_bindgen_test]
    fn erc20_asset_exposes_token_address_as_hex() {
        let asset = Asset::ethereum_dai();

        assert_eq!(asset.kind(), "erc20");
        let address: JsValue = asset.address().unwrap().into();
        assert_eq!(
            address.as_string().unwrap(),
            "0x6b175474e89094c44da98b954eedeac495271d0f"
        );
    }
}
