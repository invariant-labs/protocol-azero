#[cfg(test)]
pub mod e2e_tests {
    use crate::{contract::ContractRef, math::types::percentage::Percentage};
    use decimal::*;
    use ink::primitives::AccountId;
    use token::TokenRef;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn constructor_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let constructor = TokenRef::new(500, None, None, 0);
        let _token: AccountId = client
            .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
            .await
            .expect("Instantiate failed")
            .account_id;

        let constructor = ContractRef::new(Percentage::new(0));

        let _contract: AccountId = client
            .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
            .await
            .expect("Instantiate failed")
            .account_id;
        Ok(())
    }
}
