use crate::alloc::string::ToString;
use alloc::string::String;
use math::types::{
    fee_growth::FeeGrowth, liquidity::Liquidity, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Pool {
    pub liquidity: Liquidity,
    pub sqrt_price: SqrtPrice,
    #[tsify(type = "bigint")]
    pub current_tick_index: i32,
    pub fee_growth_global_x: FeeGrowth,
    pub fee_growth_global_y: FeeGrowth,
    pub fee_protocol_token_x: TokenAmount,
    pub fee_protocol_token_y: TokenAmount,
    #[tsify(type = "bigint")]
    pub start_timestamp: u64,
    #[tsify(type = "bigint")]
    pub last_timestamp: u64,
    pub fee_receiver: String,
}

impl Default for Pool {
    fn default() -> Self {
        Self {
            liquidity: Liquidity::default(),
            sqrt_price: SqrtPrice::default(),
            current_tick_index: i32::default(),
            fee_growth_global_x: FeeGrowth::default(),
            fee_growth_global_y: FeeGrowth::default(),
            fee_protocol_token_x: TokenAmount(0u128),
            fee_protocol_token_y: TokenAmount(0u128),
            start_timestamp: u64::default(),
            last_timestamp: u64::default(),
            fee_receiver: String::from("0"),
        }
    }
}
