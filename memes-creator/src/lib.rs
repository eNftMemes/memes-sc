#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const TRIPA_NFT: &[u8] = "TRIPA-bb7f5c".as_bytes();

#[elrond_wasm_derive::contract]
pub trait MemesCreator {
	#[init]
	fn init(&self) {}

	#[endpoint]
	fn create_meme(&self, name: BoxedBytes, url: BoxedBytes) -> SCResult<()> {
		let amount: &Self::BigUint = &Self::BigUint::from(1u64);
		let royalties: &Self::BigUint = &Self::BigUint::from(500u64);
		let address: Address = self.blockchain().get_caller();
		let sc_address: Address = self.blockchain().get_sc_address();
		let nft_token: TokenIdentifier = TokenIdentifier::from(TRIPA_NFT);
		let hash: &BoxedBytes = &BoxedBytes::empty();

		self.send().esdt_nft_create(&nft_token, &amount, &name, royalties, hash, &{}, &[url]);

		let nonce: u64 = self.blockchain().get_current_esdt_nft_nonce(&sc_address, &nft_token);
		self.send().direct_nft(&address, &nft_token, nonce, amount, &[]);

		Ok(())
	}
}
