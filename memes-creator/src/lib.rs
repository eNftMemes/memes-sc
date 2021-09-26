#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();

const THROTTLE_MEME_TIME: u64 = 600; // 10 minutes in seconds
const NFT_AMOUNT: u32 = 1;
// TODO: Make royalties configurable; also in voting?
const NFT_ROYALTIES: u32 = 500;
const PER_PAGE: usize = 10;

mod voting_proxy {
	elrond_wasm::imports!();

	#[elrond_wasm::proxy]
	pub trait Voting {
		#[endpoint]
		fn add_meme(&self, nonce: &u64) -> SCResult<()>;
	}
}

#[elrond_wasm::contract]
pub trait MemesCreator {
	#[init]
	fn init(&self, token_identifier: TokenIdentifier) -> SCResult<()> {
		// TODO: Maybe issue NFT here directly?
		require!(
            token_identifier.is_valid_esdt_identifier(),
            "Invalid token provided"
        );
		self.token_identifier().set(&token_identifier);

		Ok(())
	}

	#[endpoint]
	fn create_meme(&self, name: ManagedBuffer, url: ManagedBuffer, category: &u8) -> SCResult<AsyncCall> {
		let address: ManagedAddress = self.blockchain().get_caller();
		let block_timestamp: u64 = self.blockchain().get_block_timestamp();
		let address_meme_time: SingleValueMapper<u64> = self.address_last_meme_time(&address);

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

		Ok(self.create_meme_nft(&address, &name, url, *category))
	}

	#[only_owner]
	#[endpoint]
	fn modify_categories(&self, category: &u8, name: &ManagedBuffer) -> SCResult<()> {
		if self.categories(category).is_empty() {
			self.categories(category).set(name);
		} else {
			self.categories(category).clear();
		}

		Ok(())
	}

	fn create_meme_nft(&self, address: &ManagedAddress, name: &ManagedBuffer, url: ManagedBuffer, category: u8) -> AsyncCall {
		let amount: &BigUint = &self.types().big_uint_from(NFT_AMOUNT);
		let royalties: &BigUint = &BigUint::from(NFT_ROYALTIES);

		// let sc_address: ManagedAddress = self.blockchain().get_sc_address();
		let nft_token: &TokenIdentifier = &self.token_identifier().get();
		let hash: &ManagedBuffer = &self.types().managed_buffer_empty();

		let mut urls = ManagedVec::new_empty(self.type_manager());
		urls.push(url);

		let nonce: u64 = self.send().esdt_nft_create(nft_token, &amount, name, royalties, hash, &{ category }, &urls);

		// let nonce: u64 = self.blockchain().get_current_esdt_nft_nonce(&sc_address.to_address(), nft_token);
		self.send().direct(address, nft_token, nonce, amount, &[]);
		self.address_memes(address).push(&nonce);
		self.meme_creator(&nonce).set(address);

		let nonce: u64 = 1;

		return self.voting_proxy()
	        .contract(self.voting_sc().get())
			.add_meme(&nonce)
			.async_call();
	}

	#[proxy]
	fn voting_proxy(&self) -> voting_proxy::Proxy<Self::Api>;

	#[only_owner]
	#[endpoint]
	fn set_voting_sc(&self, sc: &ManagedAddress) -> SCResult<()> {
		self.voting_sc().set(sc);

		Ok(())
	}

	#[view]
	fn address_memes_len(&self, address: &ManagedAddress) -> usize {
		return self.address_memes(address).len();
	}

	#[view]
	fn address_memes_latest(&self, address: &ManagedAddress, page: usize) -> MultiResultVec<u64> {
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
	fn address_memes(&self, address: &ManagedAddress) -> VecMapper<u64>;

	#[view]
	#[storage_mapper("memeCreator")]
	fn meme_creator(&self, nonce: &u64) -> SingleValueMapper<ManagedAddress>;

	#[view]
	#[storage_mapper("addressLastMemeTime")]
	fn address_last_meme_time(&self, address: &ManagedAddress) -> SingleValueMapper<u64>;

	#[view]
	#[storage_mapper("votingSc")]
	fn voting_sc(&self) -> SingleValueMapper<ManagedAddress>;

	#[storage_mapper("tokenIdentifier")]
	fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

	// TODO: Maybe use SafeMapStorageMapper/MapMapper or similar?
	#[view]
	#[storage_mapper("categories")]
	fn categories(&self, category: &u8) -> SingleValueMapper<ManagedBuffer>;

	// Always needed
	#[endpoint]
	#[only_owner]
	fn claim(&self) -> SCResult<()> {
		let caller = self.blockchain().get_caller();
		self.send().direct_egld(&caller, &self.blockchain().get_sc_balance(&self.types().token_identifier_egld(), 0), b"claiming success");
		Ok(())
	}
}
