#[macro_export]
macro_rules! transfer_v1 {
    ($token: expr, $to: expr, $amount: expr) => {
        let psp22: PSP22Wrapper = $token.into();
        let mut builder = psp22.call().clone();
        builder
            .transfer($to, $amount, vec![])
            .call_v1()
            .invoke()
            .map_err(|_| InvariantError::TransferError)?;
    };
}

#[macro_export]
macro_rules! transfer_from_v1 {
    ($token: expr, $from: expr, $to: expr, $amount: expr) => {
        let psp22: PSP22Wrapper = $token.into();
        let mut builder = psp22.call().clone();
        builder
            .transfer_from($from, $to, $amount, vec![])
            .call_v1()
            .invoke()
            .map_err(|_| InvariantError::TransferError)?;
    };
}

#[macro_export]
macro_rules! balance_of_v1 {
    ($token: expr, $owner: expr) => {{
        let psp22: PSP22Wrapper = $token.into();
        let builder = psp22.call();
        builder.balance_of($owner).call_v1().invoke()
    }};
}

#[macro_export]
macro_rules! withdraw_v1 {
    ($token: expr, $amount: expr) => {
        let wazero: WrappedAZEROWrapper = $token.into();
        let mut builder = wazero.call().clone();
        builder
            .withdraw($amount)
            .call_v1()
            .invoke()
            .map_err(|_| InvariantError::WAZEROWithdrawError)?;
    };
}
