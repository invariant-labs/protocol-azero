#[cfg(test)]
pub mod e2e_tests {
    use crate::InvariantError;
    use crate::{contracts::entrypoints::InvariantTrait, invariant::InvariantRef};
    use decimal::*;
    use ink_e2e::build_message;
    use math::types::percentage::Percentage;
    use test_helpers::{change_protocol_fee, create_dex, get_protocol_fee};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_change_protocol_fee(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let contract = create_dex!(client, InvariantRef, Percentage::new(0));
        let alice = ink_e2e::alice();

        let protocol_fee = get_protocol_fee!(client, InvariantRef, contract);
        assert_eq!(protocol_fee, Percentage::new(0));

        change_protocol_fee!(client, InvariantRef, contract, Percentage::new(1), alice).unwrap();
        let protocol_fee = get_protocol_fee!(client, InvariantRef, contract);
        assert_eq!(protocol_fee, Percentage::new(1));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_change_protocol_fee_not_admin(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let contract = create_dex!(client, InvariantRef, Percentage::new(0));
        let user = ink_e2e::bob();

        let result = change_protocol_fee!(client, InvariantRef, contract, Percentage::new(1), user);
        assert_eq!(result, Err(InvariantError::NotAdmin));
        Ok(())
    }
}
