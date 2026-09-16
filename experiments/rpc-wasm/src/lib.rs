use std::time::Duration;

use alloy::{
    primitives::{address, keccak256},
    providers::ProviderBuilder,
};
use kohaku_tornadocash::{
    indexer::{
        rpc::RpcSyncer,
        syncer::{SyncEvent, SyncerBackend},
    },
    pool::{Asset, Pool},
};
use serde_json::json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn report_panic(message: &str);
}

#[wasm_bindgen(start)]
pub fn start() {
    // Diagnostic only: a panic is still fatal to the isolated test process.
    std::panic::set_hook(Box::new(|info| report_panic(&info.to_string())));
}

// Signature copied from the inspected SDK ABI; hash it with Alloy, not JS.
#[wasm_bindgen]
pub fn deposit_topic() -> String {
    keccak256(b"Deposit(bytes32,uint32,uint256)").to_string()
}

#[wasm_bindgen]
pub async fn sync_fixture(port: u16, scenario: u8) -> Result<String, JsError> {
    let route = match scenario {
        0 => "success",
        1 => "fail-first",
        2 => "fail-second",
        _ => return Err(JsError::new("unknown fixture scenario")),
    };
    // No caller-supplied hosts, credentials, or real network endpoints.
    let url = format!("http://127.0.0.1:{port}/{route}")
        .parse()
        .map_err(|error| JsError::new(&format!("invalid local URL: {error}")))?;
    let provider = ProviderBuilder::new()
        .disable_recommended_fillers()
        .connect_http(url);
    let syncer = RpcSyncer::new(provider)
        .with_batch_size(2)
        .with_batch_delay(Duration::from_millis(100));
    let pool = Pool {
        chain_id: 31337,
        address: address!("0x1111111111111111111111111111111111111111"),
        asset: Asset::ETH,
        amount_wei: 100_000_000_000_000_000,
        deployed_block: 100,
        paymaster_address: None,
        adapter_address: None,
    };

    // Explicitly exercise the SDK trait implementation adapted in Lab 09.
    let events = SyncerBackend::sync(&syncer, &pool, 100, 105)
        .await
        .map_err(|error| JsError::new(&error.to_string()))?;
    let rows: Vec<_> = events
        .into_iter()
        .map(|event| match event {
            SyncEvent::Deposit(deposit) => json!({
                "kind": "deposit",
                "commitment": deposit.commitment.to_string(),
                "leafIndex": deposit.leafIndex,
                "timestamp": deposit.timestamp.to_string(),
            }),
            SyncEvent::Withdrawal(_) => json!({"kind": "withdrawal"}),
        })
        .collect();
    Ok(json!(rows).to_string())
}
