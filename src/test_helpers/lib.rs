use decimal::*;
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
        if x > y {
            (y, x)
        } else {
            (x, y)
        }
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
macro_rules! approve {
    ($client:ident, $token:ty ,$token_address:expr, $dex_address:expr, $amount:expr, $caller:ident) => {{
        // client => ink_e2e_client
        // token:ty => TokenRef
        // token_address:expr => Addres of token
        // dex_address:expr => Address of contract
        // amount:expr => Amount of tokens that contract will get allowance

        let _msg = build_message::<$token>($token_address.clone())
            .call(|sc| sc.increase_allowance($dex_address.clone(), $amount));
        $client
            .call(&$caller, _msg, 0, None)
            .await
            .expect("Approval failed")
    }};
}

#[macro_export]
macro_rules! create_standard_fee_tiers {
    ($client:ident, $dex:ty, $dex_address:expr) => {{
        // client => ink_e2e_client
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // 1 * 10^(-4) = 0.0001 = 0.01%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(1, 4),
                tick_spacing: 1
            }
        );
        // 5 * 10^(-4) = 0.0005 = 0.05%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(5, 4),
                tick_spacing: 5
            }
        );
        // 1  * 10^(-3) = 0.001 = 0.1%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(1, 3),
                tick_spacing: 10
            }
        );
        // 3 * 10(-3) = 0.003 = 0.3%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(3, 3),
                tick_spacing: 30
            }
        );
        // 1 * 10^(-2) = 0.01 = 1%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(1, 2),
                tick_spacing: 100
            }
        );
        // 5 * 10^(-2) = 0.05 = 5%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(5, 2),
                tick_spacing: 100
            }
        );
        // 1 * 10^(-1) = 0.1 = 10%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(1, 1),
                tick_spacing: 100
            }
        );
        // 5 * 10^(-1) = 0.5 = 50%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(5, 1),
                tick_spacing: 100
            }
        );
    }};
}

