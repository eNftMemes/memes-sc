#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();

pub mod auction;
use auction::*;

// let min_bid_start: BigUint = BigUint::from("1000000000000000"); // 0.001 EGLD

#[elrond_wasm::contract]
pub trait MemesAuction {
	#[init]
	fn init(&self, voting_contract: &ManagedAddress, token_identifier: &TokenIdentifier, min_bid_start: &BigUint) {
		self.voting_contract().set(voting_contract);
		self.token_identifier().set(token_identifier);
		let bid_cut: u16 = 500;
		self.bid_cut_percentage().set(&bid_cut);
		self.min_bid_start().set(min_bid_start);
	}

	#[only_owner]
	#[endpoint]
	fn set_bid_cut_percentage(&self, bid_cut: u16) -> SCResult<()> {
		require!(bid_cut > 100 && bid_cut < 2500, "Bid cut percentage can not be less than 1% and greater than 25%");

		self.bid_cut_percentage().set(&bid_cut);

		Ok(())
	}

	#[only_owner]
	#[endpoint]
	fn set_min_bid_start(&self, min_bid_start: &BigUint) -> SCResult<()> {
		self.min_bid_start().set(min_bid_start);

		Ok(())
	}

	#[endpoint]
	fn start_auction(&self, period: u64, #[var_args] nfts: VarArgs<u64>) -> SCResult<()> {
		let caller: ManagedAddress = self.blockchain().get_caller();
		require!(
			caller == self.voting_contract().get(),
			"Only voting contract can call this"
		);
		require!(
			nfts.len() <= 10,
			"There can't be more than 10 nfts in the auction. Something went wrong..."
		);

		let bid_cut_percentage: u16 = self.bid_cut_percentage().get();
		let min_bid_start: BigUint = self.min_bid_start().get();

		let memes: Vec<u64> = nfts.into_vec();
		for index in (0..memes.len()).rev() {
			let nonce: u64 = memes[index];
			let mut min_bid: BigUint = self.types().big_uint_from((index + 1) as u64);
			min_bid *= &min_bid_start;

			let auction = Auction {
				min_bid,
				current_bid: self.types().big_uint_zero(),
				current_winner: self.types().managed_address_zero(),
				bid_cut_percentage,
				owner_payed: false,
			};
			self.period_meme_auction(period, nonce).set(&auction);
			self.period_auctioned_memes(period).push(&nonce);
		}

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

	#[view]
	#[storage_mapper("minBidStart")]
	fn min_bid_start(&self) -> SingleValueMapper<BigUint>;

	#[view]
	#[storage_mapper("periodAuctionMemes")]
	fn period_meme_auction(&self, period: u64, nonce: u64) -> SingleValueMapper<Auction<Self::Api>>;

	#[view]
	#[storage_mapper("periodAuctionedMemes")]
	fn period_auctioned_memes(&self, period: u64) -> VecMapper<u64>;

    // Always needed
    #[endpoint]
    #[only_owner]
    fn claim(&self) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        self.send().direct_egld(&caller, &self.blockchain().get_sc_balance(&self.types().token_identifier_egld(), 0), b"claiming success");
        Ok(())
    }
}
