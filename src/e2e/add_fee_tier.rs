#[cfg(test)]
pub mod e2e_tests {
    use crate::math::types::percentage::Percentage;
    use crate::{
        contract::ContractRef,
        contracts::{entrypoints::Invariant, FeeTier},
    };
    use decimal::*;
    use ink_e2e::build_message;
    use test_helpers::{add_fee_tier, create_dex, fee_tier_exist, get_fee_tiers};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn add_multiple_fee_tiers(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, ContractRef, Percentage::new(0));

        let first_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
        add_fee_tier!(client, ContractRef, dex, first_fee_tier, admin).unwrap();

        let second_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap();
        add_fee_tier!(client, ContractRef, dex, second_fee_tier, admin).unwrap();

        let third_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
        add_fee_tier!(client, ContractRef, dex, third_fee_tier, admin).unwrap();

        let exist = fee_tier_exist!(
            client,
            ContractRef,
            dex,
            FeeTier::new(Percentage::from_scale(2, 4), 1u16).unwrap()
        );
        assert!(exist);

        let exist = fee_tier_exist!(
            client,
            ContractRef,
            dex,
            FeeTier::new(Percentage::from_scale(2, 4), 2u16).unwrap()
        );
        assert!(exist);

        let exist = fee_tier_exist!(
            client,
            ContractRef,
            dex,
            FeeTier::new(Percentage::from_scale(2, 4), 4u16).unwrap()
        );
        assert!(exist);

        let fee_tiers = get_fee_tiers!(client, ContractRef, dex);
        assert_eq!(fee_tiers.len(), 3);
        assert_eq!(fee_tiers[0], first_fee_tier);
        assert_eq!(fee_tiers[1], second_fee_tier);
        assert_eq!(fee_tiers[2], third_fee_tier);

        Ok(())
    }

    #[ink_e2e::test]
    #[should_panic]
    async fn add_existing_fee_tier(mut client: ink_e2e::Client<C, E>) -> () {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, ContractRef, Percentage::new(0));

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
        add_fee_tier!(client, ContractRef, dex, fee_tier, admin).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
        add_fee_tier!(client, ContractRef, dex, fee_tier, admin).unwrap();
    }

    #[ink_e2e::test]
    #[should_panic]
    async fn add_fee_tier_not_admin(mut client: ink_e2e::Client<C, E>) -> () {
        let user = ink_e2e::bob();
        let dex = create_dex!(client, ContractRef, Percentage::new(0));

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
        add_fee_tier!(client, ContractRef, dex, fee_tier, user).unwrap();
    }

    #[ink_e2e::test]
    async fn add_fee_tier_zero_fee(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, ContractRef, Percentage::new(0));

        let fee_tier = FeeTier::new(Percentage::new(0), 10).unwrap();
        add_fee_tier!(client, ContractRef, dex, fee_tier, admin).unwrap();
        Ok(())
    }

    #[ink_e2e::test]
    #[should_panic]
    async fn add_fee_tier_tick_spacing_zero(mut client: ink_e2e::Client<C, E>) -> () {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, ContractRef, Percentage::new(0));

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 0).unwrap();
        add_fee_tier!(client, ContractRef, dex, fee_tier, admin).unwrap();
    }
}
