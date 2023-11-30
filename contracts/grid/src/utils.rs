
use near_sdk::{Timestamp};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use uint::construct_uint;

// pub const GAS_FOR_RESOLVE_TRANSFER: Gas = 10_000_000_000_000;
//
// pub const GAS_FOR_FT_TRANSFER: Gas = 20_000_000_000_000;


construct_uint! {
    /// 256-bit unsigned integer.
    #[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
    #[serde(crate = "near_sdk::serde")]
    pub struct U256C(4);
}

construct_uint! {
    /// 128-bit unsigned integer.
    #[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
    #[serde(crate = "near_sdk::serde")]
    pub struct U128C(2);
}

pub fn nano_to_sec(nano: Timestamp) -> u32 {
    (nano / 1_000_000_000) as u32
}

pub mod u128_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u128, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
        where
            D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
