# SP1 Tendermint Template

An example of a Tendermint light client on Ethereum powered by SP1.

## Overview

The SP1 Tendermint template is a simple example of a Tendermint light client on Ethereum powered by SP1. It demonstrates how to use SP1 to generate a proof of a Tendermint update and verify it on Ethereum.

* The `contracts` directory contains a Solidity contract that implements the Tendermint light client.
* The `program` directory contains a Succinct zkVM program that implements Tendermint light client verification logic.
* The `operator` directory contains a Rust program that interacts with the Solidity contract to run in a loop, fetch the latest header and generate a proof of the update, and then updates the contract with the proof.

## End to end deployment

* Follow instructions to install [SP1](https://succinctlabs.github.io/sp1/).
* Install [Forge](https://book.getfoundry.sh/getting-started/installation.html).
* Deploy the SP1 Tendermint contract to a testnet network using the `contracts/script/SP1Tendermint.s.sol` script.

## Generate Fixtures for Forge Testing

To generate fixtures for local testing run:

```shell
$ cd operator
$ TENDERMINT_RPC_URL="https://rpc.celestia-mocha.com/" REAL_PROOFS=true cargo run --release --bin fixture -- --trusted-block 2 --target-block 6
```

To run the example operator with a deployed Tendermint contract run:

CHAIN_ID=<chain_id> RPC_URL=<rpc_url> PRIVATE_KEY=<private_key> cargo run --bin operator

```shell
$ cd operator
$ cargo run --bin operator
```

https://sepolia.etherscan.io/address/0x7C2f8c1CFEC0aFda76fF0f8304602729f280FEd7
