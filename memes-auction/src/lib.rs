#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();

pub mod auction;
use auction::*;

// let min_bid_start: BigUint = BigUint::from("1000000000000000"); // 0.001 EGLD

const PERCENTAGE_TOTAL: u64 = 10_000; // 100%
const NFT_AMOUNT: u32 = 1;

const BID_TIME: u64 = 345600; // 4 days in seconds (time users can bid on NFTs)
const AUCTION_TIME: u64 = 432000; // 5 days in seconds (time the owner of the NFT has for locking it)

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
			"There can't be more than 10 nfts"
		);

		// let bid_cut_percentage: u16 = self.bid_cut_percentage().get();
		// let min_bid_start: BigUint = self.min_bid_start().get();
		// let mut multiplier: u8 = nfts.len() as u8;

		for nonce in nfts.iter() {
			// let mut min_bid: BigUint = self.types().big_uint_from(multiplier);
			// min_bid *= &min_bid_start;
			// multiplier -= 1;

			let auction = Auction {
				min_bid: self.types().big_uint_from(1u64),
				current_bid: self.types().big_uint_from(2u64),
				current_winner: self.types().managed_address_zero(),
				bid_cut_percentage: 3,
				original_owner: self.types().managed_address_zero(),
				ended: false,
			};
			self.period_meme_auction(period, *nonce).set(&auction);
			self.period_auctioned_memes(period).push(nonce);
		}

		Ok(())
	}

	#[payable("*")]
	#[endpoint]
	fn lock_token(
		&self,
		period: u64,
		#[payment_token] nft_type: TokenIdentifier,
		#[payment_nonce] nonce: u64,
		#[payment_amount] nft_amount: BigUint,
	) -> SCResult<()> {
		require!(nft_type == self.token_identifier().get(), "Nft is not of the correct type");
		require!(nft_amount == NFT_AMOUNT, "Nft amount should be 1");

		let block_timestamp = self.blockchain().get_block_timestamp();

		require!(
            block_timestamp > period && block_timestamp - period < AUCTION_TIME,
            "Auction deadline has passed"
        );

		let mut auction = self.try_get_auction(period, nonce)?;
		let caller = self.blockchain().get_caller();

		auction.original_owner = caller;

		self.period_meme_auction(period, nonce).set(&auction);

		Ok(())
	}

	#[payable("EGLD")]
	#[endpoint]
	fn bid(
		&self,
		#[payment_amount] payment_amount: BigUint,
		period: u64,
		nonce: u64
	) -> SCResult<()> {
		let mut auction = self.try_get_auction(period, nonce)?;
		let caller = self.blockchain().get_caller();
		let block_timestamp = self.blockchain().get_block_timestamp();

		require!(
			block_timestamp > period && block_timestamp - period < BID_TIME,
            "Auction bidding ended already"
        );
		require!(auction.current_winner != caller, "Can't outbid yourself");
		require!(
            payment_amount >= auction.min_bid,
            "Bid must be higher than or equal to the min bid"
        );
		require!(
            payment_amount > auction.current_bid,
            "Bid must be higher than the current winning bid"
        );

		// refund losing bid
		if !auction.current_winner.is_zero() {
			self.send().direct_egld(
				&auction.current_winner,
				&auction.current_bid,
				b"bid refund",
			);
		}

		// update auction bid and winner
		auction.current_bid = payment_amount;
		auction.current_winner = caller;

		self.period_meme_auction(period, nonce).set(&auction);

		Ok(())
	}

	#[endpoint]
	fn end_auction(&self, period: u64, nonce: u64) -> SCResult<()> {
		let mut auction = self.try_get_auction(period, nonce)?;
		let block_timestamp = self.blockchain().get_block_timestamp();

		require!(
            block_timestamp > period && block_timestamp - period >= AUCTION_TIME,
            "Auction deadline has not passed"
        );

		self.distribute_tokens_after_auction_end(nonce, &auction);

		auction.ended = true;

		self.period_meme_auction(period, nonce).set(&auction);

		Ok(())
	}

	// private

	fn try_get_auction(&self, period: u64, nonce: u64) -> SCResult<Auction<Self::Api>> {
		let auction = self.period_meme_auction(period, nonce);

		require!(
            !auction.is_empty(),
            "Auction does not exist"
        );

		Ok(auction.get())
	}

	fn distribute_tokens_after_auction_end(&self, nft_nonce: u64, auction: &Auction<Self::Api>) {
		let nft_type = &self.token_identifier().get();
		let nft_amount_to_send = BigUint::from(NFT_AMOUNT);
		let bid_cut_percentage = BigUint::from(auction.bid_cut_percentage);

		if auction.current_winner.is_zero() && !auction.original_owner.is_zero() {
			// return nft to original owner
			self.send().direct(
				&auction.original_owner,
				nft_type,
				nft_nonce,
				&nft_amount_to_send,
				b"returned token",
			);

			return;
		}

		if !auction.current_winner.is_zero() {
			if auction.original_owner.is_zero() {
				// return money to current winner
				self.send().direct_egld(
					&auction.current_winner,
					&auction.current_bid,
					b"bid refund",
				);

				return;
			}

			let bid_cut = &auction.current_bid * &bid_cut_percentage / PERCENTAGE_TOTAL;
			let mut owner_cut = auction.current_bid.clone();
			owner_cut -= &bid_cut;

			// send part as cut for contract owner
			let owner = self.blockchain().get_owner_address();
			self.send().direct_egld(
				&owner,
				&bid_cut,
				b"bid cut for sold token",
			);

			// send rest of the bid to original owner
			self.send().direct_egld(
				&auction.original_owner,
				&owner_cut,
				b"sold token",
			);

			// send NFT to auction winner
			self.send().direct(
				&auction.current_winner,
				nft_type,
				nft_nonce,
				&nft_amount_to_send,
				b"bought token at auction",
			);
		}
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
	#[storage_mapper("periodMemeAuction")]
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
