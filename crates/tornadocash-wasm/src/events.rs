use alloy_primitives::{Address, B256};
use alloy_rpc_types_eth::Log;
use js_sys::BigInt;
use kohaku_tornadocash::syncer::SyncEvent;
use serde::{Deserialize, Serialize};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::{
    bigint::{optional_block_number, u256_to_bigint},
    hex::Hex,
};

/// Caller-supplied log data, without emitter or reorg metadata.
#[derive(Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LogInput {
    topics: Vec<Hex>,
    data: Hex,
    // Preserve the JS type so the shared helper can reject numbers and null.
    #[serde(default, with = "tsify::serde_wasm_bindgen::preserve")]
    #[tsify(optional, type = "bigint")]
    block_number: JsValue,
}

/// A plain JavaScript snapshot of [`SyncEvent`].
#[derive(Serialize, Tsify)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[tsify(large_number_types_as_bigints)]
pub enum DecodedEvent {
    Deposit {
        commitment: Hex,
        leaf_index: u32,
        block_number: u64,
    },
    Withdrawal {
        to: Hex,
        nullifier_hash: Hex,
        relayer: Hex,
        #[serde(with = "tsify::serde_wasm_bindgen::preserve")]
        #[tsify(type = "bigint")]
        fee: BigInt,
        block_number: u64,
    },
}

/// Decode a supplied log using [`SyncEvent::try_from_log`].
///
/// Returns a plain snapshot or undefined for any core no-result, including an
/// unknown/undecodable event or absent block number. The caller selects the
/// correct pool/chain and handles removals and reorgs.
///
/// # Errors
///
/// Throws a JavaScript Error for malformed input: topics must be 32-byte Hex
/// strings with at most four entries, data must be Hex bytes, and a present
/// blockNumber must be an unsigned u64 bigint.
#[wasm_bindgen(js_name = decodeEvent)]
pub fn decode_event(log: &Ts<LogInput>) -> Result<Option<Ts<DecodedEvent>>, JsError> {
    let log = log.to_rust()?.into_log()?;
    match SyncEvent::try_from_log(&log) {
        Some(event) => Ok(Some(DecodedEvent::from(event).into_ts()?)),
        None => Ok(None),
    }
}

impl From<SyncEvent> for DecodedEvent {
    fn from(event: SyncEvent) -> Self {
        match event {
            SyncEvent::Deposit(event) => Self::Deposit {
                commitment: event.commitment.into(),
                leaf_index: event.leaf_index,
                block_number: event.block_number,
            },
            SyncEvent::Withdrawal(event) => Self::Withdrawal {
                to: event.to.into(),
                nullifier_hash: event.nullifier_hash.into(),
                relayer: event.relayer.into(),
                fee: u256_to_bigint(event.fee),
                block_number: event.block_number,
            },
        }
    }
}

