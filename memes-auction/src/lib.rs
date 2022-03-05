#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();

mod owner;
use owner::AUCTION_TIME;

mod auction;
use auction::*;

const PERCENTAGE_TOTAL: u64 = 10_000; // 100%
const NFT_AMOUNT: u32 = 1;
const MAX_TOP_MEMES: usize = 10;

const BID_TIME: u64 = 345600; // 4 days in seconds (time users can bid on NFTs)

#[elrond_wasm::contract]
pub trait MemesAuction: owner::OwnerModule {
	#[init]
	fn init(&self, voting_contract: &ManagedAddress, token_identifier: &TokenIdentifier, min_bid_start: &BigUint) {
		self.voting_contract().set(voting_contract);
		self.token_identifier().set(token_identifier);
		self.min_bid_start().set(min_bid_start);

		if self.bid_cut_percentage().is_empty() {
			let bid_cut: u16 = 1000;
			self.bid_cut_percentage().set(&bid_cut);
		}
	}

	#[endpoint]
	fn start_auction(&self, period: u64, #[var_args] nfts: MultiValueEncoded<u64>) {
		let caller: ManagedAddress = self.blockchain().get_caller();
		require!(
			caller == self.voting_contract().get(),
			"Only voting contract can call this"
		);
		require!(
			nfts.len() <= MAX_TOP_MEMES,
			"There can't be more than 10 nfts"
		);
		require!(
			self.period_auctioned_memes(period).is_empty(),
			"There are already auctioned nfts for this period"
		);

		let bid_cut_percentage: u16 = self.bid_cut_percentage().get();
		let min_bid_start: BigUint = self.min_bid_start().get();
		let mut multiplier: u8 = nfts.len() as u8;

		// 1st meme is the one on 1st place etc
		for nonce in nfts.into_iter() {
			self.add_auction(&period, &bid_cut_percentage, &min_bid_start, &multiplier, &nonce);

			let meme_rarity = self.meme_rarity(nonce);
			if meme_rarity.is_empty() || multiplier > meme_rarity.get() {
				let rarity: u8 = multiplier;
				meme_rarity.set(&rarity);
			}

			multiplier -= 1;
		}
	}

	#[payable("*")]
	#[endpoint]
	fn lock_token(
		&self,
		period: u64,
		#[payment_token] nft_type: TokenIdentifier,
		#[payment_nonce] nonce: u64,
		#[payment_amount] nft_amount: BigUint,
	) {
		require!(nft_type == self.token_identifier().get(), "Nft is not of the correct type");
		require!(nft_amount == NFT_AMOUNT, "Nft amount should be 1");

		let block_timestamp = self.blockchain().get_block_timestamp();

		require!(
            block_timestamp > period && block_timestamp - period < AUCTION_TIME,
            "Auction deadline has passed"
        );

		let mut auction = self.try_get_auction(period, nonce);
		let caller = self.blockchain().get_caller();

		auction.original_owner = caller;

		self.period_meme_auction(period, nonce).set(&auction);
	}

