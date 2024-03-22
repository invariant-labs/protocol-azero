#[macro_export]
macro_rules! address_of {
    ($account:ident) => {
        ink_e2e::account_id(ink_e2e::AccountKeyring::$account)
    };
}

#[macro_export]
macro_rules! balance_of {
    ($client:ident, $token:ident, $owner:expr) => {{
        let mut call_builder = $token.call_builder::<Token>();
        let call = call_builder.balance_of($owner);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await?
            .return_value()
    }};
}

#[macro_export]
macro_rules! mint {
    ($client:ident, $token:ident, $to:expr, $value:expr, $caller:ident) => {{
        let mut call_builder = $token.call_builder::<Token>();
        let call = call_builder.mint($to, $value);
        let result = $client
            .call(&$caller, &call)
            .dry_run()
            .await?
            .return_value();

        if result.is_ok() {
            $client.call(&$caller, &call).submit().await?.return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! approve {
    ($client:ident, $token:ident, $spender:expr, $value:expr, $caller:ident) => {{
        let mut call_builder = $token.call_builder::<Token>();
        let call = call_builder.increase_allowance($spender, $value);
        let result = $client
            .call(&$caller, &call)
            .dry_run()
            .await?
            .return_value();

        if result.is_ok() {
            $client.call(&$caller, &call).submit().await?.return_value()
        } else {
            result
        }
    }};
}
