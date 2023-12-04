#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contract::ContractRef, contracts::entrypoints::Invariant,
        math::types::percentage::Percentage,
    };
    use decimal::*;
    use ink_e2e::build_message;
    use test_helpers::create_dex;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn change_protocol_fee(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let contract = create_dex!(client, ContractRef, Percentage::new(0));

        let protocol_fee = {
            let _msg = build_message::<ContractRef>(contract.clone())
                .call(|contract| contract.get_protocol_fee());
            client
                .call(&ink_e2e::alice(), _msg, 0, None)
                .await
                .expect("getting protocol fee failed")
        }
        .return_value();

        assert_eq!(protocol_fee, Percentage::new(0));

        let _result = {
            let _msg = build_message::<ContractRef>(contract.clone())
                .call(|contract| contract.change_protocol_fee(Percentage::new(1)));
            client
                .call(&ink_e2e::alice(), _msg, 0, None)
                .await
                .expect("changing protocol fee failed")
        };

        let protocol_fee = {
            let _msg = build_message::<ContractRef>(contract.clone())
                .call(|contract| contract.get_protocol_fee());
            client
                .call(&ink_e2e::alice(), _msg, 0, None)
                .await
                .expect("getting protocol fee failed")
        }
        .return_value();

        assert_eq!(protocol_fee, Percentage::new(1));

        Ok(())
    }

    #[ink_e2e::test]
    #[should_panic]
    async fn change_protocol_fee_should_panic(mut client: ink_e2e::Client<C, E>) -> () {
        let contract = create_dex!(client, ContractRef, Percentage::new(0));

        let _msg = build_message::<ContractRef>(contract.clone())
            .call(|contract| contract.change_protocol_fee(Percentage::new(1)));
        client
            .call(&ink_e2e::bob(), _msg, 0, None)
            .await
            .expect("changing protocol fee failed");
    }
}
