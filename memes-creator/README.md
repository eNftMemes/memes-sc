# Deploy

`erdpy --verbose contract deploy --project=memes-creator --pem="devnet.pem" --gas-limit=50000000 --proxy="https://devnet-api.elrond.com" --outfile="memes-creator.json" --recall-nonce --send --chain="D"`

**Afterwards, transfer the NFT create role to the contract, from the old contract with the following transaction:**
```
receiver: erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u
data: transferNFTCreateRole@54524950412d626237663563@00000000000000000500d299bc14759ea0666a848db4ae68299caa4e110ff803@NEW_CONTRACT_ADDRESS_HEX
```

Old contract address: erd1qqqqqqqqqqqqqpgq62vmc9r4n6sxv65y3k62u6pfnj4yuyg0lqpspet9sd

# Example calls

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=50000000 --function="create_meme" --proxy="https://devnet-api.elrond.com" --recall-nonce --send --chain="D" --arguments 0x746573742033 0x68747470733a2f2f697066732e6d6f72616c69732e696f3a323035332f697066732f516d645a634b7836374d4571677a64376631774e345755484253454661446f6978777046544b347175567175514d`