	#[payable("EGLD")]
	#[endpoint]
	fn bid(
		&self,
		#[payment_amount] payment_amount: BigUint,
		period: u64,
		nonce: u64
	) {
		let mut auction = self.try_get_auction(period, nonce);
		let caller = self.blockchain().get_caller();
		let block_timestamp = self.blockchain().get_block_timestamp();

		require!(
			block_timestamp > period && block_timestamp - period < BID_TIME,
            "Auction bidding ended already"
        );
		require!(
            auction.original_owner != caller,
            "Can't bid on your own token"
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
	}

	#[endpoint]
	fn end_auction(&self, period: u64, nonce: u64) {
		let mut auction = self.try_get_auction(period, nonce);
		let block_timestamp = self.blockchain().get_block_timestamp();

		require!(
            block_timestamp > period && block_timestamp - period >= AUCTION_TIME,
            "Auction deadline has not passed"
        );
		require!(
			!auction.ended,
			"Auction was already ended"
		);

		auction.ended = true;

		self.period_meme_auction(period, nonce).set(&auction);

		self.distribute_tokens_after_auction_end(&nonce, &auction);
	}

	#[payable("*")]
	#[endpoint]
	fn upgrade_token(
		&self,
		#[payment_token] nft_type: TokenIdentifier,
		#[payment_nonce] nonce: u64,
		#[payment_amount] nft_amount: BigUint,
	) {
		require!(nft_type == self.token_identifier().get(), "Nft is not of the correct type");
		require!(nft_amount == NFT_AMOUNT, "Nft amount should be 1");
		require!(!self.meme_rarity(nonce).is_empty(), "Nft can't be upgraded");

		self.update_nft_attributes(
			&self.blockchain().get_caller(),
			&nonce,
			b"nft upgraded"
		);
	}

	// private

	fn try_get_auction(&self, period: u64, nonce: u64) -> Auction<Self::Api> {
		let auction = self.period_meme_auction(period, nonce);

		require!(
            !auction.is_empty(),
            "Auction does not exist"
        );

		auction.get()
	}

	fn distribute_tokens_after_auction_end(&self, nft_nonce: &u64, auction: &Auction<Self::Api>) {
		if auction.original_owner.is_zero() {
			if auction.current_winner.is_zero() {
				return;
			}

			// return money to current winner
			self.send().direct_egld(
				&auction.current_winner,
				&auction.current_bid,
				b"bid refund",
			);

			return;
		}

		if auction.current_winner.is_zero() {
			// return nft to original owner
			self.update_nft_attributes(&auction.original_owner, nft_nonce, b"returned token");

			return;
		}

		let bid_cut_percentage = BigUint::from(auction.bid_cut_percentage);
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
		self.update_nft_attributes(&auction.current_winner, nft_nonce, b"bought token at auction");
	}

	fn update_nft_attributes(&self, send_to: &ManagedAddress, nft_nonce: &u64, text: &[u8]) {
		let nft_token = &self.token_identifier().get();
		let amount = BigUint::from(NFT_AMOUNT);

		let own_address: ManagedAddress = self.blockchain().get_sc_address();
		let token_data: EsdtTokenData<Self::Api> = self.blockchain().get_esdt_token_data(&own_address, nft_token, *nft_nonce);
		let mut new_attributes = token_data.decode_attributes::<MemeAttributes<Self::Api>>();

		if !self.meme_rarity(*nft_nonce).is_empty() && self.meme_rarity(*nft_nonce).get() > new_attributes.rarity {
			new_attributes.rarity = self.meme_rarity(*nft_nonce).get();

			self.send().nft_update_attributes(
				&self.token_identifier().get(),
				*nft_nonce,
				&new_attributes
			);
		}

		self.send().direct(
			send_to,
			nft_token,
			*nft_nonce,
			&amount,
			text,
		);

		self.meme_rarity(*nft_nonce).clear();
	}

	// views/storage

	#[view]
	fn period_auctions_memes_all(&self, period: u64) -> MultiValueEncoded<FullAuction<Self::Api>> {
		let mut result: MultiValueEncoded<FullAuction<Self::Api>> = MultiValueEncoded::new();
		for index in 1..=self.period_auctioned_memes(period).len() {
			let nonce = self.period_auctioned_memes(period).get(index);
			let auction = self.period_meme_auction(period, nonce).get();

			let full_auction = FullAuction {
				nonce,
				auction,
			};

			result.push(full_auction);
		}

		return result;
	}

	#[view]
	#[storage_mapper("votingContract")]
	fn voting_contract(&self) -> SingleValueMapper<ManagedAddress>;

	#[view]
	#[storage_mapper("tokenIdentifier")]
	fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

	// The rarity of a meme depending on the place the meme was in an auction, to be used in the future
	// If an auction has less than 10 memes, the max rarity is < 10
	// 10 - 1st place, most rare
	// 1 - 10th place, most common
	#[view]
	#[storage_mapper("memeRarity")]
	fn meme_rarity(&self, nonce: u64) -> SingleValueMapper<u8>;
}
