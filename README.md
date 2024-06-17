# SP1 Tendermint Template

An example of a Tendermint light client on Ethereum powered by SP1.

## Overview

The SP1 Tendermint template is a simple example of a ZK Tendermint light client on Ethereum powered by SP1. It demonstrates how to use SP1 to generate a proof of the update between two Tendermint headers and verify it on Ethereum.

* The `contracts` directory contains a Solidity contract that implements the Tendermint light client.
* The `program` directory contains a Succinct zkVM program that implements Tendermint light client verification logic.
* The `operator` directory contains a Rust program that interacts with the Solidity contract. It fetches the latest header and generates a proof of the update, and then updates the contract with the proof. It also contains several scripts to help with testing and deployment of the contract.

## Run Tendermint Light Client End to End

* Follow instructions to install [SP1](https://succinctlabs.github.io/sp1/).
* Install [Forge](https://book.getfoundry.sh/getting-started/installation.html).

1. Generate the initialization parameters for the contract.

    ```shell
    cd operator
    TENDERMINT_RPC_URL=https://rpc.celestia-mocha.com/ cargo run --bin genesis --release
    ```

2. Copy the parameters from the output in the previous step into `contracts/.env`:

    ```shell
    TENDERMINT_VKEY_HASH=<tendermint_vkey_hash>
    TRUSTED_HEADER_HASH=<trusted_header_hash>
    TRUSTED_HEIGHT=<trusted_height>
    ```

3. Deploy the `SP1Tendermint` contract with the initialization parameters:

    ```shell
    cd ../contracts

    forge install

    forge script script/SP1Tendermint.s.sol --rpc-url https://ethereum-sepolia.publicnode.com/ --private-key <PRIVATE_KEY> --broadcast
    ```

    If you see the following error, add `--legacy` to the command.
    ```shell
    Error: Failed to get EIP-1559 fees    
    ```

4. Your deployed contract address will be printed to the terminal.

    ```shell
    == Return ==
    0: address <SP1_TENDERMINT_ADDRESS>
    ```

5. Add the configuration to the `.env` file:
    ```shell
    # Example configuration for Sepolia + Celestia Mocha.
    TENDERMINT_RPC_URL=https://rpc.celestia-mocha.com/
    CHAIN_ID=11155111
    RPC_URL=https://ethereum-sepolia.publicnode.com/
    CONTRACT_ADDRESS=<SP1_TENDERMINT_ADDRESS>
    # Key for relaying to the contract.
    PRIVATE_KEY=

    # If you're using the Succinct network, set SP1_PROVER to "network". Otherwise, set it to "local" or "mock".
    SP1_PROVER=
    # Only required if SP1_PROVER is set to "network".
    SP1_PRIVATE_KEY=
    ```

5. Run the Tendermint operator.
    ```shell
    cd ../operator
    RUST_LOG=info cargo run --bin operator --release
    ```

## Contract Tests
### Generate fixtures for forge tests

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