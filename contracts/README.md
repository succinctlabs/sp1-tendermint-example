## Testing
Once you've added a fixture to `fixtures/fixture.json`, you can run tests with:

```shell
forge test -vvv
```


### Deploy

```shell
$ forge script script/SP1Tendermint.s.sol --rpc-url $RPC_11155111 --private-key $PRIVATE_KEY --etherscan-api-key $ETHERSCAN_API_KEY_11155111 --broadcast --verify
```
