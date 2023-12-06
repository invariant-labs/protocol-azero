<div align="center">
    <h1>⚡Invariant protocol⚡</h1>
    <p>
        | <a href="https://docs.invariant.app/docs/aleph_zero">DOCS 📚</a> |
        <a href="https://invariant.app/math-spec-a0.pdf">MATH SPEC 📄</a> |
        <a href="https://discord.gg/VzS3C9wR">DISCORD 🌐</a> |
    </p>
</div>

Invariant protocol is an AMM built on [Aleph Zero](https://alephzero.org), leveraging high capital efficiency and the ability to list markets in a permissionless manner. At the core of the DEX is the Concentrated Liquidity mechanism, designed to handle tokens compatible with the [PSP22 standard](https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md). The protocol is structured around a single contract architecture.

## 🔨 Getting Started

### Prerequisites

- Rust & Cargo ([rustup](https://www.rust-lang.org/tools/install))
- cargo-contract ([cargo-contract](https://github.com/paritytech/cargo-contract))
- substrate-contracts-node ([substrate-contracts-node](https://github.com/paritytech/substrate-contracts-node))

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

### Build protocol

- Clone repository

```bash
git clone git@github.com:invariant-labs/protocol-a0.git
```

- Build contract

```bash
cargo contract build
```

- Run tests

```bash
cargo test --features e2e-tests
```
