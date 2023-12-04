#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contract::ContractRef,
        contracts::{entrypoints::Invariant, FeeTier},
        math::types::percentage::Percentage,
    };
    use decimal::*;
    use ink_e2e::build_message;
    use test_helpers::{add_fee_tier, create_dex, create_standard_fee_tiers, fee_tier_exist};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn add_fee_tier_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, ContractRef, Percentage::new(0));
        let fee_tier = FeeTier::new(Percentage::new(0), 10u16).unwrap();
        let alice = ink_e2e::alice();
        add_fee_tier!(client, ContractRef, dex, fee_tier, alice);
        let fee_tier = fee_tier_exist!(
            client,
            ContractRef,
            dex,
            FeeTier::new(Percentage::new(0), 10).unwrap()
        );
        assert!(fee_tier);
        Ok(())
    }

    #[ink_e2e::test]
    async fn fee_tier_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, ContractRef, Percentage::new(0));
        let admin = ink_e2e::alice();
        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let result = add_fee_tier!(client, ContractRef, dex, fee_tier, admin);
        assert!(result.is_ok());
        Ok(())
    }

    #[ink_e2e::test]
    #[should_panic]
    async fn invalid_spacing_fee_tier_test(mut client: ink_e2e::Client<C, E>) -> () {
        let dex = create_dex!(client, ContractRef, Percentage::new(0));
        let admin = ink_e2e::alice();
        // 0 tick spacing | should fail
        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 0).unwrap();
        add_fee_tier!(client, ContractRef, dex, fee_tier, admin);
    }

    #[ink_e2e::test]
    #[should_panic]
    async fn non_admin_fee_tier_caller_test(mut client: ink_e2e::Client<C, E>) -> () {
        let dex = create_dex!(client, ContractRef, Percentage::new(0));
        let user = ink_e2e::bob();
        // not-admin
        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 10).unwrap();
        add_fee_tier!(client, ContractRef, dex, fee_tier, user).unwrap();
    }

    #[ink_e2e::test]
    async fn create_standard_fee_tier_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, ContractRef, Percentage::new(0));
        create_standard_fee_tiers!(client, ContractRef, dex);
        let fee_tier = fee_tier_exist!(
            client,
            ContractRef,
            dex,
            FeeTier::new(Percentage::from_scale(5, 2), 100).unwrap()
        );
        assert!(fee_tier);
        Ok(())
    }
}
