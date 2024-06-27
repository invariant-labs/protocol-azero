#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::Invariant;
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, InvariantError, PoolKey},
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
    use ink::primitives::AccountId;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, claim_fee, create_dex, create_pool,
        create_position, create_tokens, get_pool, get_position, init_basic_pool,
        init_basic_position, init_basic_swap, init_dex_and_tokens, mint, swap,
    };
    use token::PSP22Mintable;
    use token::Token;
    use token::TokenRef;
    use token::PSP22;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_claim(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client);

        init_basic_pool!(client, dex, token_x, token_y);
        init_basic_position!(client, dex, token_x, token_y);
        init_basic_swap!(client, dex, token_x, token_y);

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
        let alice = ink_e2e::alice();
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let user_amount_before_claim = balance_of!(client, token_x, address_of!(Alice));
        let dex_amount_before_claim = balance_of!(client, token_x, dex.account_id);

        claim_fee!(client, dex, 0, alice).unwrap();

        let user_amount_after_claim = balance_of!(client, token_x, address_of!(Alice));
        let dex_amount_after_claim = balance_of!(client, token_x, dex.account_id);
        let position = get_position!(client, dex, 0, alice).unwrap();
        let expected_tokens_claimed = 5;

        assert_eq!(
            user_amount_after_claim - expected_tokens_claimed,
            user_amount_before_claim
        );
        assert_eq!(
            dex_amount_after_claim + expected_tokens_claimed,
            dex_amount_before_claim
        );
        assert_eq!(position.fee_growth_inside_x, pool.fee_growth_global_x);
        assert_eq!(position.tokens_owed_x, TokenAmount(0));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_claim_not_owner(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client);
        init_basic_pool!(client, dex, token_x, token_y);
        init_basic_position!(client, dex, token_x, token_y);
        init_basic_swap!(client, dex, token_x, token_y);

        let user = ink_e2e::bob();
        let result = claim_fee!(client, dex, 0, user);

        assert_eq!(result, Err(InvariantError::PositionNotFound));

        Ok(())
    }
}
