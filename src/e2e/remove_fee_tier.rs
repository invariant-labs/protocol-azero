#[cfg(test)]
pub mod e2e_tests {
    use crate::InvariantError;
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier},
        invariant::InvariantRef,
    };
    use decimal::*;
    use ink_e2e::build_message;
    use math::types::percentage::Percentage;
    use test_helpers::{add_fee_tier, create_dex, fee_tier_exist, remove_fee_tier};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_remove_fee_tier(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
        add_fee_tier!(client, InvariantRef, dex, fee_tier, admin).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap();
        add_fee_tier!(client, InvariantRef, dex, fee_tier, admin).unwrap();

        remove_fee_tier!(client, InvariantRef, dex, fee_tier, admin).unwrap();
        let exist = fee_tier_exist!(
            client,
            InvariantRef,
            dex,
            FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap()
        );
        assert!(!exist);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_remove_not_existing_fee_tier(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
        add_fee_tier!(client, InvariantRef, dex, fee_tier, admin).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap();
        let result = remove_fee_tier!(client, InvariantRef, dex, fee_tier, admin);
        assert_eq!(result, Err(InvariantError::FeeTierNotFound));
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_remove_fee_tier_not_admin(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let user = ink_e2e::bob();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
        add_fee_tier!(client, InvariantRef, dex, fee_tier, admin).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap();
        add_fee_tier!(client, InvariantRef, dex, fee_tier, admin).unwrap();

        let result = remove_fee_tier!(client, InvariantRef, dex, fee_tier, user);
        assert_eq!(result, Err(InvariantError::NotAdmin));
        Ok(())
    }
}
