### DEVNET
**Contract Address: erd1qqqqqqqqqqqqqpgqw6fdmgw7hk4tw6ljjrxymh8ah9lryq4flqpsgu6d86**

**Token Id: NFTMEMES-706046**

### MAINNET
**Contract Address: erd1qqqqqqqqqqqqqpgqjkjuya04mzesltxphap72k7n6jrme6w9a2pqd2r2ah**

**Token Id: NFTMEMES-0300ba**

# Deploy

`erdpy --verbose contract deploy --project=memes-voting --pem="devnet.pem" --gas-limit=20000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-voting.json" --recall-nonce --chain="D" --metadata-payable-by-sc --send --arguments START_PERIOD_TIMESTAMP`

# Upgrade

`erdpy --verbose contract upgrade --project=memes-voting --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-voting.json" --recall-nonce --chain="D" --metadata-payable-by-sc --send "BECH32_ADDRESS" --arguments START_PERIOD_TIMESTAMP`

Contracts are upgradable by default. START_PERIOD_TIMESTAMP will be ignored for upgrades

Devnet:
`erdpy --verbose contract upgrade --project=memes-voting --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-voting.json" --recall-nonce --chain="D" --metadata-payable-by-sc --send "erd1qqqqqqqqqqqqqpgqw6fdmgw7hk4tw6ljjrxymh8ah9lryq4flqpsgu6d86" --arguments 0`

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

On Devnet the signer is wallet address `erd1zq5zmnqjdlymzxg6av0623vw8ke6fmp8qkk4lqn0gt2nxca2mh2sayqt3g`

## Linking auction contract
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=100000000 --function="set_auction_sc" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "0xSPECIAL_HEX_ENCODING_OF_AUCTION_CONTRACT"`

## Linking staking contract
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=10000000 --function="set_staking_sc" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "STAKING_CONTRACT"`

## Set period time
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=10000000 --function="set_period_time" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "PERIOD_TIME_IN_SECONDS"`

# Example calls

### Query categories
`erdpy contract query $CONTRACT_ADDRESS --function="categories" --proxy="http
s://devnet-gateway.elrond.com"`

### Create Meme
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=50000000 --function="create_meme" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --chain="D" --arguments 0x746573742033 0x68747470733a2f2f697066732e6d6f72616c69732e696f3a323035332f697066732f516d645a634b7836374d4571677a64376631774e345755484253454661446f6978777046544b347175567175514d`

## Set period time (2 weeks)
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=10000000 --function="set_period_time" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --chain="D" --arguments "1209600"`

## Set period time (4 weeks)
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=10000000 --function="set_period_time" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --chain="D" --arguments "2419200"`

### Pause contract
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=5000000 --function="pause" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D"`

### Unpause contract
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=5000000 --function="unpause" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D"`
