### DEVNET
**Contract Address: erd1qqqqqqqqqqqqqpgqgu5h3wwlza4p5k6nqh9vfr9plfa87sj5lqpsa98mr6**

**Farm Token Id: METASMEMES-b6d9cf**

### MAINNET
**Contract Address: TBD**

**Farm Token Id: TBD**

# Deploy

`erdpy --verbose contract deploy --project=memes-staking --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-staking.json" --recall-nonce --chain="D" --send --arguments "VOTING_SC" "AUCTION_SC" "TOKEN_IDENTIFIER_TOP"`

Devnet:

`erdpy --verbose contract deploy --project=memes-staking --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-staking.json" --recall-nonce --chain="D" --send --arguments "erd1qqqqqqqqqqqqqpgqw6fdmgw7hk4tw6ljjrxymh8ah9lryq4flqpsgu6d86" "erd1qqqqqqqqqqqqqpgq8mwzpgp65c2rxel9a7f7v7zqr55xq9t4lqpsl6rgc8" "str:TNFTMEMES-db928b"`

Mainnet:

`erdpy --verbose contract deploy --project=memes-staking --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-staking.json" --recall-nonce --chain="D" --send --arguments "erd1qqqqqqqqqqqqqpgqjkjuya04mzesltxphap72k7n6jrme6w9a2pqd2r2ah" "erd1qqqqqqqqqqqqqpgq9drfegafhccp4sqe67leu80gkcz8keuza2pqete3vd" "str:TNFTMEMES-7e0cde"`

# Upgrade

`erdpy --verbose contract upgrade --project=memes-staking --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-staking.json" --recall-nonce --chain="D" --send "BECH32_ADDRESS" --arguments "VOTING_SC" "AUCTION_SC" "TOKEN_IDENTIFIER_TOP"`

Devnet:
`erdpy --verbose contract upgrade --project=memes-staking --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-gateway.elrond.com" --outfile="memes-staking.json" --recall-nonce --chain="D" --send "erd1qqqqqqqqqqqqqpgqgu5h3wwlza4p5k6nqh9vfr9plfa87sj5lqpsa98mr6" --arguments "erd1qqqqqqqqqqqqqpgqw6fdmgw7hk4tw6ljjrxymh8ah9lryq4flqpsgu6d86" "erd1qqqqqqqqqqqqqpgq8mwzpgp65c2rxel9a7f7v7zqr55xq9t4lqpsl6rgc8" "str:TNFTMEMES-db928b"`

## Issue farm token
`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=100000000 --function="registerFarmToken" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "0xHEX_ENCODING_OF_NAME" "0xHEX_ENCODING_OF_TICKER"`

eg: (MetaStakedMeme - METASMEME)

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=100000000 --function="registerFarmToken" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "str:MetaStakedMeme" "str:METASMEME"`

## Top up rewards

- EGLD:

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=5000000 --function="top_up_rewards" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --value EGLD_VALUE`

- ESDT:

`erdpy --verbose contract call $CONTRACT_ADDRESS --pem="devnet.pem" --value=50000000000000000 --gas-limit=5000000 --function="ESDTTransfer" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send --chain="D" --arguments "str:TOKEN" "VALUE_BIGUINT" "str:top_up_rewards"`

## Query token id

`erdpy contract query $CONTRACT_ADDRESS --function="getFarmTokenId" --proxy="https://devnet-gateway.elrond.com"`

## Query tokens

`erdpy contract query $CONTRACT_ADDRESS --function="all_reward_tokens" --proxy="https://devnet-gateway.elrond.com"`

`erdpy contract query $CONTRACT_ADDRESS --function="reward_tokens" --proxy="https://devnet-gateway.elrond.com" --arguments "str:TOKEN"`
