#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();

pub mod meme;
use meme::*;

const PER_PAGE: usize = 10;
const PERIOD_TIME: u64 = 604800; // 1 week in seconds
const VOTES_PER_ADDRESS_PER_PERIOD: u8 = 20;

#[elrond_wasm::contract]
pub trait MemesVoting {
	#[init]
	fn init(&self, creator_contract: &Address, period: &u64) {
		self.creator_contract().set(creator_contract);
		self.periods().push(period);
	}

	#[endpoint]
	fn add_meme(&self, owner: Address, nft_nonce: u64, category: u8) -> SCResult<()> {
		let caller: Address = self.blockchain().get_caller();
		require!(
			caller == self.creator_contract().get(),
			"Only creator contract can call this"
		);

		self.alter_period();

		let meme = Meme {
			owner,
			nft_nonce,
			created_at: self.blockchain().get_block_timestamp(),
			category,
		};
		self.period_memes(self.current_period()).push(&meme);

		Ok(())
	}

	fn alter_period(&self) {
		let block_timestamp: u64 = self.blockchain().get_block_timestamp();
		let period: u64 = self.current_period();

		if block_timestamp > period && block_timestamp - period >= PERIOD_TIME {
			self.periods().push(&block_timestamp);
			// TODO: Send Top memes to other contract
		}
	}

	#[endpoint]
	fn vote_memes(&self, nft_nonces: &[u64]) -> SCResult<()> {
		let caller: Address = self.blockchain().get_caller();

		self.alter_period();

		let address_votes: SingleValueMapper<Self::Storage, AddressVotes> = self.address_votes(caller);
		let current_period: u64 = self.current_period();
		let nb_nfts: usize = nft_nonces.len();
		let reset_address_votes = address_votes.is_empty() || address_votes.get().period != self.current_period();

		require!(
			reset_address_votes || (address_votes.get().votes >= nb_nfts as u8),
			"Not enough votes left currently"
		);

		let mut meme_votes: Vec<MemeVotes> = Vec::new();
		for nonce in nft_nonces {
			let current_votes: SingleValueMapper<Self::Storage, u32> = self.meme_votes(*nonce, current_period);

			let mut votes = 0;
			if !current_votes.is_empty() {
				votes = current_votes.get();
			}

			votes += 1;
			current_votes.set(&votes);
			meme_votes.push(MemeVotes {
				nft_nonce: *nonce,
				votes,
			});
		}

		address_votes.set(&AddressVotes {
			period: current_period,
			votes: (if reset_address_votes { VOTES_PER_ADDRESS_PER_PERIOD } else { address_votes.get().votes }) - (nb_nfts as u8),
		});

		self.alter_period_top_memes(&mut meme_votes);

		Ok(())
	}

	fn alter_period_top_memes(&self, meme_votes: &mut Vec<MemeVotes>) {
		let current_period: u64 = self.current_period();
		let top_memes: SingleValueMapper<Self::Storage, Vec<MemeVotes>> = self.period_top_memes(current_period);

		if !top_memes.is_empty() {
			meme_votes.append(&mut top_memes.get());
		}

		// Sort first by nonce and then by votes, higher nonce with higher votes first
		meme_votes.sort_unstable_by(|a, b| b.nft_nonce.cmp(&a.nft_nonce).then_with(|| b.votes.cmp(&a.votes)));
		// Remove duplicates
		meme_votes.dedup();
		// Sort by votes
		meme_votes.sort_unstable_by(|a, b| b.votes.cmp(&a.votes));

		// Remove memes if more than 10
		if meme_votes.len() > 10 {
			for index in (10..meme_votes.len()).rev() {
				meme_votes.remove(index);
			}
		}

		top_memes.set(&meme_votes);
	}

	#[view]
	fn current_period_len(&self) -> usize {
		return self.period_len(self.current_period());
	}

	#[view]
	fn current_period_meme(&self, index: usize) -> Meme {
		return self.period_meme(self.current_period(), index);
	}

	#[view]
	fn current_period_memes_latest(&self, page: usize) -> MultiResultVec<Meme> {
		return self.period_memes_latest(self.current_period(), page);
	}

	#[view]
	fn period_len(&self, period: u64) -> usize {
		return self.period_memes(period).len();
	}

	#[view]
	fn period_meme(&self, period: u64, index: usize) -> Meme {
		return self.period_memes(period).get(index);
	}

	#[view]
	fn period_memes_latest(&self, period: u64, page: usize) -> MultiResultVec<Meme> {
		let len = self.period_memes(period).len();

		if len <= page * PER_PAGE {
			return MultiResultVec::new();
		}

		let last_index = len - page * PER_PAGE;
		let first_index = if last_index > PER_PAGE { last_index - PER_PAGE + 1 } else { 1 };

		let mut result: Vec<Meme> = Vec::with_capacity(last_index - first_index + 1);
		for index in (first_index..=last_index).rev() {
			result.push(self.period_memes(period).get(index));
		}

		return MultiResultVec::<Meme>::from(result);
	}

	#[view]
	fn current_period(&self) -> u64 {
		let len = self.periods().len();

		return self.periods().get(len);
	}

	#[view]
	#[storage_mapper("periodMemes")]
	fn period_memes(&self, period: u64) -> VecMapper<Self::Storage, Meme>;

	#[view]
	#[storage_mapper("periods")]
	fn periods(&self) -> VecMapper<Self::Storage, u64>;

	#[view]
	#[storage_mapper("memeVotes")]
	fn meme_votes(&self, nft_nonce: u64, period: u64) -> SingleValueMapper<Self::Storage, u32>;

	#[view]
	#[storage_mapper("periodTopMemes")]
	fn period_top_memes(&self, period: u64) -> SingleValueMapper<Self::Storage, Vec<MemeVotes>>;

	#[view]
	#[storage_mapper("addressVotes")]
	fn address_votes(&self, address: Address) -> SingleValueMapper<Self::Storage, AddressVotes>;

	#[storage_mapper("creatorContract")]
	fn creator_contract(&self) -> SingleValueMapper<Self::Storage, Address>;

	// Always needed
	#[endpoint]
	#[only_owner]
	fn claim(&self) -> SCResult<()> {
		let caller = self.blockchain().get_caller();
		self.send().direct_egld(&caller, &self.blockchain().get_sc_balance(&TokenIdentifier::egld(), 0), b"claiming success");
		Ok(())
	}
}
