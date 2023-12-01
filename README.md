## Overview

This repository contains an Concentrated Liquidity Automated Market Maker program. The program is designed to provide users with the ability to efficiently manage and optimize liquidity pools with concentrated positions on [Aleph ZERO](https://alephzero.org/).

## Getting Started

### Prerequisites

- Rust & Cargo ([rustup](https://www.rust-lang.org/tools/install))
- cargo-contract ([cargo-contract](https://github.com/paritytech/cargo-contract))
- substrate-contracts-node ([substrate-contracts-node](https://github.com/paritytech/substrate-contracts-node))
- ink! ([ink!](https://use.ink/getting-started/setup))

#### Rust & Cargo

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### cargo-contract

```bash
rustup component add rust-src && cargo install --force --locked cargo-contract
```

#### substrate-contracts-node

```bash
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git
```

### Installation

- Clone repository

```
git clone git@github.com:invariant-labs/protocol-a0.git
```

- Run tests

```
cargo test --features e2e-tests
```

- Build contract

```
cargo contract build
```
