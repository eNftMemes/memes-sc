#![no_std]

use owner::*;

elrond_wasm::imports!();

mod owner;

#[elrond_wasm::contract]
pub trait StakingContract: owner::OwnerModule
    + elrond_wasm_modules::pause::PauseModule
{
    #[init]
    fn init(&self, voting_contract: &ManagedAddress, auction_contract: &ManagedAddress, token_identifier_top: &TokenIdentifier) {
        self.voting_sc().set(voting_contract);
        self.auction_sc().set(auction_contract);
        self.token_identifier_top().set(token_identifier_top);
    }

    #[view]
    #[storage_mapper("votingSc")]
    fn voting_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[view]
    #[storage_mapper("auctionSc")]
    fn auction_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[view]
    #[storage_mapper("tokenIdentifierTop")]
    fn token_identifier_top(&self) -> SingleValueMapper<TokenIdentifier>;
}
