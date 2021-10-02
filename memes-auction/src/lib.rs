#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait MemesAuction {
	#[init]
	fn init(&self, voting_contract: &ManagedAddress) {
		self.voting_contract().set(voting_contract);
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

	#[storage_mapper("votingContract")]
	fn voting_contract(&self) -> SingleValueMapper<ManagedAddress>;

    // Always needed
    #[endpoint]
    #[only_owner]
    fn claim(&self) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        self.send().direct_egld(&caller, &self.blockchain().get_sc_balance(&self.types().token_identifier_egld(), 0), b"claiming success");
        Ok(())
    }
}
