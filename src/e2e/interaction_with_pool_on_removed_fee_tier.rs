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
    use math::{
        types::{
            fee_growth::FeeGrowth,
            liquidity::Liquidity,
            percentage::Percentage,
            sqrt_price::{calculate_sqrt_price, SqrtPrice},
            token_amount::TokenAmount,
        },
        MIN_SQRT_PRICE,
    };
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, change_fee_receiver, claim_fee, create_dex,
        create_pool, create_position, create_tokens, fee_tier_exist, get_all_positions, get_pool,
        get_pools, get_position, init_basic_pool, init_basic_position, init_basic_swap,
        init_dex_and_tokens, mint, positions_equals, remove_fee_tier, remove_position, swap,
        transfer_position, withdraw_protocol_fee,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_interaction_with_pool_on_removed_fee_tier(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client, InvariantRef, TokenRef);
        init_basic_pool!(client, InvariantRef, TokenRef, dex, token_x, token_y);
        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let deployer = ink_e2e::alice();
        // Remove Fee Tier
        {
            remove_fee_tier!(client, InvariantRef, dex, fee_tier, deployer).unwrap();
            let exist = fee_tier_exist!(client, InvariantRef, dex, fee_tier);
            assert!(!exist);
        }
        // Attempt to create same pool again
        {
            let init_tick = 0;
            let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
            let result = create_pool!(
                client,
                InvariantRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_sqrt_price,
                init_tick,
                deployer
            );
            assert_eq!(result, Err(InvariantError::FeeTierNotFound));
        }
        // Init  position
        {
            init_basic_position!(client, InvariantRef, TokenRef, dex, token_x, token_y);
        }
        // Init swap
        {
            init_basic_swap!(client, InvariantRef, TokenRef, dex, token_x, token_y);
        }
        // Claim fee
        {
            let (claimed_x, claimed_y) =
                claim_fee!(client, InvariantRef, dex, 0, deployer).unwrap();
            assert_eq!(claimed_x, TokenAmount(5));
            assert_eq!(claimed_y, TokenAmount(0));
        }
        // Change fee receiver
        {
            change_fee_receiver!(
                client,
                InvariantRef,
                dex,
                pool_key,
                address_of!(Bob),
                deployer
            )
            .unwrap();
        }
        // Withdraw protocol fee
        {
            let fee_receiver = ink_e2e::bob();
            withdraw_protocol_fee!(client, InvariantRef, dex, pool_key, fee_receiver).unwrap();
        }
        // Close position
        {
            remove_position!(client, InvariantRef, dex, 0, deployer).unwrap();
        }
        // Get pool
        {
            get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
        }
        // Get Pools
        {
            let pools = get_pools!(client, InvariantRef, dex);
            assert_eq!(pools.len(), 1);
        }
        // Transfer position
        {
            init_basic_position!(client, InvariantRef, TokenRef, dex, token_x, token_y);
            let transferred_index = 0;
            let position_owner = deployer;
            let recipient = ink_e2e::bob();
            let recipient_address = address_of!(Bob);
            let owner_list_before = get_all_positions!(client, InvariantRef, dex, position_owner);
            let recipient_list_before = get_all_positions!(client, InvariantRef, dex, recipient);
            let removed_position =
                get_position!(client, InvariantRef, dex, transferred_index, position_owner)
                    .unwrap();

            transfer_position!(
                client,
                InvariantRef,
                dex,
                transferred_index,
                recipient_address,
                position_owner
            )
            .unwrap();

            let recipient_position =
                get_position!(client, InvariantRef, dex, transferred_index, recipient).unwrap();
            let owner_list_after = get_all_positions!(client, InvariantRef, dex, position_owner);
            let recipient_list_after = get_all_positions!(client, InvariantRef, dex, recipient);

            assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);
            assert_eq!(owner_list_before.len() - 1, owner_list_after.len());
            assert_eq!(owner_list_after.len(), 0);

            // Equals fields of transferred position
            positions_equals!(recipient_position, removed_position);
        }
        // Readd fee tier and create same pool
        {
            let deployer = ink_e2e::alice();
            add_fee_tier!(client, InvariantRef, dex, fee_tier, deployer).unwrap();
            let init_tick = 0;
            let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
            let result = create_pool!(
                client,
                InvariantRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_sqrt_price,
                init_tick,
                deployer
            );
            assert_eq!(result, Err(InvariantError::PoolAlreadyExist));
        }

        Ok(())
    }
}
