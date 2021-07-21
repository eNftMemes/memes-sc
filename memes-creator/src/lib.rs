#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const TRIPA_NFT: &[u8] = "TRIPA-bb7f5c".as_bytes();
const THROTTLE_MEME_TIME: u64 = 600; // 10 minutes in seconds
const NFT_AMOUNT: u32 = 1;
const NFT_ROYALTIES: u32 = 500;
const PER_PAGE: usize = 10;

mod voting_proxy {
	elrond_wasm::imports!();

	#[elrond_wasm_derive::proxy]
	pub trait Voting {
		#[endpoint]
		fn add_meme(&self, owner: Address, nft_nonce: u64, category: u8) -> SCResult<()>;
	}
}

#[elrond_wasm_derive::contract]
pub trait MemesCreator {
	#[init]
	fn init(&self) {}

	#[endpoint]
	fn create_meme(&self, name: BoxedBytes, url: BoxedBytes, category: &u8) -> SCResult<AsyncCall<Self::SendApi>> {
		let address: Address = self.blockchain().get_caller();
		let block_timestamp: u64 = self.blockchain().get_block_timestamp();
		let address_meme_time: SingleValueMapper<Self::Storage, u64> = self.address_last_meme_time(&address);

		require!(
			address_meme_time.is_empty()
				|| address_meme_time.get() < block_timestamp - THROTTLE_MEME_TIME,
			"Address already created a meme in the last 10 minutes"
		);
		require!(
			!self.categories(category).is_empty(),
			"Category does not exist"
		);

		self.address_last_meme_time(&address).set(&block_timestamp);

		// TODO: Check if async call works correctly
		Ok(self.create_meme_nft(&address, &name, &[url], *category))
	}

	#[endpoint]
	fn modify_categories(&self, category: &u8, name: &BoxedBytes) -> SCResult<()> {
		self.require_permissions()?;

		if self.categories(category).is_empty() {
			self.categories(category).set(name);
		} else {
			self.categories(category).clear();
		}

		Ok(())
	}

	fn create_meme_nft(&self, address: &Address, name: &BoxedBytes, urls: &[BoxedBytes], category: u8) -> AsyncCall<Self::SendApi> {
		let amount: &Self::BigUint = &Self::BigUint::from(NFT_AMOUNT);
		let royalties: &Self::BigUint = &Self::BigUint::from(NFT_ROYALTIES);

		let sc_address: Address = self.blockchain().get_sc_address();
		let nft_token: TokenIdentifier = TokenIdentifier::from(TRIPA_NFT);
		let hash: &BoxedBytes = &BoxedBytes::empty();

		self.send().esdt_nft_create(&nft_token, &amount, name, royalties, hash, &{ category }, urls);

		let nonce: u64 = self.blockchain().get_current_esdt_nft_nonce(&sc_address, &nft_token);
		// TODO: Change to self.send().direct(...) in next version
		self.send().direct_nft(address, &nft_token, nonce, amount, &[]);
		self.address_memes(address).push(&nonce);

		return self.voting_sc_add_meme(nonce, category);
	}

	fn voting_sc_add_meme(&self, nft_nonce: u64, _category: u8) -> AsyncCall<Self::SendApi> {
		let owner: Address = self.blockchain().get_caller();

		return self.voting_proxy(self.voting_sc().get())
			.add_meme(owner, nft_nonce, _category)
			.async_call();
	}

	#[proxy]
	fn voting_proxy(&self, to: Address) -> voting_proxy::Proxy<Self::SendApi>;

	#[endpoint]
	fn set_voting_sc(&self, sc: &Address) -> SCResult<()> {
		self.require_permissions()?;

		self.voting_sc().set(sc);

		Ok(())
	}

	fn require_permissions(&self) -> SCResult<()> {
		let caller = self.blockchain().get_caller();
		require!(
			caller == self.blockchain().get_owner_address(),
			"only owner can call this"
		);
		Ok(())
	}

	#[view]
	fn address_memes_len(&self, address: &Address) -> usize {
		return self.address_memes(address).len();
	}

	#[view]
	fn address_memes_latest(&self, address: &Address, page: usize) -> MultiResultVec<u64> {
		let len = self.address_memes(address).len();

		if len <= page * PER_PAGE {
			return MultiResultVec::new();
		}

		let last_index = len - page * PER_PAGE;
		let first_index = if last_index > PER_PAGE { last_index - PER_PAGE + 1 } else { 1 };

		let mut result: Vec<u64> = Vec::with_capacity(last_index - first_index + 1);
		for index in (first_index..=last_index).rev() {
			result.push(self.address_memes(address).get(index));
		}

		return MultiResultVec::<u64>::from(result);
	}

	#[view]
	#[storage_mapper("addressMemes")]
	fn address_memes(&self, address: &Address) -> VecMapper<Self::Storage, u64>;

	#[view]
	#[storage_mapper("addressLastMemeTime")]
	fn address_last_meme_time(&self, address: &Address) -> SingleValueMapper<Self::Storage, u64>;

	#[view]
	#[storage_mapper("votingSc")]
	fn voting_sc(&self) -> SingleValueMapper<Self::Storage, Address>;

	#[view]
	#[storage_mapper("categories")]
	fn categories(&self, category: &u8) -> SingleValueMapper<Self::Storage, BoxedBytes>;

	// Always needed
	#[endpoint]
	fn claim(&self) -> SCResult<()> {
		self.require_permissions()?;

		let caller = self.blockchain().get_caller();
		self.send().direct_egld(&caller, &self.blockchain().get_sc_balance(), b"claiming success");
		Ok(())
	}
}
