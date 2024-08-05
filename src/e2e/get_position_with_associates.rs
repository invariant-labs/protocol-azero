#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, InvariantError, PoolKey},
        invariant::{Invariant, InvariantRef},
        math::types::{
            liquidity::Liquidity, percentage::Percentage, sqrt_price::calculate_sqrt_price,
        },
    };
    use decimal::*;
    use ink::primitives::AccountId;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, approve, create_dex, create_pool, create_position, create_tokens, get_pool,
        get_position, get_position_with_associates, get_tick, init_basic_pool, init_basic_position,
        init_dex_and_tokens,
    };
    use token::Token;
    use token::TokenRef;
    use token::PSP22;
    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_get_position_with_associates(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client);
        let alice = ink_e2e::alice();

        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 10,
        };

        init_basic_pool!(client, dex, token_x, token_y);
        init_basic_position!(client, dex, token_x, token_y);

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let (lower_tick_index, upper_tick_index) = (-20, 10);

        let position_regular = get_position!(client, dex, 0, alice).unwrap();

        let pool_regular = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick_regular = get_tick!(client, dex, pool_key, lower_tick_index).unwrap();
        let upper_tick_regular = get_tick!(client, dex, pool_key, upper_tick_index).unwrap();

        let (position, pool, lower_tick, upper_tick) =
            get_position_with_associates!(client, dex, 0, alice).unwrap();

        assert_eq!(position_regular, position);
        assert_eq!(pool_regular, pool);
        assert_eq!(lower_tick_regular, lower_tick);
        assert_eq!(upper_tick_regular, upper_tick);
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_position_does_not_exist(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client);
        let alice = ink_e2e::alice();

        init_basic_pool!(client, dex, token_x, token_y);
        let result = get_position_with_associates!(client, dex, 0, alice);

        assert_eq!(result, Err(InvariantError::PositionNotFound));
        Ok(())
    }
}
