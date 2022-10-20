### DEVNET
**Contract Address: erd1qqqqqqqqqqqqqpgq8mwzpgp65c2rxel9a7f7v7zqr55xq9t4lqpsl6rgc8**

**Top Token Id: TNFTMEMES-db928b**

### MAINNET
**Contract Address: erd1qqqqqqqqqqqqqpgq9drfegafhccp4sqe67leu80gkcz8keuza2pqete3vd**

**Token Id: TNFTMEMES-7e0cde**

# Deploy

First decode voting contract address to hex using erdpy:

`erdpy wallet bech32 --decode $CONTRACT_ADDRESS`

Then deploy:

`erdpy --verbose contract deploy --project=memes-auction --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-auction.json" --recall-nonce --chain="D" --metadata-payable-by-sc --send --arguments "0xHEX_ADDRESS_OF_VOTING_CONTRACT" "0xHEX_ENCODING_OF_TOKEN_IDENTIFIER" "25000000000000000"`

25000000000000000 - 0.025 EGLD

# Upgrade

`erdpy --verbose contract upgrade --project=memes-auction --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-auction.json" --recall-nonce --chain="D" --metadata-payable-by-sc --send "BECH32_ADDRESS" --arguments "0xHEX_ADDRESS_OF_VOTING_CONTRACT" "0xHEX_ENCODING_OF_TOKEN_IDENTIFIER" "25000000000000000"`

Devnet:

`erdpy --verbose contract upgrade --project=memes-auction --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-auction.json" --recall-nonce --chain="D" --metadata-payable-by-sc --send "erd1qqqqqqqqqqqqqpgq8mwzpgp65c2rxel9a7f7v7zqr55xq9t4lqpsl6rgc8" --arguments "erd1qqqqqqqqqqqqqpgqw6fdmgw7hk4tw6ljjrxymh8ah9lryq4flqpsgu6d86" "str:NFTMEMES-706046" "25000000000000000"`

Mainnet:

`erdpy --verbose contract upgrade --project=memes-auction --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-auction.json" --recall-nonce --chain="D" --metadata-payable-by-sc "erd1qqqqqqqqqqqqqpgq9drfegafhccp4sqe67leu80gkcz8keuza2pqete3vd" --arguments "erd1qqqqqqqqqqqqqpgqjkjuya04mzesltxphap72k7n6jrme6w9a2pqd2r2ah" "str:NFTMEMES-0300ba" "25000000000000000"`

## Issue token
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=10000000 --function="issue_token" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "0xHEX_ENCODING_OF_NAME" "0xHEX_ENCODING_OF_TICKER"`

eg: (TopNftMemes - TNFTMEMES)

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=100000000 --function="issue_token" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "0x546f704e66744d656d6573" "0x544e46544d454d4553"`

## Set local roles

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=100000000 --function="set_local_roles" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D"`

## Linking staking contract
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=10000000 --function="set_staking_sc" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "STAKING_CONTRACT"`

## Query token id
`erdpy contract query $CONTRACT_ADDRESS --function="token_identifier_top" --proxy="https://devnet-gateway.elrond.com"`

### Then set the address of the auction contract in the voting contract!

# Example calls

## Set custom attributes
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=4000000 --function="set_custom_attributes" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --chain="D" --arguments "NONCE" "CATEGORY" "RARITY"`

Example:

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=4000000 --function="set_custom_attributes" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --chain="D" --arguments "2" "str:eNftMemes Official" "255"`

## Add custom auction
The OPTIONAL_MULTIPLIER is 20 if not specified (for 0.025 min bid that's 0.025 * 20 = 0.5 EGLD).

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=5000000 --function="add_custom_auction" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --chain="D" --arguments "PERIOD" "NONCE" "OPTIONAL_MULTIPLIER"`
