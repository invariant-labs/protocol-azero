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
    use test_helpers::{add_fee_tier, create_dex, fee_tier_exist, get_fee_tiers};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_add_multiple_fee_tiers(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));

        let first_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
        add_fee_tier!(client, InvariantRef, dex, first_fee_tier, admin).unwrap();

        let second_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap();
        add_fee_tier!(client, InvariantRef, dex, second_fee_tier, admin).unwrap();

        let third_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
        add_fee_tier!(client, InvariantRef, dex, third_fee_tier, admin).unwrap();

        let exist = fee_tier_exist!(
            client,
            InvariantRef,
            dex,
            FeeTier::new(Percentage::from_scale(2, 4), 1u16).unwrap()
        );
        assert!(exist);

        let exist = fee_tier_exist!(
            client,
            InvariantRef,
            dex,
            FeeTier::new(Percentage::from_scale(2, 4), 2u16).unwrap()
        );
        assert!(exist);

        let exist = fee_tier_exist!(
            client,
            InvariantRef,
            dex,
            FeeTier::new(Percentage::from_scale(2, 4), 4u16).unwrap()
        );
        assert!(exist);

        let fee_tiers = get_fee_tiers!(client, InvariantRef, dex);
        assert_eq!(fee_tiers.len(), 3);
        assert_eq!(fee_tiers[0], first_fee_tier);
        assert_eq!(fee_tiers[1], second_fee_tier);
        assert_eq!(fee_tiers[2], third_fee_tier);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_add_existing_fee_tier(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
        add_fee_tier!(client, InvariantRef, dex, fee_tier, admin).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
        let result = add_fee_tier!(client, InvariantRef, dex, fee_tier, admin);
        assert_eq!(result, Err(InvariantError::FeeTierAlreadyExist));
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_add_fee_tier_not_admin(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let user = ink_e2e::bob();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
        let result = add_fee_tier!(client, InvariantRef, dex, fee_tier, user);
        assert_eq!(result, Err(InvariantError::NotAdmin));
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_add_fee_tier_zero_fee(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));

        let fee_tier = FeeTier::new(Percentage::new(0), 10).unwrap();
        add_fee_tier!(client, InvariantRef, dex, fee_tier, admin).unwrap();
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_add_fee_tier_tick_spacing_zero(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));

        let fee_tier = FeeTier {
            fee: Percentage::from_scale(2, 4),
            tick_spacing: 0,
        };

        let result = add_fee_tier!(client, InvariantRef, dex, fee_tier, admin);
        assert_eq!(result, Err(InvariantError::InvalidTickSpacing));
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_add_fee_tier_over_upper_bound_tick_spacing(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));

        let fee_tier = FeeTier {
            fee: Percentage::from_scale(2, 4),
            tick_spacing: 101,
        };

        let result = add_fee_tier!(client, InvariantRef, dex, fee_tier, admin);
        assert_eq!(result, Err(InvariantError::InvalidTickSpacing));
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_add_fee_tier_fee_above_limit(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));

        let fee_tier = FeeTier {
            fee: Percentage::from_integer(1),
            tick_spacing: 10,
        };

        let result = add_fee_tier!(client, InvariantRef, dex, fee_tier, admin);
        assert_eq!(result, Err(InvariantError::InvalidFee));
        Ok(())
    }
}
