use js_sys::Promise;
use kohaku_tornadocash_relayer::{JobReceipt as CoreJobReceipt, JobStatus};
use serde::Serialize;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::{Pool, ProvenWithdrawal, Relayer, TornadoProvider, hex::Hex};

/// A submitted relayer job. The receipt can be persisted between status checks.
#[wasm_bindgen]
pub struct JobReceipt {
    pub(crate) inner: CoreJobReceipt,
}

#[wasm_bindgen]
impl JobReceipt {
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn id(&self) -> String {
        self.inner.id.to_string()
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn pool(&self) -> Pool {
        self.inner.pool.clone().into()
    }

    #[wasm_bindgen(getter, js_name = nullifierHash)]
    pub fn nullifier_hash(&self) -> Result<Ts<Hex>, JsError> {
        Ok(Hex::from(self.inner.nullifier_hash).into_ts()?)
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<String, JsError> {
        Ok(serde_json::to_string(&self.inner)?)
    }

    #[wasm_bindgen(js_name = fromJSON)]
    pub fn from_json(json: &str) -> Result<Self, JsError> {
        Ok(Self {
            inner: serde_json::from_str(json)?,
        })
    }
}

/// Only `spent` comes from the chain. The relayer reports the hash and job state.
#[derive(Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalStatus {
    spent: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tx_hash: Option<Hex>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tsify(
        type = "\"QUEUED\" | \"ACCEPTED\" | \"SENT\" | \"MINED\" | \"RESUBMITTED\" | \"CONFIRMED\" | \"FAILED\""
    )]
    job: Option<JobStatus>,
}

#[wasm_bindgen]
impl Relayer {
    /// Submit an independent copy of the proven withdrawal.
    #[wasm_bindgen(unchecked_return_type = "Promise<JobReceipt>")]
    pub fn withdraw(&self, withdrawal: &ProvenWithdrawal) -> Promise {
        let client = self.inner.clone();
        let withdrawal = withdrawal.inner.clone();
        future_to_promise(async move {
            Ok(JobReceipt {
                inner: client.withdraw(withdrawal).await.map_err(JsError::from)?,
            }
            .into())
        })
    }

    #[wasm_bindgen(unchecked_return_type = "Promise<WithdrawalStatus>")]
    pub fn check(&self, provider: &TornadoProvider, receipt: &JobReceipt) -> Promise {
        let client = self.inner.clone();
        let provider = provider.inner.clone();
        let receipt = receipt.inner.clone();
        future_to_promise(async move {
            let status = client
                .check(&provider, &receipt)
                .await
                .map_err(JsError::from)?;
            Ok(WithdrawalStatus {
                spent: status.spent,
                tx_hash: status.tx_hash.map(|hash| Hex::from(hash.as_slice())),
                job: status.job,
            }
            .into_ts()
            .map_err(JsError::from)?
            .into())
        })
    }

    /// Wait for on-chain confirmation. The relayer may omit the transaction hash.
    #[wasm_bindgen(js_name = awaitConfirmation, unchecked_return_type = "Promise<Hex | undefined>")]
    pub fn await_confirmation(&self, provider: &TornadoProvider, receipt: &JobReceipt) -> Promise {
        let client = self.inner.clone();
        let provider = provider.inner.clone();
        let receipt = receipt.inner.clone();
        future_to_promise(async move {
            let hash = client
                .await_confirmation(&provider, &receipt)
                .await
                .map_err(JsError::from)?;
            Ok(match hash {
                Some(hash) => Hex::from(hash.as_slice())
                    .into_ts()
                    .map_err(JsError::from)?
                    .into(),
                None => JsValue::UNDEFINED,
            })
        })
    }
}
