**Current contract address: erd1qqqqqqqqqqqqqpgqr5ycsth89qgv5r8fq7jl06axd2rht7d4lqpsdlfsma**

# Deploy

First decode creator contract address to hex using erdpy:

`erdpy wallet bech32 --decode ADDRESS`

Then deploy:

`erdpy --verbose contract deploy --project=memes-voting --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-api.elrond.com" --outfile="memes-voting.json" --recall-nonce --send --chain="D" --arguments "0xSPECIAL_HEX_ADDRESS_OF_CREATOR_CONTRACT" START_PERIOD_TIMESTAMP`

Then set the address of the voting contract in the creator contract


# Example calls
