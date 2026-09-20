//! Tornadocash pool deployment.

use alloy::{
    primitives::{Address, U256},
    providers::{DynProvider, Provider},
};
use kohaku_tornadocash::pool::{Asset, Pool};

mod sol {
    use alloy::sol;

    sol!(
        #[sol(rpc)]
        Hasher,
        "fixtures/hasher.json"
    );
    sol!(
        #[sol(rpc)]
        Verifier,
        "fixtures/verifier.json"
    );
    sol!(
        #[sol(rpc)]
        ETHTornado,
        "fixtures/eth_tornado.json"
    );
    sol!(
        #[sol(rpc)]
        TornadoProxyLight,
        "fixtures/tornado_proxy_light.json"
    );
}

const DENOMINATION_WEI: u128 = 10_u128.pow(17);
const MERKLE_TREE_HEIGHT: u32 = 20;

/// Deploys fresh tornadocash pool contracts.
///
/// Returns the deployed [`Pool`].
///
/// # Errors
/// Returns an error if any contract fails to deploy.
pub async fn deploy_pool(provider: DynProvider) -> Result<Pool, anyhow::Error> {
    let hasher = sol::Hasher::deploy(&provider).await?;
    let verifier = sol::Verifier::deploy(&provider).await?;
    let tornado = sol::ETHTornado::deploy(
        &provider,
        *verifier.address(),
        *hasher.address(),
        U256::from(DENOMINATION_WEI),
        MERKLE_TREE_HEIGHT,
    )
    .await?;

    Ok(Pool {
        chain_id: provider.get_chain_id().await?,
        address: *tornado.address(),
        asset: Asset::ETH,
        amount_wei: DENOMINATION_WEI,
        deployed_block: 0,
        paymaster_address: None,
        adapter_address: None,
    })
}

/// Deploy a fresh `TornadoProxyLight` to `provider` and return its address.
///
/// # Errors
/// Returns an error if the contract fails to deploy.
pub async fn deploy_proxy(provider: DynProvider) -> Result<Address, anyhow::Error> {
    let proxy = sol::TornadoProxyLight::deploy(&provider).await?;
    Ok(*proxy.address())
}
