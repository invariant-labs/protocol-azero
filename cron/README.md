A standalone subproject for scripts that can be executed without compiling the sdk.

## Getting started

### Prerequisites

-Node v20 ([node](https://nodejs.org/en/download/package-manager))

#### Node

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash &&
nvm install 20

```

### Fetch dependencies

```bash
npm i
```

### Fill in your mnemonic

In the *.env* file add your own mnemonic in the following line:
```
DEPLOYER_MNEMONIC=add your mnemonic here this is the right place for twelve words
```

Mnemonic is only required if scripts perform operations other than queries.

### Available scripts

#### random-swap-testnet.ts
Perform random swaps on Invariant testnet using only our faucet tokens.
```bash
npm run random-swap-testnet
```

#### validate-state.ts
Validate the integrity of the protocol's state. Performs only query operations.

```bash
npm run validate-state
```