impl LogInput {
    fn into_log(self) -> Result<Log, JsError> {
        let topics = self
            .topics
            .into_iter()
            .map(|topic| {
                let bytes = topic.to_bytes()?;
                let bytes: [u8; 32] = bytes.as_slice().try_into()?;
                Ok(B256::from(bytes))
            })
            .collect::<Result<Vec<_>, JsError>>()?;
        let data = self.data.to_bytes()?.into();
        // The input does not identify an emitter. Core decoding uses only topics/data.
        let inner = alloy_primitives::Log::new(Address::ZERO, topics, data)
            .ok_or_else(|| JsError::new("Log topics must contain at most 4 entries"))?;

        Ok(Log {
            inner,
            block_number: optional_block_number(&self.block_number)?,
            ..Log::default()
        })
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use js_sys::{Array, Object, Reflect};
    use tsify::{Ts, Tsify};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::{LogInput, decode_event};
    use crate::hex::Hex;

    // Encoded offline with `cast abi-encode-event` using the event declarations in
    // crates/tornadocash/src/abis/tornado.rs. Deposit: commitment=0xab, leafIndex=7,
    // timestamp=123. Withdrawal: to=0xab, nullifierHash=0xcd, relayer=Ethereum DAI,
    // fee=2^200+17. These are synthetic fixtures, not evidence of on-chain activity.
    const DEPOSIT_TOPIC: &str = "a945e51eec50ab98c161376f0db4cf2aeba3ec92755fe2fcd388bdbbb80ff196";
    const DEPOSIT_DATA: &str = concat!(
        "0x",
        "0000000000000000000000000000000000000000000000000000000000000007",
        "000000000000000000000000000000000000000000000000000000000000007b",
    );
    const WITHDRAWAL_TOPIC: &str =
        "e9e508bad6d4c3227e881ca19068f099da81b5164dd6d62b2eaf1e8bc6c34931";
    const WITHDRAWAL_DATA: &str = concat!(
        "0x",
        "00000000000000000000000000000000000000000000000000000000000000ab",
        "00000000000000000000000000000000000000000000000000000000000000cd",
        "0000000000000100000000000000000000000000000000000000000000000011",
    );

    fn deposit(block_number: &JsValue) -> Ts<LogInput> {
        input(
            &[
                &hex::decode(DEPOSIT_TOPIC).unwrap(),
                &hex::decode(format!("{}ab", "00".repeat(31))).unwrap(),
            ],
            &JsValue::from_str(DEPOSIT_DATA),
            block_number,
        )
    }

    fn property(value: &JsValue, name: &str) -> JsValue {
        Reflect::get(value, &JsValue::from_str(name)).unwrap()
    }

    fn input(topics: &[&[u8]], data: &JsValue, block_number: &JsValue) -> Ts<LogInput> {
        let value = Object::new();
        let js_topics = Array::new();
        for topic in topics {
            js_topics.push(&Hex::from(*topic).into_ts().unwrap().into());
        }
        Reflect::set(&value, &"topics".into(), &js_topics).unwrap();
        Reflect::set(&value, &"data".into(), data).unwrap();
        if !block_number.is_undefined() {
            Reflect::set(&value, &"blockNumber".into(), block_number).unwrap();
        }
        Ts::new_unchecked(value.into())
    }

    #[wasm_bindgen_test]
    fn adapts_topics_arbitrary_data_and_exact_block_number() {
        let topics: [&[u8]; 4] = [&[0x00; 32], &[0x01; 32], &[0x02; 32], &[0xff; 32]];
        let log = input(
            &topics,
            &JsValue::from_str("0x0001ff"),
            &JsValue::from(u64::MAX),
        )
        .to_rust()
        .unwrap()
        .into_log()
        .unwrap();

        for (actual, expected) in log.topics().iter().zip(topics) {
            assert_eq!(actual.as_slice(), expected);
        }
        assert_eq!(log.data().data.as_ref(), &[0x00, 0x01, 0xff]);
        assert_eq!(log.block_number, Some(u64::MAX));
    }

    #[wasm_bindgen_test]
    fn preserves_absent_block_number_and_empty_log_data() {
        let log = input(&[], &JsValue::from_str("0x"), &JsValue::UNDEFINED)
            .to_rust()
            .unwrap()
            .into_log()
            .unwrap();

        assert_eq!(log.block_number, None);
        assert!(log.topics().is_empty());
        assert!(log.data().data.is_empty());
    }

    #[wasm_bindgen_test]
    fn rejects_topic_with_incorrect_byte_length() {
        assert!(
            input(
                &[&[0xab; 31]],
                &JsValue::from_str("0x"),
                &JsValue::UNDEFINED
            )
            .to_rust()
            .unwrap()
            .into_log()
            .is_err()
        );
    }

    #[wasm_bindgen_test]
    fn rejects_too_many_topics() {
        let topics: [&[u8]; 5] = [&[0xab; 32]; 5];
        assert!(
            input(&topics, &JsValue::from_str("0x"), &JsValue::UNDEFINED)
                .to_rust()
                .unwrap()
                .into_log()
                .is_err()
        );
    }

    #[wasm_bindgen_test]
    fn rejects_invalid_hex_data() {
        assert!(
            input(&[], &JsValue::from_str("0xzz"), &JsValue::UNDEFINED)
                .to_rust()
                .unwrap()
                .into_log()
                .is_err()
        );
    }

    #[wasm_bindgen_test]
    fn rejects_number_block_without_coercion() {
        assert!(
            input(&[], &JsValue::from_str("0x"), &JsValue::from_f64(1.0))
                .to_rust()
                .unwrap()
                .into_log()
                .is_err()
        );
    }

    #[wasm_bindgen_test]
    fn decodes_deposit_to_plain_snapshot_with_exact_block_number() {
        let block_number = 9_007_199_254_740_993_u64;
        let Some(event) = decode_event(&deposit(&JsValue::from(block_number))).unwrap() else {
            panic!("expected deposit snapshot");
        };
        let value = event.js_value();

        assert_eq!(property(&value, "kind"), JsValue::from_str("deposit"));
        assert_eq!(
            property(&value, "commitment"),
            JsValue::from_str(&format!("0x{}ab", "00".repeat(31)))
        );
        assert_eq!(property(&value, "leafIndex"), JsValue::from(7));
        assert_eq!(property(&value, "blockNumber"), JsValue::from(block_number));
    }

    #[wasm_bindgen_test]
    fn decodes_withdrawal_with_checksum_addresses_and_full_width_fee() {
        let log = input(
            &[
                &hex::decode(WITHDRAWAL_TOPIC).unwrap(),
                &hex::decode("0000000000000000000000006b175474e89094c44da98b954eedeac495271d0f")
                    .unwrap(),
            ],
            &JsValue::from_str(WITHDRAWAL_DATA),
            &JsValue::from(u64::MAX),
        );
        let Some(event) = decode_event(&log).unwrap() else {
            panic!("expected withdrawal snapshot");
        };
        let value = event.js_value();

        assert_eq!(property(&value, "kind"), JsValue::from_str("withdrawal"));
        assert_eq!(
            property(&value, "to"),
            JsValue::from_str("0x00000000000000000000000000000000000000AB")
        );
        assert_eq!(
            property(&value, "nullifierHash"),
            JsValue::from_str(&format!("0x{}cd", "00".repeat(31)))
        );
        assert_eq!(
            property(&value, "relayer"),
            JsValue::from_str("0x6B175474E89094C44Da98b954EedeAC495271d0F")
        );
        assert_eq!(
            property(&value, "fee"),
            JsValue::bigint_from_str(
                "1606938044258990275541962092341162602522202993782792835301393"
            )
        );
        assert_eq!(property(&value, "blockNumber"), JsValue::from(u64::MAX));
    }

    #[wasm_bindgen_test]
    fn preserves_core_no_result_as_undefined() {
        let signature = hex::decode(DEPOSIT_TOPIC).unwrap();
        for log in [
            deposit(&JsValue::UNDEFINED),
            input(
                &[&[0x11; 32]],
                &JsValue::from_str("0x"),
                &JsValue::from(1_u64),
            ),
            input(
                &[&signature, &[0; 32]],
                &JsValue::from_str("0x"),
                &JsValue::from(1_u64),
            ),
            input(
                &[&signature, &[0xff; 32]],
                &JsValue::from_str(DEPOSIT_DATA),
                &JsValue::from(1_u64),
            ),
        ] {
            let result: JsValue = decode_event(&log).unwrap().into();
            assert!(result.is_undefined());
        }
    }

    #[wasm_bindgen_test]
    fn public_decoder_returns_js_errors_for_structural_input() {
        for log in [
            Ts::new_unchecked(JsValue::NULL),
            input(&[&[0; 31]], &JsValue::from_str("0x"), &JsValue::UNDEFINED),
            deposit(&JsValue::from_f64(1.0)),
        ] {
            let Err(error) = decode_event(&log) else {
                panic!("expected structural error");
            };
            let error: JsValue = error.into();
            assert!(error.is_instance_of::<js_sys::Error>());
        }
    }
}
