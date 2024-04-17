# SP1 Tendermint Template

Tendermint light client on Ethereum powered by SP1.

## Overview
This repository contains the 


## Generate Fixtures for Forge Testing
To generate fixtures for local testing run:

```shell
cd operator
cargo run --bin fixture -- --trusted-block 2 --target-block 6
```

To run the example operator with a deployed Tendermint contract run:

```shell
cd operator
cargo run --bin operator
```

https://sepolia.etherscan.io/address/0xd73F7935d19d7D37E1a5c6271b9D8BA989928C16

