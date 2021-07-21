#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod meme;
use meme::*;

const PER_PAGE: usize = 10;
const PERIOD_TIME: u64 = 604800; // 1 week in seconds

#[elrond_wasm_derive::contract]
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
			// TODO: Calculate top 10 memes and save them
		}
	}

	#[endpoint]
	fn vote_memes(&self, nft_nonces: &[u64]) -> SCResult<()> {
		// let caller: Address = self.blockchain().get_caller();
		// require!(
		// 	TODO: Add condition,
		// 	"Not enough votes left currently"
		// );

		for nonce in nft_nonces {
			let current_votes: SingleValueMapper<Self::Storage, u32> = self.meme_votes(*nonce);

			let mut votes = 0;
			if !current_votes.is_empty() {
				votes = current_votes.get();
			}

			votes += 1;
			current_votes.set(&votes);
		}

		Ok(())
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
	fn meme_votes(&self, nft_nonce: u64) -> SingleValueMapper<Self::Storage, u32>;

	#[view]
	#[storage_mapper("periodTopMemes")]
	fn period_top_memes(&self, period: u64) -> VecMapper<Self::Storage, (Meme, u32)>;

	#[storage_mapper("creatorContract")]
	fn creator_contract(&self) -> SingleValueMapper<Self::Storage, Address>;

	// Always needed
	#[endpoint]
	fn claim(&self) -> SCResult<()> {
		let caller = self.blockchain().get_caller();
		require!(
			caller == self.blockchain().get_owner_address(),
			"only owner can claim"
		);
		self.send().direct_egld(&caller, &self.blockchain().get_sc_balance(), b"claiming success");
		Ok(())
	}
}
