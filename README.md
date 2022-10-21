# 🛠 cypher client v3 🛠

This repository contains all necessary crates to interact with the `cypher v3` on-chain program in Rust.

- cypher-client 
  - A barebones client library generated from the IDL of the `cypher v3` program
  - Contains some helper methods for cypher accounts to calculate margin ratios, derive PDAs and decode AOB and Serum accounts
- cypher-utils
  - Abstractions over `cypher-client` which help with loading multiple Pools, Markets or user accounts from the client side
  - Other utilities to help with efficiently crafting and submitting transactions, subscribing to account updates etc.
- faucet-client
  - A barebones client library generated from the IDL of the `faucet` program ran on devnet for every single market listed for lending, borrowing and spot trading on `cypher v3`

## Installation

Adding the crates as dependency can be done like this (soon to be possible from crates.io):

```toml
[package]
name = "your-app"

[dependencies]
cypher-client = { git = "https://github.com/chugach-foundation/cypher-client-v3" }
cypher-utils = { git = "https://github.com/chugach-foundation/cypher-client-v3" }
faucet-client = { git = "https://github.com/chugach-foundation/cypher-client-v3" }
```

By default, all crates enable the `devnet` feature, in order to use them on `mainnet-beta`, that flag should be enabled:

```toml
[package]
name = "your-app"

[dependencies]
cypher-client = { git = "https://github.com/chugach-foundation/cypher-client-v3", features = [ "mainnet-beta" ] }
cypher-utils = { git = "https://github.com/chugach-foundation/cypher-client-v3", features = [ "mainnet-beta" ] }
faucet-client = { git = "https://github.com/chugach-foundation/cypher-client-v3", features = [ "mainnet-beta" ] }
```

### ⚠️⚠️ Compilation issues for M1 chips ⚠️⚠️

In order to prevent issues when compiling due to the `agnostic-orderbook`.

In the root directory of the repo:

`rustup override set 1.59.0-x86_64-apple-darwin`

## Usage

The [`cypher-cli`](https://github.com/chugach-foundation/cypher-cli-v3) repo is a good example of how to use the aforementioned libraries from a consumer perspective, including how to manage positions by placing and canceling orders.

### Keeper

The [`cypher-keeper-v3`](https://github.com/chugach-foundation/cypher-keeper-v3) repository contains the `keeper` functionality for cypher, which can also be used as an example on how to read on-chain data and craft certain transactions that interact with `cypher v3` in order to:

- Cache Oracle prices on Pools, Perpetual Markets and Futures Markets
- Update token indices on Pools to reflect borrow and lending interest rates on accounts interacting with them
- Updating funding rates on Perpetual Markets to reflect available liquidity on the books

### Crank

The [`cypher-crank-v3`](https://github.com/chugach-foundation/cypher-crank-v3) repository contains the `crank`  functionality for cypher, which can also be used as an example on how to read on-chain data and craft certain transactions that interact with `cypher v3` in order to:

- Process events in the Market's Event Queues (current implementation only cranks events for spot markets on devnet)

### Examples

Coming Soon.