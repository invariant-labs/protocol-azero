#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::InvariantRef,
        math::types::liquidity::Liquidity,
        math::types::percentage::Percentage,
        math::types::sqrt_price::{calculate_sqrt_price, SqrtPrice},
    };
    use decimal::*;
    use ink_e2e::build_message;
    use test_helpers::{
        add_fee_tier, approve, create_dex, create_pool, create_position, create_tokens, get_pool,
        get_tickmap,
    };
    use token::TokenRef;
    use token::PSP22;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    fn _to_binary(v: (u16, u64)) {
        println!(
            "Chunk Index = {:?} Value = {:?}, Binary = {:b}",
            v.0, v.1, v.1
        );
    }

    #[ink_e2e::test]
    async fn test_get_tickmap(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

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

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let liquidity_delta = Liquidity::new(1000);

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            0,
            1,
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            1,
            3,
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let tickmap = get_tickmap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            pool.current_tick_index,
            alice
        );

        assert_eq!(tickmap.len(), 2047);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_tickmap_close_to_upper_limit(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let init_tick = 200_000;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

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

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let liquidity_delta = Liquidity::new(1000);

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            -58,
            5,
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let tickmap = get_tickmap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            pool.current_tick_index,
            alice
        );

        assert_eq!(tickmap.len(), 2047);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_tickmap_close_to_bottom_limit(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let init_tick = -200_000;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

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

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let liquidity_delta = Liquidity::new(1000);

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            -58,
            5,
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let tickmap = get_tickmap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            pool.current_tick_index,
            alice
        );

        assert_eq!(tickmap.len(), 2047);
        Ok(())
    }
}
