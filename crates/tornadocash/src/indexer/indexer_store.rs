use kohaku_kv_store::{Store, backend::StoreError};

const LATEST_BLOCK_KEY: &[u8] = b"latest_block";

#[cfg_attr(native, async_trait::async_trait)]
#[cfg_attr(wasm, async_trait::async_trait(?Send))]
pub trait IndexerStoreExt {
    async fn latest_block(&self) -> Result<u64, StoreError>;
    async fn commit(&self, latest_block: u64) -> Result<(), StoreError>;
}

#[cfg_attr(native, async_trait::async_trait)]
#[cfg_attr(wasm, async_trait::async_trait(?Send))]
impl IndexerStoreExt for Store {
    async fn latest_block(&self) -> Result<u64, StoreError> {
        Ok(self.get(LATEST_BLOCK_KEY).await?.map_or(0, |v| {
            u64::from_be_bytes(v.try_into().expect("latest block is 8 bytes"))
        }))
    }

    /// Atomically writes `latest_block` to the store.
    async fn commit(&self, latest_block: u64) -> Result<(), StoreError> {
        let latest_block_bytes = latest_block.to_be_bytes();
        let items = vec![(LATEST_BLOCK_KEY, &latest_block_bytes)];

        self.put_batch(items).await
    }
}
