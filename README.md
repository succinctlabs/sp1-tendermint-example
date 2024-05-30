# SP1 Tendermint Template

An example of a Tendermint light client on Ethereum powered by SP1.

## Overview

The SP1 Tendermint template is a simple example of a Tendermint light client on Ethereum powered by SP1. It demonstrates how to use SP1 to generate a proof of a Tendermint update and verify it on Ethereum.

* The `contracts` directory contains a Solidity contract that implements the Tendermint light client.
* The `program` directory contains a Succinct zkVM program that implements Tendermint light client verification logic.
* The `operator` directory contains a Rust program that interacts with the Solidity contract. It fetches the latest header and generates a proof of the update, and then updates the contract with the proof. It also contains several scripts to help with testing and deployment of the contract.

## Test Program

To test that your Tendermint program is working correctly, set `SP1_PROVER=mock` to use the mock prover, and then run the test script in the operator folder. If this executes successfully, you will see a message like `Successfully generated proof!`.

```shell
$ cd operator
$ RUST_LOG=info SP1_PROVER=mock TENDERMINT_RPC_URL="https://rpc.celestia-mocha.com/" cargo run --bin test --release -- --trusted-block 500 --target-block 1000
```

## End to end deployment

* Follow instructions to install [SP1](https://succinctlabs.github.io/sp1/).
* Install [Forge](https://book.getfoundry.sh/getting-started/installation.html).
* Install go and make sure that `go` is in your `PATH` (you can run `go version` to check). Your Go version should be >1.22.1

## Run Tendermint Operator End to End

1. Generate the initialization parameters for the contract.

    ```shell
    cd operator
    cargo run --bin genesis --release
    ```

    This will output `TRUSTED_HEADER_HASH`, `TRUSTED_HEIGHT` and `VKEY_DIGEST`.

2. Copy the initialization parameters from the output into the `contracts/.env` file:

    ```shell
    TRUSTED_HEADER_HASH=<trusted_header_hash>
    TRUSTED_HEIGHT=<trusted_height>
    VKEY_DIGEST=<vkey_digest>
    ```

3. Deploy the `SP1Tendermint` contract with the initialization parameters:

    ```shell
    cd ../contracts

    forge script script/SP1Tendermint.s.sol --rpc-url <RPC_URL> --private-key <PRIVATE_KEY> --etherscan-api-key <ETHERSCAN_API_KEY> --broadcast --verify
    ```

    If you see the following error, add `--legacy` to the command.
    ```shell
    Error: Failed to get EIP-1559 fees    
    ```

4. Add the operator configuration to the `/.env` file:
    ```shell
    CHAIN_ID=
    RPC_URL=
    CONTRACT_ADDRESS=
    PRIVATE_KEY=
    ```

5. Run the Tendermint operator.
    ```shell
    cd operator
    cargo run --bin operator --release
    ```

## Fixtures for Forge Tests

To generate fixtures for local testing run:

```shell
# Generates fixture.json (valid proof)
$ cd operator
$ RUST_LOG=info TENDERMINT_RPC_URL="https://rpc.celestia-mocha.com/" cargo run --bin fixture --release -- --trusted-block 500 --target-block 1000

# Generates mock_fixture.json (mock proof)
$ cd operator
$ RUST_LOG=info SP1_PROVER=mock TENDERMINT_RPC_URL="https://rpc.celestia-mocha.com/" cargo run --bin fixture --release -- --trusted-block 500 --target-block 1000
```

You can check that the generated fixture proofs verify by running the forge tests:
```shell
$ cd contracts
$ forge test -vvv
```