#[macro_export]
macro_rules! create_fee_tier {
    ($client:ident, $dex:ty, $dex_address:expr, $fee_tier:expr) => {{
        // client => ink_e2e_client
        // x:ident || y:ident => Addresses of x and y tokens
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // fee:expr => Percentage
        // spacing:expr => tick_spacing as u16
        let _msg = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.add_fee_tier($fee_tier));
        $client
            .call(&ink_e2e::alice(), _msg, 0, None)
            .await
            .expect("Fee Tier creation failed")
            .return_value()
    }};
}
#[macro_export]
macro_rules! get_fee_tier {
    ($client:ident, $dex:ty, $dex_address:expr, $fee:expr, $spacing:expr) => {{
        // client => ink_e2e_client
        // x:ident || y:ident => Addresses of x and y tokens
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // fee:expr => Percentage
        // spacing:expr => tick_spacing as u16
        let key = FeeTierKey($fee, $spacing);
        let _msg =
            build_message::<$dex>($dex_address.clone()).call(|contract| contract.get_fee_tier(key));
        $client
            .call(&ink_e2e::alice(), _msg, 0, None)
            .await
            .expect("Fee Tier creation failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! create_pool {
    ($client:ident, $dex:ty, $dex_address:expr, $x:ident, $y:ident, $fee_tier:expr, $init_tick:expr) => {{
        // client => ink_e2e_client
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // x:ident || y:ident => Addresses of x and y tokens
        // fee_tier:expr => Pool fee tier
        // init_tick:expr => init tick as i32
        let _msg = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.add_pool($x, $y, $fee_tier, $init_tick));
        $client
            .call(&ink_e2e::alice(), _msg, 0, None)
            .await
            .expect("Pool creation failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_pool {
    ($client:ident, $dex:ty, $dex_address:expr, $x:ident, $y:ident, $fee_tier:expr) => {{
        // client => ink_e2e_client
        // x:ident || y:ident => Addresses of x and y tokens
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // fee_tier:expr => Pool fee tier
        let _msg = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_pool($x, $y, $fee_tier));
        $client
            .call(&ink_e2e::alice(), _msg, 0, None)
            .await
            .expect("Pool creation failed")
            .return_value()
    }};
}

///

#[macro_export]
macro_rules! create_position {
    ($client:ident, $dex:ty, $dex_address:expr, $pool_key:expr, $lower_tick:expr, $upper_tick:expr, $l:expr, $limit_lower:expr, $limit_upper:expr,$caller:ident) => {{
        // client => ink_e2e_client
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // pool_key:expr => Pool key
        // lower_tick:ident || upper_tick:ident => index of lower tick
        // l => liquidity
        // limit_lower | limit_upper => price limit slippage
        // caller => ink_e2e account to sign call

        let _msg = build_message::<$dex>($dex_address.clone()).call(|contract| {
            contract.create_position(
                $pool_key,
                $lower_tick,
                $upper_tick,
                $l,
                $limit_lower,
                $limit_upper,
            )
        });
        $client
            .call(&$caller, _msg, 0, None)
            .await
            .expect("Create position failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! remove_position {
    ($client:ident, $dex:ty, $dex_address:expr, $index:expr, $caller:ident) => {{
        // client => ink_e2e_client
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // index:expr => position index to remove
        // caller => ink_e2e account to sign call
        let _msg = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.remove_position($index));
        $client.call(&$caller, _msg, 0, None).await
    }};
}

#[macro_export]
macro_rules! get_position {
    ($client:ident, $dex:ty, $dex_address:expr, $index:expr, $caller:ident) => {{
        // client => ink_e2e_client
        // x:ident || y:ident => Addresses of x and y tokens
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // index:expr => position index to remove
        // caller => ink_e2e account to sign call
        let _msg = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_position($index));
        $client
            .call(&$caller, _msg, 0, None)
            .await
            .expect("Position recieving failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_all_positions {
    ($client:ident, $dex:ty, $dex_address:expr, $caller:ident) => {{
        // client => ink_e2e_client
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // caller:expr => ink_e2e::account to sign the call
        let _msg = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_all_positions());
        $client
            .call(&$caller, _msg, 0, None)
            .await
            .expect("getting posisitons failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! claim_fee {
    ($client:ident, $dex:ty, $dex_address:expr, $index:expr, $pool_key:expr, $caller:ident) => {{
        // client => ink_e2e_client
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // index:expr => u32
        // pool_key:expr => pool_key
        // $caller => signer from ink_e2e env
        let _msg = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.position_claim_fee($index, $pool_key));
        $client
            .call(&$caller, _msg, 0, None)
            .await
            .expect("Pool creation failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! swap {
    ($client:ident, $dex:ty, $dex_address:expr, $pool_key:expr, $x_to_y:expr, $amount:expr, $by_amount_in:expr, $sqrt_price_limit:expr, $caller:ident) => {{
        // client => ink_e2e_client
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // pool_key:expr => pool_key
        // x_to_y:expr => bool
        // amount:expr => TokenAmount to swap
        // by_amount_in:expr => bool
        // sqrt_price_limit:expr => price limit
        // caller => signer from ink_e2e env
        let _msg = build_message::<$dex>($dex_address.clone()).call(|contract| {
            contract.swap(
                $pool_key,
                $x_to_y,
                $amount,
                $by_amount_in,
                $sqrt_price_limit,
            )
        });
        $client
            .call(&$caller, _msg, 0, None)
            .await
            .expect("Pool creation failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_tick {
    ($client:ident, $dex:ty, $dex_address:expr, $index:expr, $pool_key:expr, $caller:ident) => {{
        // client => ink_e2e_client
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // index:expr => tick index
        // pool_key:expr => pool_key
        // caller => ink_e2e account to sign call
        let _msg = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_tick($pool_key, $index));
        $client
            .call(&$caller, _msg, 0, None)
            .await
            .expect("Tick recieving failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! tickmap_bit {
    ($client:ident, $dex:ty, $dex_address:expr, $index:expr, $pool_key:expr, $caller:ident) => {{
        // client => ink_e2e_client
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // index:expr => tick index
        // pool_key:expr => pool_key
        // caller => ink_e2e account to sign call
        let _msg = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_tickmap_bit($pool_key, $index));
        $client
            .call(&$caller, _msg, 0, None)
            .await
            .expect("Tickmap byte failed")
            .return_value()
    }};
}
