**Current contract address: erd1qqqqqqqqqqqqqpgqs04t3q7dgf2824pk5cq29geg8xze26lylqpszclta5**

# Deploy

`erdpy --verbose contract deploy --project=memes-auction --pem="devnet.pem" --gas-limit=100000000 --proxy="https://devnet-api.elrond.com" --outfile="memes-auction.json" --recall-nonce --send --chain="D" --arguments "0xHEX_ADDRESS_OF_VOTING_CONTRACT" "0xHEX_ENCODING_OF_TOKEN_IDENTIFIER" "25000000000000000"`

25000000000000000 - 0.025 EGLD

Then set the address of the auction contract in the voting contract

# Example calls
