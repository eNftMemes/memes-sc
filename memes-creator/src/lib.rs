#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const TRIPA_NFT: &[u8] = "TRIPA-bb7f5c".as_bytes();
const THROTTLE_MEME_TIME: u64 = 600; // 10 minutes in seconds

#[elrond_wasm_derive::contract]
pub trait MemesCreator {
	#[init]
	fn init(&self) {}

	#[endpoint]
	fn create_meme(&self, name: BoxedBytes, url: BoxedBytes) -> SCResult<()> {
		let address: Address = self.blockchain().get_caller();
		let block_timestamp: u64 = self.blockchain().get_block_timestamp();
		let address_meme_time: SingleValueMapper<Self::Storage, u64> = self.address_last_meme_time(&address);

		require!(
			address_meme_time.is_empty()
				|| address_meme_time.get() < block_timestamp - THROTTLE_MEME_TIME,
			"Address already created a meme in the last 10 minutes"
		);

		self.create_meme_nft(&address, &name, &[url]);

		self.address_last_meme_time(&address).set(&block_timestamp);

		Ok(())
	}

	fn create_meme_nft(&self, address: &Address, name: &BoxedBytes, urls: &[BoxedBytes]) {
		let amount: &Self::BigUint = &Self::BigUint::from(1u64);
		let royalties: &Self::BigUint = &Self::BigUint::from(500u64);

		let sc_address: Address = self.blockchain().get_sc_address();
		let nft_token: TokenIdentifier = TokenIdentifier::from(TRIPA_NFT);
		let hash: &BoxedBytes = &BoxedBytes::empty();

		self.send().esdt_nft_create(&nft_token, &amount, name, royalties, hash, &{}, urls);

		let nonce: u64 = self.blockchain().get_current_esdt_nft_nonce(&sc_address, &nft_token);
		self.send().direct_nft(&address, &nft_token, nonce, amount, &[]);
	}

	#[view]
	#[storage_mapper("addressLastMemeTime")]
	fn address_last_meme_time(&self, address: &Address) -> SingleValueMapper<Self::Storage, u64>;
}
