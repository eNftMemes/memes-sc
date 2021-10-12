**Current contract address:** erd1qqqqqqqqqqqqqpgqyslvzamgpw7e6dxr946307dpvewygt4qlqpsn689h4

# Deploy

`erdpy --verbose contract deploy --project=memes-creator --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-api.elrond.com" --outfile="memes-creator.json" --recall-nonce --send --chain="D"`

Contract is upgradable by default.

## Issue token
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=100000000 --function="issue_token" --proxy="https://devnet-api.elrond.com" --recall-nonce --send --chain="D" --arguments "0xHEX_ENCODING_OF_NAME" "0xHEX_ENCODING_OF_TICKER"`

eg: (TripalovskyNFT - TRPNFT)

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=100000000 --function="issue_token" --proxy="https://devnet-api.elrond.com" --recall-nonce --send --chain="D" --arguments "0x54726970616c6f76736b794e4654" "0x5452504e4654"`

## Query token id

`erdpy contract query $CONTRACT_ADDRESS --function="token_identifier" --proxy="https://devnet-gateway.elrond.com"`

## Linking voting contract
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=50000000 --function="set_voting_sc" --proxy="https://devnet-api.elrond.com" --recall-nonce --send --chain="D" --arguments "0xSPECIAL_HEX_ENCODING_OF_OTHER_CONTRACT"`

# Example calls

Add category:
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=50000000 --function="modify_categories" --proxy="https://devnet-api.elrond.com" --recall-nonce --send --chain="D" --arguments 0 0xCATEGORY_NAME`

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --gas-limit=50000000 --function="create_meme" --proxy="https://devnet-api.elrond.com" --recall-nonce --send --chain="D" --arguments 0x746573742033 0x68747470733a2f2f697066732e6d6f72616c69732e696f3a323035332f697066732f516d645a634b7836374d4571677a64376631774e345755484253454661446f6978777046544b347175567175514d`
