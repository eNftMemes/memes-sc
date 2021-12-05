#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();

pub mod meme;
use meme::*;

use hashbrown::HashMap;

const PER_PAGE: usize = 10;
const PERIOD_TIME: u64 = 604800; // 1 week in seconds
const VOTES_PER_ADDRESS_PER_PERIOD: u8 = 20;

mod auction_proxy {
	elrond_wasm::imports!();

	#[elrond_wasm::proxy]
	pub trait Auction {
		#[endpoint]
		fn start_auction(&self, period: u64, #[var_args] nfts: VarArgs<u64>) -> SCResult<()>;
	}
}

#[elrond_wasm::contract]
pub trait MemesVoting {
	#[init]
	fn init(&self, creator_contract: &ManagedAddress, period: &u64) {
		self.creator_contract().set(creator_contract);

		if self.periods().len() == 0 {
			self.periods().push(period);
		}
	}

	#[endpoint]
	fn add_meme(&self, nonce: &u64) -> SCResult<OptionalResult<AsyncCall>> {
		let caller: ManagedAddress = self.blockchain().get_caller();
		require!(
			caller == self.creator_contract().get(),
			"Only creator contract can call this"
		);

		let result = self.alter_period();

		let current_period: u64 = self.current_period();

		self.period_memes(current_period).push(nonce);
		self.meme_votes(*nonce).insert(current_period, 0);

		Ok(result)
	}

	fn alter_period(&self) -> OptionalResult<AsyncCall> {
		let block_timestamp: u64 = self.blockchain().get_block_timestamp();
		let period: u64 = self.current_period();

		if block_timestamp > period && block_timestamp - period >= PERIOD_TIME {
			let new_period: u64 = period + PERIOD_TIME;

			self.periods().push(&new_period);

			let top_memes = self.period_top_memes(period).get();
			let mut top_memes_args: MultiArgVec<u64> = MultiArgVec::new();

			for meme in top_memes.iter() {
				top_memes_args.push(meme.nft_nonce);
			}

			return OptionalResult::Some(
				self.auction_proxy()
					.contract(self.auction_sc().get())
					.start_auction(new_period, top_memes_args)
					.async_call()
			);
		}

		OptionalResult::None
	}

	#[proxy]
	fn auction_proxy(&self) -> auction_proxy::Proxy<Self::Api>;

	#[only_owner]
	#[endpoint]
	fn set_auction_sc(&self, sc: &ManagedAddress) -> SCResult<()> {
		self.auction_sc().set(sc);

		Ok(())
	}

	#[endpoint]
	fn vote_memes(&self, #[var_args] nft_nonces: ManagedVarArgs<u64>) -> SCResult<OptionalResult<AsyncCall>> {
		let caller: ManagedAddress = self.blockchain().get_caller();

		let result = self.alter_period();

		let address_votes: SingleValueMapper<AddressVotes> = self.address_votes(caller);
		let current_period: u64 = self.current_period();
		let reset_address_votes = address_votes.is_empty() || address_votes.get().period != current_period;
		let nb_nfts: usize = nft_nonces.len();

		require!(
			reset_address_votes || (address_votes.get().votes >= nb_nfts as u8),
			"Not enough votes left currently"
		);

		let mut new_meme_votes: HashMap<u64, u32> = HashMap::new();
		for nonce in nft_nonces.into_iter() {
			let votes: &mut u32 = new_meme_votes.entry(nonce).or_insert(0);
			*votes += 1;
		}

		for (nonce, new_votes) in new_meme_votes.iter_mut() {
			require!(!self.meme_votes(*nonce).is_empty(), "Meme does not exist");

			let current_votes: u32 = self.meme_votes(*nonce)
				.entry(current_period)
				.and_modify(|value| *value += *new_votes)
				.or_insert(*new_votes)
				.get();

			*new_votes = current_votes;
		}

		address_votes.set(&AddressVotes {
			period: current_period,
			votes: (if reset_address_votes { VOTES_PER_ADDRESS_PER_PERIOD } else { address_votes.get().votes }) - (nb_nfts as u8),
		});

		self.alter_period_top_memes(&mut new_meme_votes);

		Ok(result)
	}

