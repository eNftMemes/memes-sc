# Deploy

First create a new NFT token, since once the role has been transferred there is no getting it back (unless implemented in contract).

`erdpy --verbose contract deploy --project=memes-creator --pem="devnet.pem" --gas-limit=60000000 --proxy="https://devnet-api.elrond.com" --outfile="memes-creator.json" --recall-nonce --send --chain="D" --arguments "0xHEX_ENCODING_OF_NFT_IDENTIFIER"`

**Afterwards, transfer the NFT create role to the contract, from the old contract with the following transaction:**
```
receiver: erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u
data: transferNFTCreateRole@NFT_IN_HEX@8287406b9ba0a47d327f1288e3460f2cb6bbcbedfd8d449621e6223b4b37f803@NEW_CONTRACT_ADDRESS_HEX
```

Decode bech32 address to HEX using:
`erdpy wallet bech32 --decode erd1qqqqqqqqqqqqqpgqeynfjc4kntv8xkjmtfulgakgy5gluuuzlqpsep9f3j`

**Current contract address: erd1qqqqqqqqqqqqqpgq0new7jhwqyqz6tc84rfuhmn0fs7k9et0lqpsnxz8wc**

# Linking voting contract
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=50000000 --function="set_voting_sc" --proxy="https://devnet-api.elrond.com" --recall-nonce --send --chain="D" --arguments "0xHEX_ENCODING_OF_OTHER_CONTRACT"`

# Example calls

Add category:
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=50000000 --function="modify_categories" --proxy="https://devnet-api.elrond.com" --recall-nonce --send --chain="D" --arguments 0 0xCATEGORY_NAME`

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=50000000 --function="create_meme" --proxy="https://devnet-api.elrond.com" --recall-nonce --send --chain="D" --arguments 0x746573742033 0x68747470733a2f2f697066732e6d6f72616c69732e696f3a323035332f697066732f516d645a634b7836374d4571677a64376631774e345755484253454661446f6978777046544b347175567175514d`
