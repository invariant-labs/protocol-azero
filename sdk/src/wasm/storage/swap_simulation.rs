use serde::{Deserialize, Serialize};

use crate::{sqrt_price::SqrtPrice, token_amount::TokenAmount, LiquidityTick};

#[derive(Serialize, Deserialize, Debug, tsify::Tsify)]
#[tsify(from_wasm_abi, into_wasm_abi)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Default)]
pub struct CalculateSwapResult {
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub fee: TokenAmount,
    pub start_sqrt_price: SqrtPrice,
    pub target_sqrt_price: SqrtPrice,
    pub crossed_ticks: Vec<LiquidityTick>,
    pub global_insufficient_liquidity: bool,
    pub state_outdated: bool,
    pub max_ticks_crossed: bool,
}
