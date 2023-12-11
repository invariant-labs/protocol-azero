#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contract::ContractRef,
        contracts::{entrypoints::Invariant, FeeTier, PoolKey},
        math::{
            types::{
                fee_growth::FeeGrowth, liquidity::Liquidity, percentage::Percentage,
                sqrt_price::SqrtPrice, token_amount::TokenAmount,
            },
            MIN_SQRT_PRICE,
        },
    };
    use decimal::*;
    use ink_e2e::build_message;
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, claim_fee, create_dex, create_pool,
        create_position, create_tokens, get_pool, get_position, init_basic_pool,
        init_basic_position, init_basic_swap, init_dex_and_tokens, mint, swap, transfer_position,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_claim(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client, ContractRef, TokenRef);
        init_basic_pool!(client, ContractRef, TokenRef, dex, token_x, token_y);
        init_basic_position!(client, ContractRef, TokenRef, dex, token_x, token_y);
        init_basic_swap!(client, ContractRef, TokenRef, dex, token_x, token_y);

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
        let alice = ink_e2e::alice();
        let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
        let user_amount_before_claim = balance_of!(client, TokenRef, token_x, address_of!(Alice));
        let dex_amount_before_claim = balance_of!(client, TokenRef, token_x, dex);

        claim_fee!(client, ContractRef, dex, 0, alice).unwrap();

        let user_amount_after_claim = balance_of!(client, TokenRef, token_x, address_of!(Alice));
        let dex_amount_after_claim = balance_of!(client, TokenRef, token_x, dex);
        let position = get_position!(client, ContractRef, dex, 0, alice).unwrap();
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
    async fn test_claim_not_deployer(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client, ContractRef, TokenRef);
        init_basic_pool!(client, ContractRef, TokenRef, dex, token_x, token_y);
        init_basic_position!(client, ContractRef, TokenRef, dex, token_x, token_y);
        init_basic_swap!(client, ContractRef, TokenRef, dex, token_x, token_y);

        let user = ink_e2e::bob();
        let user_address = address_of!(Bob);
        let admin = ink_e2e::alice();

        transfer_position!(client, ContractRef, dex, 0, user_address, admin).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
        let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
        let user_amount_before_claim = balance_of!(client, TokenRef, token_x, user_address);
        let dex_amount_before_claim = balance_of!(client, TokenRef, token_x, dex);

        claim_fee!(client, ContractRef, dex, 0, user).unwrap();

        let user_amount_after_claim = balance_of!(client, TokenRef, token_x, user_address);
        let dex_amount_after_claim = balance_of!(client, TokenRef, token_x, dex);
        let position = get_position!(client, ContractRef, dex, 0, user).unwrap();
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
}
