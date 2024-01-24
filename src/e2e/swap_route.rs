#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::{InvariantRef, SwapHop},
    };
    use decimal::*;
    use ink_e2e::build_message;
    use math::types::{
        liquidity::Liquidity, percentage::Percentage, sqrt_price::calculate_sqrt_price,
        token_amount::TokenAmount,
    };
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, claim_fee, create_3_tokens, create_dex,
        create_pool, create_position, get_pool, init_dex_and_3_tokens, mint, quote_route,
        swap_route,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn swap_route(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y, token_z) =
            init_dex_and_3_tokens!(client, InvariantRef, TokenRef);

        let alice = ink_e2e::alice();
        approve!(client, TokenRef, token_x, dex, u64::MAX as u128, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, u64::MAX as u128, alice).unwrap();
        approve!(client, TokenRef, token_z, dex, u64::MAX as u128, alice).unwrap();

        let amount = 1000;
        let bob = ink_e2e::bob();
        mint!(client, TokenRef, token_x, address_of!(Bob), amount, alice).unwrap();
        approve!(client, TokenRef, token_x, dex, amount, bob).unwrap();
        approve!(client, TokenRef, token_y, dex, u64::MAX as u128, bob).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            client,
            InvariantRef,
            dex,
            token_y,
            token_z,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        let pool_key_1 = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let pool_key_2 = PoolKey::new(token_y, token_z, fee_tier).unwrap();

        let liquidity_delta = Liquidity::new(2u128.pow(63) - 1);

        let pool_1 = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
        let slippage_limit_lower = pool_1.sqrt_price;
        let slippage_limit_upper = pool_1.sqrt_price;
        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key_1,
            -1,
            1,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        )
        .unwrap();

        let pool_2 = get_pool!(client, InvariantRef, dex, token_y, token_z, fee_tier).unwrap();
        let slippage_limit_lower = pool_2.sqrt_price;
        let slippage_limit_upper = pool_2.sqrt_price;
        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key_2,
            -1,
            1,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        )
        .unwrap();

        let amount_in = TokenAmount(1000);
        let slippage = Percentage::new(0);
        let swaps = vec![
            SwapHop {
                pool_key: pool_key_1,
                x_to_y: true,
            },
            SwapHop {
                pool_key: pool_key_2,
                x_to_y: true,
            },
        ];

        let expected_token_amount =
            quote_route!(client, InvariantRef, dex, amount_in, swaps.clone()).unwrap();

        swap_route!(
            client,
            InvariantRef,
            dex,
            amount_in,
            expected_token_amount,
            slippage,
            swaps.clone(),
            bob
        )
        .unwrap();

        let bob_amount_x = balance_of!(client, TokenRef, token_x, address_of!(Bob));
        let bob_amount_y = balance_of!(client, TokenRef, token_y, address_of!(Bob));
        let bob_amount_z = balance_of!(client, TokenRef, token_z, address_of!(Bob));

        assert_eq!(bob_amount_x, 0);
        assert_eq!(bob_amount_y, 0);
        assert_eq!(bob_amount_z, 986);

        let pool_1_after =
            get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
        assert_eq!(pool_1_after.fee_protocol_token_x, TokenAmount(1));
        assert_eq!(pool_1_after.fee_protocol_token_y, TokenAmount(0));

        let pool_2_after =
            get_pool!(client, InvariantRef, dex, token_y, token_z, fee_tier).unwrap();
        assert_eq!(pool_2_after.fee_protocol_token_x, TokenAmount(1));
        assert_eq!(pool_2_after.fee_protocol_token_y, TokenAmount(0));

        let alice_amount_x_before = balance_of!(client, TokenRef, token_x, address_of!(Alice));
        let alice_amount_y_before = balance_of!(client, TokenRef, token_y, address_of!(Alice));
        let alice_amount_z_before = balance_of!(client, TokenRef, token_z, address_of!(Alice));

        claim_fee!(client, InvariantRef, dex, 0, alice).unwrap();
        claim_fee!(client, InvariantRef, dex, 1, alice).unwrap();

        let alice_amount_x_after = balance_of!(client, TokenRef, token_x, address_of!(Alice));
        let alice_amount_y_after = balance_of!(client, TokenRef, token_y, address_of!(Alice));
        let alice_amount_z_after = balance_of!(client, TokenRef, token_z, address_of!(Alice));

        assert_eq!(alice_amount_x_after - alice_amount_x_before, 4);
        assert_eq!(alice_amount_y_after - alice_amount_y_before, 4);
        assert_eq!(alice_amount_z_after - alice_amount_z_before, 0);

        Ok(())
    }
}
