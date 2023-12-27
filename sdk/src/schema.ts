const CONTRACT_NAME = 'invariant'

export enum InvariantQuery {
    ProtocolFee = `${CONTRACT_NAME}::getProtocolFee`,
}

export enum InvariantTx {
    ChangeProtocolFee = `${CONTRACT_NAME}::changeProtocolFee`,
}