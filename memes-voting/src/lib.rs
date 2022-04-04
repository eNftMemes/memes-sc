#![no_std]
#![feature(generic_associated_types)] // needed to use ManagedVecItem derive for MemeVotes
#![allow(unused_attributes)]

elrond_wasm::imports!();

mod owner;
use owner::*;

mod meme;
use meme::*;

const THROTTLE_MEME_TIME: u64 = 600; // 10 minutes in seconds
const NFT_AMOUNT: u32 = 1;
const PER_PAGE: usize = 10;
const PERIOD_TIME: u64 = 604800; // 1 week in seconds
const VOTES_PER_ADDRESS_PER_PERIOD: u8 = 20;

mod auction_proxy {
	elrond_wasm::imports!();

	#[elrond_wasm::proxy]
	pub trait Auction {
		#[endpoint]
		fn start_auction(&self, period: u64, #[var_args] nfts: MultiValueEncoded<u64>) -> SCResult<()>;
	}
}

#[elrond_wasm::contract]
pub trait MemesVoting: owner::OwnerModule
	+ elrond_wasm_modules::pause::PauseModule
{
	#[init]
	fn init(&self, period: &u64) {
		if self.periods().len() == 0 {
			let royalties: u16 = 500;
			self.nft_royalties().set(&royalties);

			self.periods().push(period);
		}
	}

	#[endpoint]
	fn create_meme(&self, name: ManagedBuffer, url: ManagedBuffer, category: ManagedBuffer, signature: Signature<Self::Api>) {
		require!(self.not_paused(), "Contract paused, can't create new memes");

		let caller: ManagedAddress = self.blockchain().get_caller();

		self.verify_signature(&caller, &url, &signature);

		let address: ManagedAddress = self.blockchain().get_caller();
		let block_timestamp: u64 = self.blockchain().get_block_timestamp();
		let address_meme_time: SingleValueMapper<u64> = self.address_last_meme_time(&address);

		require!(
			address_meme_time.is_empty()
				|| address_meme_time.get() < block_timestamp - THROTTLE_MEME_TIME,
			"Address already created a meme in the last 10 minutes"
		);
		require!(
			self.categories().contains(&category),
			"Category does not exist"
		);

		self.address_last_meme_time(&address).set(&block_timestamp);

		let amount: &BigUint = &BigUint::from(NFT_AMOUNT);
		let royalties: &BigUint = &BigUint::from(self.nft_royalties().get());
		let nft_token: &TokenIdentifier = &self.token_identifier().get();
		let hash: &ManagedBuffer = &ManagedBuffer::new();
		let mut urls = ManagedVec::new();
		urls.push(url);

		let async_call: OptionalValue<AsyncCall> = self.alter_period();
		let current_period: u64 = self.current_period();

		// TODO: Test this function when it works properly on Devnet
		let nonce: u64 = self.send().esdt_nft_create_as_caller(
			nft_token,
			&amount,
			&name,
			royalties,
			hash,
			&MemeAttributes { period: current_period, category, rarity: 0 },
			&urls
		);

		self.send().direct(&address, nft_token, nonce, amount, &[]);

		self.period_memes(current_period).push(&nonce);

		if let OptionalValue::Some(ac) = async_call {
			ac.call_and_exit()
		}
	}

	#[endpoint]
	fn vote_memes(&self, #[var_args] nft_nonces: MultiValueEncoded<u64>) {
		require!(self.not_paused(), "Contract paused, can't vote memes");

		let caller: ManagedAddress = self.blockchain().get_caller();

		let address_votes: SingleValueMapper<AddressVotes> = self.address_votes(caller);
		let current_period: u64 = self.current_period();
		let reset_address_votes = address_votes.is_empty() || address_votes.get().period != current_period;
		let nb_nfts: usize = nft_nonces.len();

		require!(nb_nfts > 0, "At least an nft needs to be voted");
		require!(
			reset_address_votes || (address_votes.get().votes >= nb_nfts as u8),
			"Not enough votes left currently"
		);

		let last_nonce: u64 = self.blockchain().get_current_esdt_nft_nonce(
			&self.blockchain().get_sc_address(),
			&self.token_identifier().get()
		);

		let mut new_meme_votes: ManagedVec<MemeVotes> = ManagedVec::new();
		let mut temp_nonce: u64 = 0;
		let mut temp_votes: u32 = 0;

		for nonce in nft_nonces.into_iter() {
			require!(nonce > 0 && nonce >= temp_nonce, "Nonces need to be in ascending order");

			// If first element, save it to temp vars
			if temp_nonce == 0 {
				temp_nonce = nonce;
				temp_votes = 1;

				continue;
			}

			// If nonce is equal to previous temp none, increment votes
			if nonce == temp_nonce {
				temp_votes += 1;

				continue;
			}

			// A different nonce was encountered, save votes for previous temp nonce
			self.update_meme_votes(&current_period, &last_nonce, &mut new_meme_votes, &temp_nonce, &mut temp_votes);

			temp_nonce = nonce;
			temp_votes = 1;
		}

		// Update meme votes for last loop element
		self.update_meme_votes(&current_period, &last_nonce, &mut new_meme_votes, &temp_nonce, &mut temp_votes);

		// Require a max of 20 memes to be voted at a time because of static memory allocation
		require!(new_meme_votes.len() <= 20, "Only 20 memes can be voted at a time");

		address_votes.set(&AddressVotes {
			period: current_period,
			votes: (if reset_address_votes { VOTES_PER_ADDRESS_PER_PERIOD } else { address_votes.get().votes }) - (nb_nfts as u8),
		});

		self.alter_period_top_memes(&mut new_meme_votes, &current_period);
	}

	#[payable("*")]
	#[endpoint]
	fn upgrade_custom_attributes(
		&self,
		#[payment_token] nft_type: TokenIdentifier,
		#[payment_nonce] nonce: u64,
		#[payment_amount] nft_amount: BigUint,
	) {
		require!(nft_type == self.token_identifier().get(), "Nft is not of the correct type");
		require!(nft_amount == NFT_AMOUNT, "Nft amount should be 1");
		require!(!self.custom_attributes(nonce).is_empty(), "Nft can't be upgraded");

		self.update_nft_attributes(
			&self.blockchain().get_caller(),
			&nonce,
			b"nft upgraded"
		);

		self.custom_attributes(nonce).clear();
	}

	// private

	fn update_meme_votes(&self, current_period: &u64, last_nonce: &u64, new_meme_votes: &mut ManagedVec<MemeVotes>, temp_nonce: &u64, temp_votes: &mut u32) {
		require!(*temp_nonce <= *last_nonce, "Meme does not exist");

		// Update total votes before updating new_votes variable
		self.meme_votes_total(*temp_nonce).update(|value| *value += *temp_votes);

		let meme_votes = self.meme_votes(*temp_nonce, *current_period);

		if !meme_votes.is_empty() {
			*temp_votes += meme_votes.get();
		}

		meme_votes.set(*temp_votes);

		new_meme_votes.push(MemeVotes {
			nft_nonce: *temp_nonce,
			votes: *temp_votes,
		});
	}

	fn alter_period(&self) -> OptionalValue<AsyncCall> {
		let block_timestamp: u64 = self.blockchain().get_block_timestamp();
		let period: u64 = self.current_period();

		if block_timestamp > period && block_timestamp - period >= PERIOD_TIME {
			let new_period: u64 = period + PERIOD_TIME;

			self.periods().push(&new_period);

			let top_memes = self.period_top_memes(period).get();
			let mut top_memes_args: MultiValueEncoded<u64> = MultiValueEncoded::new();

			for meme in top_memes.iter() {
				top_memes_args.push(meme.nft_nonce);
			}

			return OptionalValue::Some(
				self.auction_proxy()
					.contract(self.auction_sc().get())
					.start_auction(new_period, top_memes_args)
					.async_call()
			);
		}

		OptionalValue::None
	}

	fn alter_period_top_memes(&self, new_meme_votes: &ManagedVec<MemeVotes>, current_period: &u64) {
		// 10 from top memes, 20 from max number of votes a transaction can have
		let mut result: ArrayVec<MemeVotes, 30> = ArrayVec::<_, 30>::new();

		let top_memes = self.period_top_memes(*current_period);

		if !top_memes.is_empty() {
			let mut sorted: ArrayVec<MemeVotes, 20> = ArrayVec::<_, 20>::new();

			for meme_vote in new_meme_votes.into_iter() {
				sorted.push(meme_vote);
			}

			let meme_votes: ArrayVec<MemeVotes, 10> = top_memes.get();

			for meme_vote in meme_votes {
				let nonce: u64 = meme_vote.nft_nonce;

				if sorted.binary_search_by(|probe| probe.nft_nonce.cmp(&nonce)).is_err() {
					result.push(meme_vote);
				}
			}
		}

		for meme_vote in new_meme_votes.iter() {
			result.push(meme_vote);
		}

		result.sort_unstable_by(|a, b| b.votes.cmp(&a.votes).then(b.nft_nonce.cmp(&a.nft_nonce)));

		let mut new_top_memes: ArrayVec<MemeVotes, 10> = ArrayVec::<_, 10>::new();

		let mut nb: u8 = 0;

		for item in result {
			new_top_memes.push(item);
			nb += 1;

			if nb == 10 {
				break;
			}
		}

		top_memes.set(&new_top_memes);
	}

	fn update_nft_attributes(&self, send_to: &ManagedAddress, nft_nonce: &u64, text: &[u8]) {
		let nft_token = &self.token_identifier().get();
		let amount = BigUint::from(NFT_AMOUNT);

		let own_address: ManagedAddress = self.blockchain().get_sc_address();
		let token_data: EsdtTokenData<Self::Api> = self.blockchain().get_esdt_token_data(&own_address, nft_token, *nft_nonce);
		let mut new_attributes = token_data.decode_attributes::<MemeAttributes<Self::Api>>();

		let custom_attributes = self.custom_attributes(*nft_nonce).get();

		new_attributes.category = custom_attributes.category;
		new_attributes.rarity = custom_attributes.rarity;

		self.send().nft_update_attributes(
			&self.token_identifier().get(),
			*nft_nonce,
			&new_attributes
		);

		self.send().direct(
			send_to,
			nft_token,
			*nft_nonce,
			&amount,
			text,
		);
	}

	#[proxy]
	fn auction_proxy(&self) -> auction_proxy::Proxy<Self::Api>;

	// views/storage

	#[view]
	fn current_period_len(&self) -> usize {
		return self.period_len(self.current_period());
	}

	#[view]
	fn current_period_meme(&self, index: usize) -> u64 {
		return self.period_meme(self.current_period(), index);
	}

	#[view]
	fn current_period_memes_latest(&self, page: usize) -> MultiValueEncoded<(u64, u32)> {
		return self.period_memes_latest(self.current_period(), page);
	}

	#[view]
	fn period_len(&self, period: u64) -> usize {
		return self.period_memes(period).len();
	}

	#[view]
	fn period_meme(&self, period: u64, index: usize) -> u64 {
		return self.period_memes(period).get(index);
	}

	#[view]
	fn period_memes_latest(&self, period: u64, page: usize) -> MultiValueEncoded<(u64, u32)> {
		let len = self.period_memes(period).len();

		if len <= page * PER_PAGE {
			return MultiValueEncoded::new();
		}

		let last_index = len - page * PER_PAGE;
		let first_index = if last_index > PER_PAGE { last_index - PER_PAGE + 1 } else { 1 };

		let mut result: MultiValueEncoded<(u64, u32)> = MultiValueEncoded::new();
		for index in (first_index..=last_index).rev() {
			let nonce: u64 = self.period_meme(period, index);
			let votes: u32 = self.meme_votes(nonce, period).get();

			result.push((nonce, votes));
		}

		return result;
	}

	#[view]
	fn current_period(&self) -> u64 {
		let len = self.periods().len();

		return self.periods().get(len);
	}

	#[view]
	fn meme_votes_all(&self, nonce: u64, period: u64) -> (u32, u32) {
		(
			self.meme_votes(nonce, period).get(),
			self.meme_votes_total(nonce).get()
		)
	}

	#[view]
	#[storage_mapper("addressLastMemeTime")]
	fn address_last_meme_time(&self, address: &ManagedAddress) -> SingleValueMapper<u64>;

	#[view]
	#[storage_mapper("periods")]
	fn periods(&self) -> VecMapper<u64>;

	#[view]
	#[storage_mapper("periodTopMemes")]
	fn period_top_memes(&self, period: u64) -> SingleValueMapper<ArrayVec<MemeVotes, 10>>;

	#[view]
	#[storage_mapper("addressVotes")]
	fn address_votes(&self, address: ManagedAddress) -> SingleValueMapper<AddressVotes>;

	// TODO: Remove this if data is indexed on microservice side?
	#[storage_mapper("periodMemes")]
	fn period_memes(&self, period: u64) -> VecMapper<u64>;

	#[view]
	#[storage_mapper("memeVotesTotal")]
	fn meme_votes_total(&self, nft_nonce: u64) -> SingleValueMapper<u32>;

	#[view]
	#[storage_mapper("memeVotes")]
	fn meme_votes(&self, nft_nonce: u64, period: u64) -> SingleValueMapper<u32>;
}
