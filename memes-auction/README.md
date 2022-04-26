### DEVNET
**Contract Address: erd1qqqqqqqqqqqqqpgq9f60ttjzedw78g9k975ual4z73cufd80lqpsev9nrm**

### TESTNET
**Contract Address: erd1qqqqqqqqqqqqqpgqxs0hnym50kule2jlthmzcvdmdfgyzyrya2pqdk5cx4**

### MAINNET
**Contract Address: TBD**


# Deploy

First decode voting contract address to hex using erdpy:

`erdpy wallet bech32 --decode $CONTRACT_ADDRESS`

Then deploy:

`erdpy --verbose contract deploy --project=memes-auction --pem="devnet.pem" --gas-limit=50000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-auction.json" --recall-nonce --chain="D" --metadata-payable-by-sc --send --arguments "0xHEX_ADDRESS_OF_VOTING_CONTRACT" "0xHEX_ENCODING_OF_TOKEN_IDENTIFIER" "25000000000000000"`

25000000000000000 - 0.025 EGLD

**Then set the address of the auction contract in the voting contract!**

# Upgrade

`erdpy --verbose contract upgrade --project=memes-auction --pem="devnet.pem" --gas-limit=50000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-auction.json" --recall-nonce --chain="D" --metadata-payable-by-sc --send "BECH32_ADDRESS" --arguments "0xHEX_ADDRESS_OF_VOTING_CONTRACT" "0xHEX_ENCODING_OF_TOKEN_IDENTIFIER" "25000000000000000"`

## Issue token
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=10000000 --function="issue_token" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "0xHEX_ENCODING_OF_NAME" "0xHEX_ENCODING_OF_TICKER"`

eg: (TopNftMemes - TNFTMEMES)

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=10000000 --function="issue_token" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "0x546f704e66744d656d6573" "0x544e46544d454d4553"`

## Set local roles

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=100000000 --function="set_local_roles" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D"`

## Query token id
erdpy contract query $CONTRACT_ADDRESS --function="token_identifier_top" --proxy="https://devnet-gateway.elrond.com"

# Example calls
