#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::InvariantRef,
        InvariantError,
    };
    use decimal::*;
    use ink::primitives::AccountId;
    use ink_e2e::build_message;
    use math::types::{
        fee_growth::FeeGrowth,
        liquidity::Liquidity,
        percentage::Percentage,
        sqrt_price::{calculate_sqrt_price, SqrtPrice},
    };
    use test_helpers::{
        add_fee_tier, address_of, approve, create_dex, create_pool, create_position, create_tokens,
        get_all_positions, get_pool, get_position, get_tick, is_tick_initialized, positions_equals,
        remove_position, transfer_position,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_remove_position_from_empty_list(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, InvariantRef, Percentage::from_scale(6, 3));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 3).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let init_tick = -23028;
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

        let result = remove_position!(client, InvariantRef, dex, 0, alice);
        assert_eq!(result, Err(InvariantError::PositionNotFound));
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_add_multiple_positions(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let init_tick = -23028;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_balance = 10u128.pow(10);

        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_balance, initial_balance);

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 3).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

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

        approve!(client, TokenRef, token_x, dex, initial_balance, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_balance, alice).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let pool_state = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        // Open three positions
        {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[0],
                tick_indexes[2],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[1],
                tick_indexes[4],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }

        // Remove middle position
        {
            let position_index_to_remove = 2;
            let positions_list_before = get_all_positions!(client, InvariantRef, dex, alice);
            let last_position = positions_list_before[positions_list_before.len() - 1];

            remove_position!(client, InvariantRef, dex, position_index_to_remove, alice).unwrap();

            let positions_list_after = get_all_positions!(client, InvariantRef, dex, alice);
            let tested_position = positions_list_after[position_index_to_remove as usize];

            // Last position should be at removed index
            assert_eq!(last_position.pool_key, tested_position.pool_key);
            assert_eq!(last_position.liquidity, tested_position.liquidity);
            assert_eq!(
                last_position.lower_tick_index,
                tested_position.lower_tick_index
            );
            assert_eq!(
                last_position.upper_tick_index,
                tested_position.upper_tick_index
            );
            assert_eq!(
                last_position.fee_growth_inside_x,
                tested_position.fee_growth_inside_x
            );
            assert_eq!(
                last_position.fee_growth_inside_y,
                tested_position.fee_growth_inside_y
            );
            assert_eq!(last_position.tokens_owed_x, tested_position.tokens_owed_x);
            assert_eq!(last_position.tokens_owed_y, tested_position.tokens_owed_y);
        }
        // Add position in place of the removed one
        {
            let positions_list_before = get_all_positions!(client, InvariantRef, dex, alice);

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[1],
                tick_indexes[2],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            let positions_list_after = get_all_positions!(client, InvariantRef, dex, alice);
            assert_eq!(positions_list_before.len() + 1, positions_list_after.len());
        }
        // Remove last position
        {
            let last_position_index_before =
                get_all_positions!(client, InvariantRef, dex, alice).len() - 1;

            remove_position!(
                client,
                InvariantRef,
                dex,
                last_position_index_before as u32,
                alice
            )
            .unwrap();

            let last_position_index_after =
                get_all_positions!(client, InvariantRef, dex, alice).len() - 1;

            assert_eq!(last_position_index_before - 1, last_position_index_after)
        }
        // Remove all positions
        {
            let last_position_index = get_all_positions!(client, InvariantRef, dex, alice).len();

            for i in (0..last_position_index).rev() {
                remove_position!(client, InvariantRef, dex, i as u32, alice).unwrap();
            }

            let list_length = get_all_positions!(client, InvariantRef, dex, alice).len();
            assert_eq!(list_length, 0);
        }
        // Add position to cleared list
        {
            let list_length_before = get_all_positions!(client, InvariantRef, dex, alice).len();

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
            let list_length_after = get_all_positions!(client, InvariantRef, dex, alice).len();
            assert_eq!(list_length_after, list_length_before + 1);
        }
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_only_owner_can_modify_position_list(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let init_tick = -23028;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_balance = 10u128.pow(10);

        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_balance, initial_balance);

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 3).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

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

        approve!(client, TokenRef, token_x, dex, initial_balance, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_balance, alice).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let pool_state = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        // Open three positions
        {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[0],
                tick_indexes[2],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[1],
                tick_indexes[4],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }

        // Remove middle position
        {
            let position_index_to_remove = 2;
            let positions_list_before = get_all_positions!(client, InvariantRef, dex, alice);
            let last_position = positions_list_before[positions_list_before.len() - 1];

            remove_position!(client, InvariantRef, dex, position_index_to_remove, alice).unwrap();

            let positions_list_after = get_all_positions!(client, InvariantRef, dex, alice);
            let tested_position = positions_list_after[position_index_to_remove as usize];

            // Last position should be at removed index
            assert_eq!(last_position.pool_key, tested_position.pool_key);
            assert_eq!(last_position.liquidity, tested_position.liquidity);
            assert_eq!(
                last_position.lower_tick_index,
                tested_position.lower_tick_index
            );
            assert_eq!(
                last_position.upper_tick_index,
                tested_position.upper_tick_index
            );
            assert_eq!(
                last_position.fee_growth_inside_x,
                tested_position.fee_growth_inside_x
            );
            assert_eq!(
                last_position.fee_growth_inside_y,
                tested_position.fee_growth_inside_y
            );
            assert_eq!(last_position.tokens_owed_x, tested_position.tokens_owed_x);
            assert_eq!(last_position.tokens_owed_y, tested_position.tokens_owed_y);
        }
        // Add position in place of the removed one
        {
            let positions_list_before = get_all_positions!(client, InvariantRef, dex, alice);

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[1],
                tick_indexes[2],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            let positions_list_after = get_all_positions!(client, InvariantRef, dex, alice);
            assert_eq!(positions_list_before.len() + 1, positions_list_after.len());
        }
        // Remove last position
        {
            let last_position_index_before =
                get_all_positions!(client, InvariantRef, dex, alice).len() - 1;

            let unauthorized_user = ink_e2e::bob();
            let result = remove_position!(
                client,
                InvariantRef,
                dex,
                last_position_index_before as u32,
                unauthorized_user
            );
            assert_eq!(result, Err(InvariantError::PositionNotFound));
        }
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_transfer_position_ownership(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let init_tick = -23028;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_balance = 10u128.pow(10);

        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_balance, initial_balance);

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 3).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

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

        approve!(client, TokenRef, token_x, dex, initial_balance, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_balance, alice).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let pool_state = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
        {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
            let list_length = get_all_positions!(client, InvariantRef, dex, alice).len();

            assert_eq!(list_length, 1)
        }

        let bob_address = address_of!(Bob);
        let bob = ink_e2e::bob();
        // Open  additional positions
        {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[1],
                tick_indexes[2],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[1],
                tick_indexes[3],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }
        // Transfer first position
        {
            let transferred_index = 0;
            let owner_list_before = get_all_positions!(client, InvariantRef, dex, alice);
            let recipient_list_before = get_all_positions!(client, InvariantRef, dex, bob);
            let removed_position =
                get_position!(client, InvariantRef, dex, transferred_index, alice).unwrap();
            let last_position_before = owner_list_before[owner_list_before.len() - 1];

            transfer_position!(
                client,
                InvariantRef,
                dex,
                transferred_index,
                bob_address,
                alice
            )
            .unwrap();

            let recipient_position =
                get_position!(client, InvariantRef, dex, transferred_index, bob).unwrap();
            let owner_list_after = get_all_positions!(client, InvariantRef, dex, alice);
            let recipient_list_after = get_all_positions!(client, InvariantRef, dex, bob);
            let owner_first_position_after =
                get_position!(client, InvariantRef, dex, transferred_index, alice).unwrap();

            assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);
            assert_eq!(owner_list_before.len() - 1, owner_list_after.len());

            // move last position
            positions_equals!(owner_first_position_after, last_position_before);

            // Equals fields od transferred position
            positions_equals!(recipient_position, removed_position);
        }

        // Transfer middle position
        {
            let transferred_index = 1;
            let owner_list_before = get_all_positions!(client, InvariantRef, dex, alice);
            let recipient_list_before = get_all_positions!(client, InvariantRef, dex, bob);
            let last_position_before = owner_list_before[owner_list_before.len() - 1];

            transfer_position!(
                client,
                InvariantRef,
                dex,
                transferred_index,
                bob_address,
                alice
            )
            .unwrap();

            let owner_list_after = get_all_positions!(client, InvariantRef, dex, alice);
            let recipient_list_after = get_all_positions!(client, InvariantRef, dex, bob);
            let owner_first_position_after =
                get_position!(client, InvariantRef, dex, transferred_index, alice).unwrap();

            assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);
            assert_eq!(owner_list_before.len() - 1, owner_list_after.len());

            // move last position
            positions_equals!(owner_first_position_after, last_position_before);
        }
        // Transfer last position
        {
            let owner_list_before = get_all_positions!(client, InvariantRef, dex, alice);
            let transferred_index = (owner_list_before.len() - 1) as u32;
            let removed_position =
                get_position!(client, InvariantRef, dex, transferred_index, alice).unwrap();

            transfer_position!(
                client,
                InvariantRef,
                dex,
                transferred_index,
                bob_address,
                alice
            )
            .unwrap();

            let recipient_list_after = get_all_positions!(client, InvariantRef, dex, bob);
            let recipient_position_index = (recipient_list_after.len() - 1) as u32;
            let recipient_position =
                get_position!(client, InvariantRef, dex, recipient_position_index, bob).unwrap();

            positions_equals!(removed_position, recipient_position);
        }

        // Clear position
        {
            let transferred_index = 0;
            let recipient_list_before = get_all_positions!(client, InvariantRef, dex, bob);
            let removed_position =
                get_position!(client, InvariantRef, dex, transferred_index, alice).unwrap();

            transfer_position!(
                client,
                InvariantRef,
                dex,
                transferred_index,
                bob_address,
                alice
            )
            .unwrap();

            let recipient_list_after = get_all_positions!(client, InvariantRef, dex, bob);
            let recipient_position_index = (recipient_list_after.len() - 1) as u32;
            let recipient_position =
                get_position!(client, InvariantRef, dex, recipient_position_index, bob).unwrap();
            let owner_list_after = get_all_positions!(client, InvariantRef, dex, alice);

            assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);
            assert_eq!(0, owner_list_after.len());

            // Equals fields od transferred position
            positions_equals!(recipient_position, removed_position);
        }

        // Get back position
        {
            let transferred_index = 0;
            let owner_list_before = get_all_positions!(client, InvariantRef, dex, alice);
            let recipient_list_before = get_all_positions!(client, InvariantRef, dex, bob);
            let removed_position =
                get_position!(client, InvariantRef, dex, transferred_index, bob).unwrap();
            let last_position_before = recipient_list_before[recipient_list_before.len() - 1];

            transfer_position!(
                client,
                InvariantRef,
                dex,
                transferred_index,
                address_of!(Alice),
                bob
            )
            .unwrap();

            let owner_list_after = get_all_positions!(client, InvariantRef, dex, alice);
            let recipient_list_after = get_all_positions!(client, InvariantRef, dex, bob);
            let recipient_first_position_after =
                get_position!(client, InvariantRef, dex, transferred_index, bob).unwrap();

            let owner_new_position =
                get_position!(client, InvariantRef, dex, transferred_index, alice).unwrap();

            assert_eq!(recipient_list_after.len(), recipient_list_before.len() - 1);
            assert_eq!(owner_list_before.len() + 1, owner_list_after.len());

            // move last position
            positions_equals!(last_position_before, recipient_first_position_after);

            // Equals fields od transferred position
            positions_equals!(owner_new_position, removed_position);
        }

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_only_owner_can_transfer_position(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let init_tick = -23028;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_balance = 10u128.pow(10);

        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_balance, initial_balance);

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 3).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

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

        approve!(client, TokenRef, token_x, dex, initial_balance, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_balance, alice).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let pool_state = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
        {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
            let list_length = get_all_positions!(client, InvariantRef, dex, alice).len();

            assert_eq!(list_length, 1)
        }

        let bob = ink_e2e::bob();
        // Open  additional positions
        {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[1],
                tick_indexes[2],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                tick_indexes[1],
                tick_indexes[3],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }
        // Transfer first position
        {
            let transferred_index = 0;

            let result = transfer_position!(
                client,
                InvariantRef,
                dex,
                transferred_index,
                address_of!(Alice),
                bob
            );
            assert_eq!(result, Err(InvariantError::PositionNotFound));
        }
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_multiple_positions_on_same_tick(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_balance = 100_000_000;

        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_balance, initial_balance);

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 10).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

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

        approve!(client, TokenRef, token_x, dex, initial_balance, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_balance, alice).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        // Three position on same lower and upper tick
        {
            let lower_tick_index = -10;
            let upper_tick_index = 10;

            let liquidity_delta = Liquidity::new(100);

            let pool_state =
                get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            let first_position = get_position!(client, InvariantRef, dex, 0, alice).unwrap();

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            let second_position = get_position!(client, InvariantRef, dex, 1, alice).unwrap();

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            let third_position = get_position!(client, InvariantRef, dex, 2, alice).unwrap();

            assert!(first_position.lower_tick_index == second_position.lower_tick_index);
            assert!(first_position.upper_tick_index == second_position.upper_tick_index);
            assert!(first_position.lower_tick_index == third_position.lower_tick_index);
            assert!(first_position.upper_tick_index == third_position.upper_tick_index);

            // Load states
            let pool_state =
                get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
            let lower_tick =
                get_tick!(client, InvariantRef, dex, pool_key, lower_tick_index).unwrap();
            let upper_tick =
                get_tick!(client, InvariantRef, dex, pool_key, upper_tick_index).unwrap();
            let expected_liquidity = Liquidity::new(liquidity_delta.get() * 3);
            let zero_fee = FeeGrowth::new(0);

            // Check ticks
            assert!(lower_tick.index == lower_tick_index);
            assert!(upper_tick.index == upper_tick_index);
            assert_eq!(lower_tick.liquidity_gross, expected_liquidity);
            assert_eq!(upper_tick.liquidity_gross, expected_liquidity);
            assert_eq!(lower_tick.liquidity_change, expected_liquidity);
            assert_eq!(upper_tick.liquidity_change, expected_liquidity);
            assert!(lower_tick.sign);
            assert!(!upper_tick.sign);

            // Check pool
            assert_eq!(pool_state.liquidity, expected_liquidity);
            assert!(pool_state.current_tick_index == init_tick);

            // Check first position
            assert!(first_position.pool_key == pool_key);
            assert!(first_position.liquidity == liquidity_delta);
            assert!(first_position.lower_tick_index == lower_tick_index);
            assert!(first_position.upper_tick_index == upper_tick_index);
            assert!(first_position.fee_growth_inside_x == zero_fee);
            assert!(first_position.fee_growth_inside_y == zero_fee);

            // Check second position
            assert!(second_position.pool_key == pool_key);
            assert!(second_position.liquidity == liquidity_delta);
            assert!(second_position.lower_tick_index == lower_tick_index);
            assert!(second_position.upper_tick_index == upper_tick_index);
            assert!(second_position.fee_growth_inside_x == zero_fee);
            assert!(second_position.fee_growth_inside_y == zero_fee);

            // Check third position
            assert!(third_position.pool_key == pool_key);
            assert!(third_position.liquidity == liquidity_delta);
            assert!(third_position.lower_tick_index == lower_tick_index);
            assert!(third_position.upper_tick_index == upper_tick_index);
            assert!(third_position.fee_growth_inside_x == zero_fee);
            assert!(third_position.fee_growth_inside_y == zero_fee);
        }
        {
            let lower_tick_index = -10;
            let upper_tick_index = 10;
            let zero_fee = FeeGrowth::new(0);

            let liquidity_delta = Liquidity::new(100);

            let pool_state =
                get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            let first_position = get_position!(client, InvariantRef, dex, 3, alice).unwrap();

            // Check first position
            assert!(first_position.pool_key == pool_key);
            assert!(first_position.liquidity == liquidity_delta);
            assert!(first_position.lower_tick_index == lower_tick_index);
            assert!(first_position.upper_tick_index == upper_tick_index);
            assert!(first_position.fee_growth_inside_x == zero_fee);
            assert!(first_position.fee_growth_inside_y == zero_fee);

            let lower_tick_index = -20;
            let upper_tick_index = -10;

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            let second_position = get_position!(client, InvariantRef, dex, 4, alice).unwrap();

            // Check second position
            assert!(second_position.pool_key == pool_key);
            assert!(second_position.liquidity == liquidity_delta);
            assert!(second_position.lower_tick_index == lower_tick_index);
            assert!(second_position.upper_tick_index == upper_tick_index);
            assert!(second_position.fee_growth_inside_x == zero_fee);
            assert!(second_position.fee_growth_inside_y == zero_fee);

            let lower_tick_index = 10;
            let upper_tick_index = 20;
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            let third_position = get_position!(client, InvariantRef, dex, 5, alice).unwrap();

            // Check third position
            assert!(third_position.pool_key == pool_key);
            assert!(third_position.liquidity == liquidity_delta);
            assert!(third_position.lower_tick_index == lower_tick_index);
            assert!(third_position.upper_tick_index == upper_tick_index);
            assert!(third_position.fee_growth_inside_x == zero_fee);
            assert!(third_position.fee_growth_inside_y == zero_fee);

            // Load states
            let pool_state =
                get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
            let tick_n20 = get_tick!(client, InvariantRef, dex, pool_key, -20).unwrap();
            let tick_n10 = get_tick!(client, InvariantRef, dex, pool_key, -10).unwrap();
            let tick_10 = get_tick!(client, InvariantRef, dex, pool_key, 10).unwrap();
            let tick_20 = get_tick!(client, InvariantRef, dex, pool_key, 20).unwrap();
            let tick_n20_bit = is_tick_initialized!(client, InvariantRef, dex, pool_key, -20);
            let tick_n10_bit = is_tick_initialized!(client, InvariantRef, dex, pool_key, -10);
            let tick_20_bit = is_tick_initialized!(client, InvariantRef, dex, pool_key, 20);

            let expected_active_liquidity = Liquidity::new(400);

            // Check tick -20
            assert_eq!(tick_n20.index, -20);
            assert_eq!(tick_n20.liquidity_gross, Liquidity::new(100));
            assert_eq!(tick_n20.liquidity_change, Liquidity::new(100));
            assert!(tick_n20.sign);
            assert!(tick_n20_bit);

            // Check tick -10
            assert_eq!(tick_n10.index, -10);
            assert_eq!(tick_n10.liquidity_gross, Liquidity::new(500));
            assert_eq!(tick_n10.liquidity_change, Liquidity::new(300));
            assert!(tick_n10.sign);
            assert!(tick_n10_bit);

            // Check tick 10
            assert_eq!(tick_10.index, 10);
            assert_eq!(tick_10.liquidity_gross, Liquidity::new(500));
            assert_eq!(tick_10.liquidity_change, Liquidity::new(300));
            assert!(!tick_10.sign);
            assert!(tick_20_bit);

            // Check tick 20
            assert_eq!(tick_20.index, 20);
            assert_eq!(tick_20.liquidity_gross, Liquidity::new(100));
            assert_eq!(tick_20.liquidity_change, Liquidity::new(100));
            assert!(!tick_20.sign);
            assert!(tick_20_bit);

            // Check pool
            assert_eq!(pool_state.liquidity, expected_active_liquidity);
            assert!(pool_state.current_tick_index == init_tick);
        }
        Ok(())
    }
}
