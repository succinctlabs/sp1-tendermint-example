# SP1 Tendermint Template

An example of a Tendermint light client on Ethereum powered by SP1.

## Overview

The SP1 Tendermint template is a simple example of a Tendermint light client on Ethereum powered by SP1. It demonstrates how to use SP1 to generate a proof of a Tendermint update and verify it on Ethereum.

* The `contracts` directory contains a Solidity contract that implements the Tendermint light client.
* The `program` directory contains a Succinct zkVM program that implements Tendermint light client verification logic.
* The `operator` directory contains a Rust program that interacts with the Solidity contract to run in a loop, fetch the latest header and generate a proof of the update, and then updates the contract with the proof. It also contains several scripts to help with testing and deployment.

## End to end deployment

* Follow instructions to install [SP1](https://succinctlabs.github.io/sp1/).
* Install [Forge](https://book.getfoundry.sh/getting-started/installation.html).
* Install go and make sure that `go` is in your `PATH` (you can run `go version` to check).

## Generate Fixtures for Forge Testing

To generate fixtures for local testing run:

```shell
$ cd operator
$ RUST_LOG=debug TENDERMINT_RPC_URL="https://rpc.celestia-mocha.com/" cargo run --release --bin fixture -- --trusted-block 2 --target-block 6
```

This will take around 10 minutes to complete (as benchmarked on a Macbook Pro M2), as it is generating a full Tendermint proof locally (including recursive aggregation + groth16 verification). In this case, the "core proof" will generate quickly, but the recursive aggregation will take longer because the core proof has several precompiles enabled that cause recursive aggregation to take longer than in the case of a simpler program.

Then, you can run the vkey script to export the Solidity verifier and generate the vkey digest, which is used in the contract:

```shell
$ cd operator
$ RUST_LOG=info cargo run --release --bin vkey
```

You can check that the generated fixture proofs verify by running the forge tests:
```shell
$ cd contracts
$ forge test -vvv
```

## Deploy Contracts

You can use the forge script to deploy the SP1Tendermint contract to a testnet network:

```shell
$ cd contracts
$ forge script script/SP1Tendermint.s.sol --rpc-url $RPC_URL --private-key $PRIVATE_KEY --etherscan-api-key $ETHERSCAN_API_KEY --broadcast --verify
```

## Run Operator

To run the example operator with a deployed Tendermint contract run:

CHAIN_ID=<chain_id> RPC_URL=<rpc_url> PRIVATE_KEY=<private_key> cargo run --bin operator

```shell
$ cd operator
$ cargo run --bin operator
```

