### DEVNET
**Contract Address: erd1qqqqqqqqqqqqqpgq2ss8pftydnrv04kylm8e9yx8fzfkfjk4lqpsrvs2yn**

**Token Id: NFTMEMES-6f4547**


### MAINNET
**Contract Address: TBD**

**Token Id: TBD**


# Deploy

`erdpy --verbose contract deploy --project=memes-voting --pem="devnet.pem" --gas-limit=20000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-voting.json" --recall-nonce --chain="D" --metadata-payable-by-sc --send --arguments START_PERIOD_TIMESTAMP`

Then set the address of the voting contract in the creator contract

# Upgrade

`erdpy --verbose contract upgrade --project=memes-voting --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-voting.json" --recall-nonce --chain="D" --metadata-payable-by-sc --send "BECH32_ADDRESS" --arguments START_PERIOD_TIMESTAMP`

Contracts are upgradable by default. START_PERIOD_TIMESTAMP will be ignored for upgrades 

## Issue token
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=10000000 --function="issue_token" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "0xHEX_ENCODING_OF_NAME" "0xHEX_ENCODING_OF_TICKER"`

eg: (NftMemes - NFTMEMES)

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=10000000 --function="issue_token" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "0x4e66744d656d6573" "0x4e46544d454d4553"`

## Set local roles

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=100000000 --function="set_local_roles" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D"`

## Query token id

`erdpy contract query $CONTRACT_ADDRESS --function="token_identifier" --proxy="https://devnet-gateway.elrond.com"`

## Add categories
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=10000000 --function="modify_categories" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments 0xCATEGORY_NAME`

Funny category:

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=10000000 --function="modify_categories" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments 0x66756e6e79`

## Set signer address
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=10000000 --function="set_signer" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "0xSPECIAL_HEX_ENCODING_OF_ADDRESS"`

## Linking auction contract
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=100000000 --function="set_auction_sc" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "0xSPECIAL_HEX_ENCODING_OF_OTHER_CONTRACT"`

# Example calls

### Query categories
`erdpy contract query $CONTRACT_ADDRESS --function="categories" --proxy="http
s://devnet-gateway.elrond.com"`

### Create Meme
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=50000000 --function="create_meme" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments 0x746573742033 0x68747470733a2f2f697066732e6d6f72616c69732e696f3a323035332f697066732f516d645a634b7836374d4571677a64376631774e345755484253454661446f6978777046544b347175567175514d`

### Pause contract
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=5000000 --function="pause" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D"`

### Unpause contract
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=5000000 --function="unpause" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D"`
