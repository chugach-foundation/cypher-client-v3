<div align="center">
  </br>
  <p>
    <img height="50" src="https://cypher.trade/svgs/logo.svg" />
  </p>
  <p>
    <strong>cypher v3 rust clients</strong>
  </p>
  <p>
    <a href="https://discord.gg/jr9Mu4Uz25">
      <img alt="Discord Chat" src="https://img.shields.io/discord/880917405356945449?color=blue&style=flat-square" />
    </a>
  </p>
  <h4>
    <a href="https://cypher.trade/">cypher.trade</a>
    <span> | </span>
    <a href="https://github.com/chugach-foundation/cypher-client-ts-v3">TypeScript Client</a>
  </h4>
  </br>
</div>

#

## Program Deployments

| Program | Devnet | Mainnet Beta |
| --------|--------|------------- |
| [cypher](/cypher-client)     | `E2hQJAedG6bX2w3rbPQ5XrBnPvC7u3mAorKLvU6XPxwe` | `CYPH3o83JX6jY6NkbproSpdmQ5VWJtxjfJ5P8veyYVu3` |
| [faucet](/faucet-client)     | `2gCkR5aaUiTVRiKDB79EWXm5PAVDWtNTnp9mGuu4ZKdY` |  |

## Notes

* **cypher is in active development so all APIs are subject to change.**

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

## ⚠️⚠️ Compilation issues for M1 chips ⚠️⚠️

In order to prevent issues when compiling due to the `agnostic-orderbook`.

In the root directory of the repo:

`rustup override set 1.59.0-x86_64-apple-darwin`

## Usage

The [`cypher-cli`](https://github.com/chugach-foundation/cypher-cli-v3) repo is a good example of how to use the aforementioned libraries from a consumer perspective, it is a full fledged CLI app to interact with `cypher`

## Directories

This repository contains all necessary crates to interact with the cypher v3 on-chain program in Rust.

- `cypher-client`
  - A barebones client library generated from the IDL of the cypher v3 program
  - Contains some helper methods for cypher accounts to calculate margin ratios, derive PDAs and decode AOB and Serum accounts
- `cypher-utils`
  - Abstractions over `cypher-client` which help with loading multiple Pools, Markets or user accounts from the client side
  - Other utilities to help with efficiently crafting and submitting transactions, subscribing to account updates etc.
- `faucet-client`
  - A barebones client library generated from the IDL of the `faucet` program ran on devnet for every single market listed for lending, borrowing and spot trading on cypher v3