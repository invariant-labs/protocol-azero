#[macro_export]
macro_rules! address_of {
    ($account:ident) => {
        ink_e2e::account_id(ink_e2e::AccountKeyring::$account)
    };
}

#[macro_export]
macro_rules! balance_of {
    ($contract_type:ty, $client:ident, $address:ident, $account:ident) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.balance_of(address_of!($account)));
        $client
            .call_dry_run(&ink_e2e::alice(), &_msg, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! dex_balance {
    ($contract_type:ty, $client:ident, $address:ident, $account:ident) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.balance_of($account));
        $client
            .call_dry_run(&ink_e2e::alice(), &_msg, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! owner_of {
    ($contract_type:ty, $client:ident, $address:ident, $id:expr) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.owner_of($id));
        $client
            .call_dry_run(&ink_e2e::alice(), &_msg, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! balance_of_37 {
    ($contract_type:ty, $client:ident, $address:ident, $account:ident, $token:expr) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.balance_of(address_of!($account), $token));
        $client
            .call_dry_run(&ink_e2e::alice(), &_msg, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! has_role {
    ($contract_type:ty, $client:ident, $address:ident, $role:expr, $account:ident) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.has_role($role, Some(address_of!($account))));
        $client
            .call_dry_run(&ink_e2e::alice(), &_msg, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! grant_role {
    ($contract_type:ty, $client:ident, $address:ident, $role:expr, $account:ident) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.grant_role($role, Some(address_of!($account))));
        $client
            .call(&ink_e2e::alice(), _msg, 0, None)
            .await
            .expect("grant_role failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! revoke_role {
    ($contract_type:ty, $client:ident, $address:ident, $role:expr, $account:ident) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.revoke_role($role, Some(address_of!($account))));
        $client
            .call(&ink_e2e::alice(), _msg, 0, None)
            .await
            .expect("revoke_role failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! mint_dry_run {
    ($contract_type:ty, $client:ident, $address:ident, $account:ident, $id:expr) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.mint(address_of!($account), $id));
        $client
            .call_dry_run(&ink_e2e::alice(), &_msg, 0, None)
            .await
            .return_value()
    }};
    ($contract_type:ty, $client:ident, $address:ident, $signer:ident, $account:ident, $id:expr) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.mint(address_of!($account), $id));
        $client
            .call_dry_run(&ink_e2e::$signer(), &_msg, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! mint {
    ($contract_type:ty, $client:ident, $address:ident, $account:ident, $id:expr) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.mint(address_of!($account), $id));
        $client
            .call(&ink_e2e::alice(), _msg, 0, None)
            .await
            .expect("mint failed")
            .return_value()
    }};
    ($contract_type:ty, $client:ident, $address:ident, $signer:ident, $account:ident, $id:expr) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.mint(address_of!($account), $id));
        $client
            .call(&ink_e2e::$signer(), _msg, 0, None)
            .await
            .expect("mint failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_role_member_count {
    ($contract_type:ty, $client:ident, $address:ident, $role:expr) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.get_role_member_count($role));
        $client
            .call_dry_run(&ink_e2e::alice(), &_msg, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_role_member {
    ($contract_type:ty, $client:ident, $address:ident, $role:expr, $index:expr) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.get_role_member($role, $index));
        $client
            .call_dry_run(&ink_e2e::alice(), &_msg, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_shares {
    ($contract_type:ty, $client:ident, $address:ident, $user:ident) => {{
        let _msg = build_message::<$contract_type>($address.clone())
            .call(|contract| contract.shares(address_of!($user)));
        $client
            .call_dry_run(&ink_e2e::alice(), &_msg, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! method_call {
    ($contract_type:ty, $client:ident, $address:ident, $method:ident) => {{
        let _msg = build_message::<$contract_type>($address.clone()).call(|contract| contract.$method());
        $client
            .call(&ink_e2e::alice(), _msg, 0, None)
            .await
            .expect("method_call failed")
            .return_value()
    }};
    ($contract_type:ty, $client:ident, $address:ident, $signer:ident, $method:ident) => {{
        let _msg = build_message::<$contract_type>($address.clone()).call(|contract| contract.$method());
        $client
            .call(&ink_e2e::$signer(), _msg, 0, None)
            .await
            .expect("method_call failed")
            .return_value()
    }};
    ($contract_type:ty, $client:ident, $address:ident, $method:ident($($args:expr),*)) => {{
        let _msg = build_message::<$contract_type>($address.clone()).call(|contract| contract.$method($($args),*));
        $client
            .call(&ink_e2e::alice(), _msg, 0, None)
            .await
            .expect("method_call failed")
            .return_value()
    }};
    ($contract_type:ty, $client:ident, $address:ident, $signer:ident, $method:ident($($args:expr),*)) => {{
        let _msg = build_message::<$contract_type>($address.clone()).call(|contract| contract.$method($($args),*));
        $client
            .call(&ink_e2e::$signer(), _msg, 0, None)
            .await
            .expect("method_call failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! method_call_dry_run {
    ($contract_type:ty, $client:ident, $address:ident, $method:ident) => {{
        let _msg = build_message::<$contract_type>($address.clone()).call(|contract| contract.$method());
        $client
            .call_dry_run(&ink_e2e::alice(), &_msg, 0, None)
            .await
            .return_value()
    }};
    ($contract_type:ty, $client:ident, $address:ident, $method:ident($($args:expr),*)) => {{
        let _msg = build_message::<$contract_type>($address.clone()).call(|contract| contract.$method($($args),*));
        $client
            .call_dry_run(&ink_e2e::alice(), &_msg, 0, None)
            .await
            .return_value()
    }};
    ($contract_type:ty, $client:ident, $address:ident, $signer:ident, $method:ident) => {{
        let _msg = build_message::<$contract_type>($address.clone()).call(|contract| contract.$method());
        $client
            .call_dry_run(&ink_e2e::$signer(), &_msg, 0, None)
            .await
            .return_value()
    }};
    ($contract_type:ty, $client:ident, $address:ident, $signer:ident, $method:ident($($args:expr),*)) => {{
        let _msg = build_message::<$contract_type>($address.clone()).call(|contract| contract.$method($($args),*));
        $client
            .call_dry_run(&ink_e2e::$signer(), &_msg, 0, None)
            .await
            .return_value()
    }};
}
#[macro_export]
macro_rules! create_tokens {
    ($client:ident, $x:ty, $y:ty, $supply_x:expr, $supply_y:expr) => {{
        // ink_e2e client
        // x:ty  || y:ty => x token ref => TokenRef
        // supply_x:expr || supply_y:expr => amount of initial supply x => 100
        let constructor_x = <$x>::new($supply_x);
        let constructor_y = <$y>::new($supply_y);
        let x = $client
            .instantiate("token", &ink_e2e::alice(), constructor_x, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        let y = $client
            .instantiate("token", &ink_e2e::alice(), constructor_y, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        (x, y)
    }};
}

#[macro_export]
macro_rules! create_dex {
    ($client:ident,  $dex:ty, $dex_fee:expr) => {{
        // ink_e2e client
        // dex:ty => dex ref => ContractRef
        // dex_fee:exp => protocol_fee => Percentage::new(..)
        let constructor_dex = <$dex>::new($dex_fee);
        let dex = $client
            .instantiate("contract", &ink_e2e::alice(), constructor_dex, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        dex
    }};
}

#[macro_export]
macro_rules! create_tokens_and_pair {
    ($client:ident, $x:ty, $y:ty, $supply_x:expr, $supply_y:expr) => {{
        // ink_e2e client
        // x:ty  || y:ty => x token ref => TokenRef
        // supply_x:expr || supply_y:expr => amount of initial supply x => 100
        let constructor_x = <$x>::new($supply_x);
        let constructor_y = <$y>::new($supply_y);
        let x = $client
            .instantiate("token", &ink_e2e::alice(), constructor_x, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        let y = $client
            .instantiate("token", &ink_e2e::alice(), constructor_y, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        (x, y)
    }};
}

#[macro_export]
macro_rules! create_pair {
    ($client:ident, $x:ident ,$y:ident, $dex:ty, $dex_address:expr) => {{
        // client => ink_e2e_client
        // x:ident || y:ident => Addresses of x and y tokens
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        let _msg = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.create_pair($x, $y));
        $client
            .call(&ink_e2e::alice(), _msg, 0, None)
            .await
            .expect("Pair ceration failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! approve {
    ($client:ident, $token:ty ,$token_address:expr, $dex_address:expr, $amount:expr) => {{
        // client => ink_e2e_client
        // token:ty => TokenRef
        // token_address:expr => Addres of token
        // dex_address:expr => Address of contract
        // amount:expr => Amount of tokens that contract will get allowance

        let _msg = build_message::<$token>($token_address.clone())
            .call(|sc| sc.increase_allowance($dex_address.clone(), $amount));
        $client
            .call(&ink_e2e::alice(), _msg, 0, None)
            .await
            .expect("Approval failed")
    }};
}

// increase allowances
