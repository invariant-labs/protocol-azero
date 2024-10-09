#[cfg(test)]
pub mod e2e_tests {
    use crate::contracts::{entrypoints::InvariantEntrypoints, InvariantError};
    use crate::invariant::{Invariant, InvariantRef};
    use crate::math::types::percentage::Percentage;
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use test_helpers::{address_of, change_admin, create_dex, get_admin};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_change_admin(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let admin = ink_e2e::alice();
        let dex = create_dex!(client, Percentage::new(0));

        let new_admin = address_of!(Bob);
        change_admin!(client, dex, new_admin, admin).unwrap();

        let current_admin = get_admin!(client, dex);
        assert_eq!(current_admin, new_admin);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_change_admin_not_admin(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));

        let not_admin = ink_e2e::bob();
        let new_admin = address_of!(Bob);
        let result = change_admin!(client, dex, new_admin, not_admin);
        assert_eq!(result, Err(InvariantError::NotAdmin));

        let old_admin = address_of!(Alice);
        let current_admin = get_admin!(client, dex);
        assert_eq!(current_admin, old_admin);

        Ok(())
    }
}
