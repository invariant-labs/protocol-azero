#[cfg(test)]
pub mod e2e_tests {
    use crate::contracts::InvariantError;
    use crate::invariant::Invariant;
    use crate::math::types::percentage::Percentage;
    use crate::{contracts::entrypoints::InvariantEntrypoints, invariant::InvariantRef};
    use decimal::*;
    use ink::primitives::Hash;
    use ink_e2e::ContractsBackend;
    use test_helpers::{create_dex, set_code};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_set_code_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let admin = ink_e2e::alice();

        let dex = create_dex!(client, Percentage::new(0));

        let code_hash = client
            .upload("invariant", &ink_e2e::alice())
            .submit()
            .await
            .expect("upload failed")
            .code_hash;

        let result = set_code!(client, dex, code_hash, admin);
        assert_eq!(result, Ok(()));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_set_code_not_admin(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let attacker = ink_e2e::bob();

        let dex = create_dex!(client, Percentage::new(0));

        let result = set_code!(client, dex, Hash::default(), attacker);
        assert_eq!(result, Err(InvariantError::NotAdmin));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_set_code_code_hash_does_not_exist(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let admin = ink_e2e::alice();

        let dex = create_dex!(client, Percentage::new(0));

        let result = set_code!(client, dex, Hash::default(), admin);
        assert_eq!(result, Err(InvariantError::SetCodeHashError));

        Ok(())
    }
}
