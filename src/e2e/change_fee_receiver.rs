#[cfg(test)]
pub mod e2e_tests {
    use crate::InvariantError;
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::InvariantRef,
    };
    use decimal::*;
    use ink_e2e::build_message;
    use math::types::percentage::Percentage;
    use math::types::sqrt_price::calculate_sqrt_price;
    use test_helpers::{
        add_fee_tier, address_of, change_fee_receiver, create_dex, create_pool, create_tokens,
        get_pool,
    };
    use token::TokenRef;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_change_fee_reciever(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, TokenRef, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let alice = ink_e2e::alice();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let admin = ink_e2e::alice();
        let alice = address_of!(Alice);
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        change_fee_receiver!(client, InvariantRef, dex, pool_key, alice, admin).unwrap();
        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
        assert_eq!(pool.fee_receiver, alice);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_not_admin_change_fee_reciever(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, TokenRef, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let admin = ink_e2e::alice();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, admin).unwrap();

        let result = create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
            admin
        );
        assert!(result.is_ok());

        let user = ink_e2e::bob();
        let bob = address_of!(Bob);
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let result = change_fee_receiver!(client, InvariantRef, dex, pool_key, bob, user);
        assert_eq!(result, Err(InvariantError::NotAdmin));
        Ok(())
    }
}
