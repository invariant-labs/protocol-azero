#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contract::ContractRef,
        contracts::{entrypoints::Invariant, FeeTier},
        math::types::percentage::Percentage,
        InvariantError,
    };
    use decimal::*;
    use ink_e2e::build_message;
    use test_helpers::{add_fee_tier, create_dex, create_pool, create_tokens, get_pool};
    use token::TokenRef;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_create_pool(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, ContractRef, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, TokenRef, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let init_tick = 0;

        let alice = ink_e2e::alice();

        add_fee_tier!(client, ContractRef, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            ContractRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_create_pool_x_to_y_and_y_to_x(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, ContractRef, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, TokenRef, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let init_tick = 0;

        let alice = ink_e2e::alice();

        add_fee_tier!(client, ContractRef, dex, fee_tier, alice).unwrap();

        create_pool!(
            client,
            ContractRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_tick,
            alice
        )
        .unwrap();

        get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

        let result = create_pool!(
            client,
            ContractRef,
            dex,
            token_y,
            token_x,
            fee_tier,
            init_tick,
            alice
        );
        assert_eq!(result, Err(Invariant::PoolAlreadyExists));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_create_pool_with_same_tokens(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, ContractRef, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, TokenRef, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let init_tick = 0;

        let alice = ink_e2e::alice();

        add_fee_tier!(client, ContractRef, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            ContractRef,
            dex,
            token_x,
            token_x,
            fee_tier,
            init_tick,
            alice
        );

        assert_eq!(result, Err(InvariantError::TokensAreSame));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_create_pool_fee_tier_not_added(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, ContractRef, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, TokenRef, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let init_tick = 0;

        let alice = ink_e2e::alice();

        let result = create_pool!(
            client,
            ContractRef,
            dex,
            token_x,
            token_x,
            fee_tier,
            init_tick,
            alice
        );

        assert_eq!(result, Err(InvariantError::FeeTierNotFound));

        Ok(())
    }
}
