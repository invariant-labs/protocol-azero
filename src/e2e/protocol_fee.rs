#[cfg(test)]
pub mod e2e_tests {
    use crate::contracts::InvariantError;
    use crate::invariant::Invariant;
    use crate::{
        contracts::{entrypoints::InvariantEntrypoints, FeeTier, PoolKey},
        invariant::InvariantRef,
        math::{
            types::{
                fee_growth::FeeGrowth,
                liquidity::Liquidity,
                percentage::Percentage,
                sqrt_price::{calculate_sqrt_price, SqrtPrice},
                token_amount::TokenAmount,
            },
            MIN_SQRT_PRICE,
        },
    };
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, change_fee_receiver, create_dex,
        create_pool, create_position, create_tokens, get_pool, init_basic_pool,
        init_basic_position, init_basic_swap, init_dex_and_tokens, mint, swap,
        withdraw_protocol_fee,
    };
    use token::PSP22Mintable;
    use token::Token;
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_protocol_fee(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client);
        init_basic_pool!(client, dex, token_x, token_y);
        init_basic_position!(client, dex, token_x, token_y);
        init_basic_swap!(client, dex, token_x, token_y);

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let alice = ink_e2e::alice();
        withdraw_protocol_fee!(client, dex, pool_key, alice).unwrap();

        let amount_x = balance_of!(client, token_x, address_of!(Alice));
        let amount_y = balance_of!(client, token_y, address_of!(Alice));
        assert_eq!(amount_x, 9999999501);
        assert_eq!(amount_y, 9999999000);

        let amount_x = balance_of!(client, token_x, dex.account_id);
        let amount_y = balance_of!(client, token_y, dex.account_id);
        assert_eq!(amount_x, 1499);
        assert_eq!(amount_y, 7);

        let pool_after_withdraw = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(
            pool_after_withdraw.fee_protocol_token_x,
            TokenAmount::new(0)
        );
        assert_eq!(
            pool_after_withdraw.fee_protocol_token_y,
            TokenAmount::new(0)
        );

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_protocol_fee_not_admin(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client);
        init_basic_pool!(client, dex, token_x, token_y);
        init_basic_position!(client, dex, token_x, token_y);
        init_basic_swap!(client, dex, token_x, token_y);

        let pool_key = PoolKey::new(
            token_x.account_id,
            token_y.account_id,
            FeeTier {
                fee: Percentage::from_scale(6, 3),
                tick_spacing: 10,
            },
        )
        .unwrap();
        let bob = ink_e2e::bob();
        let result = withdraw_protocol_fee!(client, dex, pool_key, bob);
        assert_eq!(result, Err(InvariantError::NotFeeReceiver));
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_withdraw_fee_not_deployer(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client);
        init_basic_pool!(client, dex, token_x, token_y);
        init_basic_position!(client, dex, token_x, token_y);
        init_basic_swap!(client, dex, token_x, token_y);

        let admin = ink_e2e::alice();
        let user_address = address_of!(Bob);
        let user = ink_e2e::bob();
        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        change_fee_receiver!(client, dex, pool_key, user_address, admin).unwrap();

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(pool.fee_receiver, user_address);

        withdraw_protocol_fee!(client, dex, pool_key, user).unwrap();

        let amount_x = balance_of!(client, token_x, user_address);
        let amount_y = balance_of!(client, token_y, user_address);
        assert_eq!(amount_x, 1);
        assert_eq!(amount_y, 993);

        let amount_x = balance_of!(client, token_x, dex.account_id);
        let amount_y = balance_of!(client, token_y, dex.account_id);
        assert_eq!(amount_x, 1499);
        assert_eq!(amount_y, 7);

        let pool_after_withdraw = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(
            pool_after_withdraw.fee_protocol_token_x,
            TokenAmount::new(0)
        );
        assert_eq!(
            pool_after_withdraw.fee_protocol_token_y,
            TokenAmount::new(0)
        );

        Ok(())
    }
}
