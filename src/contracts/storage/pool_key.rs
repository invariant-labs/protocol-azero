use decimal::Decimal;
use ink::primitives::AccountId;

use crate::contracts::FeeTier;
use crate::InvariantError;
use math::percentage::Percentage;

#[derive(scale::Decode, scale::Encode, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]

pub struct PoolKey {
    pub token_x: AccountId,
    pub token_y: AccountId,
    pub fee_tier: FeeTier,
}

impl Default for PoolKey {
    fn default() -> Self {
        Self {
            token_x: AccountId::from([0; 32]),
            token_y: AccountId::from([1; 32]),
            fee_tier: FeeTier {
                fee: Percentage::new(0),
                tick_spacing: 1,
            },
        }
    }
}

impl PoolKey {
    pub fn new(
        token_0: AccountId,
        token_1: AccountId,
        fee_tier: FeeTier,
    ) -> Result<Self, InvariantError> {
        if token_0 == token_1 {
            return Err(InvariantError::TokensAreSame);
        }

        if token_0 < token_1 {
            Ok(PoolKey {
                token_x: token_0,
                token_y: token_1,
                fee_tier,
            })
        } else {
            Ok(PoolKey {
                token_x: token_1,
                token_y: token_0,
                fee_tier,
            })
        }
    }
}
