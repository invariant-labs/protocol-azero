#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::InvariantRef;
    use decimal::*;
    use ink::primitives::AccountId;
    use math::types::percentage::Percentage;
    use token::TokenRef;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_constructor(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let constructor = TokenRef::new(500, None, None, 0);
        let _token: AccountId = client
            .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
            .await
            .expect("Instantiate failed")
            .account_id;

        let constructor = InvariantRef::new(Percentage::new(0));

        let _contract: AccountId = client
            .instantiate("invariant", &ink_e2e::alice(), constructor, 0, None)
            .await
            .expect("Instantiate failed")
            .account_id;
        Ok(())
    }
}
