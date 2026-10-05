use kohaku_tornadocash::{
    Field,
    note::{Nullifier, Secret},
};
use serde::{Serialize, Deserialize};
use tsify::Tsify;

#[derive(Serialize, Deserialize, Tsify)]
#[tsify(type = "`0x${string}`")]
pub struct Hex(String);

impl Hex {
    pub fn to_bytes(&self) -> Result<Vec<u8>, wasm_bindgen::JsError> {
        let text = self.0
            .strip_prefix("0x")
            .ok_or_else(|| wasm_bindgen::JsError::new("Hex must start with 0x"))?;

        Ok(hex::decode(text)?)
    }
}

impl TryFrom<Hex> for Nullifier {
    type Error = wasm_bindgen::JsError;

    fn try_from(value: Hex) -> Result<Self, Self::Error> {
        let bytes = value.to_bytes()?;
        Ok(Nullifier::try_from(bytes.as_slice())?)
    }
}

impl TryFrom<Hex> for Secret {
    type Error = wasm_bindgen::JsError;

    fn try_from(value: Hex) -> Result<Self, Self::Error> {
        let bytes = value.to_bytes()?;
        Ok(Secret::try_from(bytes.as_slice())?)
    }
}

impl From<Nullifier> for Hex {
    fn from(value: Nullifier) -> Self {
        Self(format!("0x{}", hex::encode(value.as_bytes())))
    }
}

impl From<Secret> for Hex {
    fn from(value: Secret) -> Self {
        Self(format!("0x{}", hex::encode(value.as_bytes())))
    }
}

impl From<&[u8]> for Hex {
    fn from(value: &[u8]) -> Self {
        Self(format!("0x{}", hex::encode(value)))
    }
}

impl From<Field> for Hex {
    fn from(value: Field) -> Self {
        Self(value.to_string())
    }
}