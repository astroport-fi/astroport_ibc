# Astroport IBC

This repo contains Astroport IBC related contracts.

## Contracts

| Name                           | Description                      |
| ------------------------------ | -------------------------------- |
| [`controller`](contracts/controller) | IBC controller contract intended to be hosted on the main chain |
| [`cw20-ics20`](contracts/cw20-ics20) | IBC Enabled contract that receives CW20 tokens and sends them over IBC channel to a remote chain |
| [`satellite`](contracts/satellite) | IBC enabled astroport satellite contract intended to be hosted on a remote chain |

## Building Contracts

You will need Rust 1.64.0+ with wasm32-unknown-unknown target installed.

You can run unit tests for each contract directory via:

```
cargo test
```

#### For a production-ready (compressed) build:
Run the following from the repository root

```
./scripts/build_release.sh
```

The optimized contracts are generated in the artifacts/ directory.

#### You can compile each contract:
Go to contract directory and run 
    
```
cargo wasm
cp ../../target/wasm32-unknown-unknown/release/astroport_token.wasm .
ls -l astroport_token.wasm
sha256sum astroport_token.wasm
```

## Branches

We use [main](https://github.com/astroport-fi/astroport-ibc/tree/main) branch for new feature development and [release](https://github.com/astroport-fi/astroport-ibc/tree/release) one for collecting features which are ready for deployment. You can find the version and commit for actually deployed contracts [here](https://github.com/astroport-fi/astroport-changelog).

## Docs

Docs can be generated using `cargo doc --no-deps`

