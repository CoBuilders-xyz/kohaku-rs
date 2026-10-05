use kohaku_tornadocash::{Field, Nullifier, Secret};
use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(Serialize, Deserialize, Tsify)]
#[tsify(type = "`0x${string}`")]
pub struct Hex(String);

#[derive(Debug, thiserror::Error)]
pub enum HexError {
    #[error("Hex must start with 0x")]
    MissingPrefix,

    #[error("Invalid hex: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    #[error("Invalid byte length: {0}")]
    InvalidLength(#[from] std::array::TryFromSliceError),
}

impl Hex {
    pub fn to_bytes(&self) -> Result<Vec<u8>, HexError> {
        let text = self.0.strip_prefix("0x").ok_or(HexError::MissingPrefix)?;

        Ok(hex::decode(text)?)
    }
}

impl TryFrom<Hex> for Nullifier {
    type Error = HexError;

    fn try_from(value: Hex) -> Result<Self, Self::Error> {
        let bytes = value.to_bytes()?;
        Ok(Nullifier::try_from(bytes.as_slice())?)
    }
}

impl TryFrom<Hex> for Secret {
    type Error = HexError;

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

#[cfg(test)]
mod tests {
    use super::{Hex, HexError};

    #[test]
    fn converts_hex_and_bytes_preserving_leading_zeros() {
        let bytes = [0x00, 0x01, 0xff];
        let text = "0x0001ff";

        assert_eq!(Hex(text.to_owned()).to_bytes().unwrap(), bytes);
        assert_eq!(Hex::from(bytes.as_slice()).0, text);
    }

    #[test]
    fn rejects_missing_prefix() {
        let value = Hex("01".repeat(31));

        assert!(matches!(value.to_bytes(), Err(HexError::MissingPrefix)));
    }
}
