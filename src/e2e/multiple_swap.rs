#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contracts::{entrypoints::InvariantTrait, get_liquidity, FeeTier, PoolKey},
        invariant::InvariantRef,
    };
    use decimal::*;
    use ink_e2e::build_message;
    use math::{
        types::{
            fee_growth::FeeGrowth,
            percentage::Percentage,
            sqrt_price::{calculate_sqrt_price, SqrtPrice},
            token_amount::TokenAmount,
        },
        MAX_SQRT_PRICE, MIN_SQRT_PRICE,
    };
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, create_dex, create_pool, create_position,
        create_tokens, get_pool, init_dex_and_tokens, mint, multiple_swap, quote, swap,
        swap_exact_limit,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_multiple_swap_x_to_y(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        multiple_swap!(client, InvariantRef, TokenRef, true);
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_multiple_swap_y_to_x(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        multiple_swap!(client, InvariantRef, TokenRef, false);
        Ok(())
    }
}
