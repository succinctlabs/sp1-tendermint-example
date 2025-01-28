# SP1 Tendermint Template

An example of a Tendermint light client on Ethereum powered by SP1.

> [!CAUTION]
>
> This repository is still an active work-in-progress and is not audited or meant for production usage.


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

    This will show the data for the genesis block as well as SP1 Tendermint program verification key
    which you will need to initialize the SP1 Tendermint contract.

2. Deploy the `SP1Tendermint` contract with the initialization parameters:

    ```shell
    cd ../contracts

    forge install

    TENDERMINT_VKEY_HASH=<TENDERMINT_VKEY_HASH> TRUSTED_HEADER_HASH=<TRUSTED_HEADER_HASH> TRUSTED_HEIGHT=<TRUSTED_HEIGHT> forge script script/SP1Tendermint.s.sol --rpc-url https://ethereum-sepolia.publicnode.com/ --private-key <PRIVATE_KEY> --broadcast
    ```

    If you see the following error, add `--legacy` to the command.
    ```shell
    Error: Failed to get EIP-1559 fees    
    ```

3. Your deployed contract address will be printed to the terminal.

    ```shell
    == Return ==
    0: address <SP1_TENDERMINT_ADDRESS>
    ```

    This will be used when you run the operator in step 5.

4. Export your SP1 Prover Network configuration
    ```shell
    # Export the PRIVATE_KEY you will use to deploy the contract & relay proofs.
    export PRIVATE_KEY=<PRIVATE_KEY>

    # To use the Succinct proving network, set `SP1_PRIVATE_KEY` to your private key on the proving network.
    export SP1_PRIVATE_KEY=<SP1_PRIVATE_KEY>
    ```

5. Run the Tendermint operator.
    ```shell
    cd ../operator

    SP1_PROVER=network TENDERMINT_RPC_URL=https://rpc.celestia-mocha.com/ CHAIN_ID=11155111 RPC_URL=https://ethereum-sepolia.publicnode.com/ CONTRACT_ADDRESS=<SP1_TENDERMINT_ADDRESS> RUST_LOG=info cargo run --bin operator --release
    ```

## Contract Tests
### Generate fixtures for forge tests

To generate fixtures for local testing run:

```shell
# Generates fixture.json (valid proof)
$ cd operator
$ RUST_LOG=info SP1_PROVER=network TENDERMINT_RPC_URL="https://rpc.celestia-mocha.com/" cargo run --bin fixture --release -- --trusted-block 500 --target-block 1000

# Generates mock_fixture.json (mock proof)
$ cd operator
$ RUST_LOG=info SP1_PROVER=mock TENDERMINT_RPC_URL="https://rpc.celestia-mocha.com/" cargo run --bin fixture --release -- --trusted-block 500 --target-block 1000
```

You can check that the generated fixture proofs verify by running the forge tests:
```shell
$ cd contracts
$ forge test -vvv
```