#[cfg(test)]
pub mod e2e_tests {
    use crate::contracts::InvariantError;
    use crate::invariant::Invariant;
    use crate::{
        contracts::{entrypoints::InvariantEntrypoints, FeeTier, PoolKey},
        invariant::InvariantRef,
        math::types::percentage::Percentage,
        math::types::sqrt_price::calculate_sqrt_price,
    };
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, address_of, change_fee_receiver, create_dex, create_pool, create_tokens,
        get_pool,
    };
    use token::TokenRef;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_change_fee_reciever(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let alice = ink_e2e::alice();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let admin = ink_e2e::alice();
        let alice = address_of!(Alice);
        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        change_fee_receiver!(client, dex, pool_key, alice, admin).unwrap();
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(pool.fee_receiver, alice);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_not_admin_change_fee_reciever(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let admin = ink_e2e::alice();

        add_fee_tier!(client, dex, fee_tier, admin).unwrap();

        let result = create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            admin
        );
        assert!(result.is_ok());

        let user = ink_e2e::bob();
        let bob = address_of!(Bob);
        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let result = change_fee_receiver!(client, dex, pool_key, bob, user);
        assert_eq!(result, Err(InvariantError::NotAdmin));
        Ok(())
    }
}