	fn alter_period_top_memes(&self, new_meme_votes: &mut HashMap<u64, u32>) {
		let current_period: u64 = self.current_period();
		let top_memes: SingleValueMapper<Vec<MemeVotes>> = self.period_top_memes(current_period);

		if !top_memes.is_empty() {
			let meme_votes: Vec<MemeVotes> = top_memes.get();
			for meme_vote in meme_votes {
				let nonce: u64 = meme_vote.nft_nonce;
				if !new_meme_votes.contains_key(&nonce) {
					new_meme_votes.insert(meme_vote.nft_nonce, meme_vote.votes);
				}
			}
		}

		let mut sorted: Vec<MemeVotes> = Vec::new();

		for (nft_nonce, votes) in new_meme_votes.iter() {
			sorted.push(MemeVotes {
				nft_nonce: *nft_nonce,
				votes: *votes,
			})
		}

		sorted.sort_unstable_by(|a, b| b.votes.cmp(&a.votes).then(b.nft_nonce.cmp(&a.nft_nonce)));

		// Remove memes if more than 10
		if sorted.len() > 10 {
			for index in (10..sorted.len()).rev() {
				sorted.remove(index);
			}
		}

		top_memes.set(&sorted);
	}

	#[view]
	fn current_period_len(&self) -> usize {
		return self.period_len(self.current_period());
	}

	#[view]
	fn current_period_meme(&self, index: usize) -> u64 {
		return self.period_meme(self.current_period(), index);
	}

	#[view]
	fn current_period_memes_latest(&self, page: usize) -> ManagedMultiResultVec<(u64, u32)> {
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
	fn period_memes_latest(&self, period: u64, page: usize) -> ManagedMultiResultVec<(u64, u32)> {
		let len = self.period_memes(period).len();

		if len <= page * PER_PAGE {
			return ManagedMultiResultVec::new();
		}

		let last_index = len - page * PER_PAGE;
		let first_index = if last_index > PER_PAGE { last_index - PER_PAGE + 1 } else { 1 };

		let mut result: ManagedMultiResultVec<(u64, u32)> = ManagedMultiResultVec::new();
		for index in (first_index..=last_index).rev() {
			let nonce: u64 = self.period_meme(period, index);
			let votes: u32 = self.meme_votes_period(nonce, &period);

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
	fn meme_votes_all(&self, nonce: u64) -> ManagedMultiResultVec<(u64, u32)> {
		let meme_votes: MapMapper<u64, u32> = self.meme_votes(nonce);

		if 1 > meme_votes.len() {
			return ManagedMultiResultVec::new();
		}

		let mut result: ManagedMultiResultVec<(u64, u32)> = ManagedMultiResultVec::new();
		for (key, value) in meme_votes.iter() {
			result.push((key, value));
		}

		return result;
	}

	#[view]
	fn meme_votes_period(&self, nonce: u64, period: &u64) -> u32 {
		self.meme_votes(nonce).get(period).unwrap_or_default()
	}

	#[view]
	#[storage_mapper("auctionSc")]
	fn auction_sc(&self) -> SingleValueMapper<ManagedAddress>;

	#[view]
	#[storage_mapper("periods")]
	fn periods(&self) -> VecMapper<u64>;

	#[view]
	#[storage_mapper("periodTopMemes")]
	fn period_top_memes(&self, period: u64) -> SingleValueMapper<Vec<MemeVotes>>;

	#[view]
	#[storage_mapper("addressVotes")]
	fn address_votes(&self, address: ManagedAddress) -> SingleValueMapper<AddressVotes>;

	#[view]
	#[storage_mapper("creatorContract")]
	fn creator_contract(&self) -> SingleValueMapper<ManagedAddress>;

	#[storage_mapper("periodMemes")]
	fn period_memes(&self, period: u64) -> VecMapper<u64>;

	#[storage_mapper("memeVotes")]
	fn meme_votes(&self, nft_nonce: u64) -> MapMapper<u64, u32>;
}
