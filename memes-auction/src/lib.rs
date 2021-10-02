#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait MemesAuction {
	#[init]
	fn init(&self, voting_contract: &ManagedAddress, token_identifier: &TokenIdentifier) {
		self.voting_contract().set(voting_contract);
		self.token_identifier().set(token_identifier);
		let bid_cut: u16 = 500;
		self.bid_cut_percentage().set(&bid_cut);
	}

	#[only_owner]
	#[endpoint]
	fn set_bid_cut_percentage(&self, bid_cut: u16) -> SCResult<()> {
		require!(bid_cut > 100 && bid_cut < 2500, "Bid cut percentage can not be less than 1% and greater than 25%");

		self.bid_cut_percentage().set(&bid_cut);

		Ok(())
	}

	#[endpoint]
	fn start_auction(&self) -> SCResult<()> {
		let caller: ManagedAddress = self.blockchain().get_caller();
		require!(
			caller == self.voting_contract().get(),
			"Only voting contract can call this"
		);

		Ok(())
	}

	#[view]
	#[storage_mapper("votingContract")]
	fn voting_contract(&self) -> SingleValueMapper<ManagedAddress>;

	#[view]
	#[storage_mapper("tokenIdentifier")]
	fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

	#[view]
	#[storage_mapper("bidCutPercentage")]
	fn bid_cut_percentage(&self) -> SingleValueMapper<u16>;

    // Always needed
    #[endpoint]
    #[only_owner]
    fn claim(&self) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        self.send().direct_egld(&caller, &self.blockchain().get_sc_balance(&self.types().token_identifier_egld(), 0), b"claiming success");
        Ok(())
    }
}
