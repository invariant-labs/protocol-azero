#[cfg(test)]
pub mod e2e_tests {
    use crate::contracts::{PocType, U256T};
    use crate::{invariant::InvariantRef, math::types::percentage::Percentage};
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use token::TokenRef;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_constructor(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let mut constructor = TokenRef::new(500, None, None, 0);

        let _token = client
            .instantiate("token", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");

        let poc_field = PocType(U256T::from(0));
        let mut constructor = InvariantRef::new(Percentage::new(0), poc_field);
        let _contract = client
            .instantiate("invariant", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");

        Ok(())
    }
}
