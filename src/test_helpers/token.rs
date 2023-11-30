#[macro_export]
macro_rules! address_of {
    ($account:ident) => {
        ink_e2e::account_id(ink_e2e::AccountKeyring::$account)
    };
}

#[macro_export]
macro_rules! balance_of {
    ($client:ident, $token:ty, $token_address:expr, $owner:expr) => {{
        let message = build_message::<$token>($token_address.clone())
            .call(|contract| contract.balance_of($owner));
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! mint {
    ($client:ident, $token:ty, $token_address:expr, $to:expr, $value:expr) => {{
        let message = build_message::<$token>($token_address.clone())
            .call(|contract| contract.mint($to, $value));
        let result = $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$token>($token_address.clone())
                .call(|contract| contract.mint($to, $value));
            $client
                .call(&ink_e2e::alice(), message, 0, None)
                .await
                .expect("mint failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! approve {
    ($client:ident, $token:ty, $token_address:expr, $spender:expr, $value:expr, $caller:ident) => {{
        let message = build_message::<$token>($token_address.clone())
            .call(|contract| contract.increase_allowance($spender, $value));
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$token>($token_address.clone())
                .call(|contract| contract.increase_allowance($spender, $value));
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("approve failed")
                .return_value()
        } else {
            result
        }
    }};
}
