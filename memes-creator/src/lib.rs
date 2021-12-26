#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();

const THROTTLE_MEME_TIME: u64 = 600; // 10 minutes in seconds
const NFT_AMOUNT: u32 = 1;

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
	fn init(&self) {
		let royalties: u16 = 500;
		self.nft_royalties().set(&royalties);
	}

	// TODO: Test this function with Mandos after it is supported to issue tokens
	#[endpoint]
	#[only_owner]
	#[payable("EGLD")]
	fn issue_token(
		&self,
		#[payment] issue_cost: BigUint,
		token_name: &ManagedBuffer,
		token_ticker: &ManagedBuffer,
	) -> SCResult<AsyncCall> {
		require!(self.token_identifier().is_empty(), "Token already issued");

		Ok(
			self.send()
				.esdt_system_sc_proxy()
				.issue_non_fungible(
					issue_cost,
					token_name,
					token_ticker,
					NonFungibleTokenProperties {
						can_freeze: true,
						can_wipe: true,
						can_pause: true,
						can_change_owner: false,
						can_upgrade: false,
						can_add_special_roles: true,
					},
				)
				.async_call()
				.with_callback(self.callbacks().init_callback())
		)
	}

	// TODO: Test this function with Mandos after it is supported to issue tokens
	#[endpoint]
	#[only_owner]
	fn set_local_roles(&self) -> SCResult<AsyncCall> {
		require!(!self.token_identifier().is_empty(), "Token not issued");

		Ok(self
			.send()
			.esdt_system_sc_proxy()
			.set_special_roles(
				&self.blockchain().get_sc_address(),
				&self.token_identifier().get(),
				(&[EsdtLocalRole::NftCreate][..]).into_iter().cloned(),
			)
			.async_call()
		)
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
			self.categories().contains_key(category),
			"Category does not exist"
		);

		self.address_last_meme_time(&address).set(&block_timestamp);

		Ok(self.create_meme_nft(&address, &name, url, *category))
	}

	#[only_owner]
	#[endpoint]
	fn modify_categories(&self, category: &u8, name: ManagedBuffer) -> SCResult<()> {
		if !self.categories().contains_key(category) {
			self.categories().insert(*category, name);
		} else {
			self.categories().remove(category);
		}

		Ok(())
	}

	fn create_meme_nft(&self, address: &ManagedAddress, name: &ManagedBuffer, url: ManagedBuffer, category: u8) -> AsyncCall {
		let amount: &BigUint = &BigUint::from(NFT_AMOUNT);
		let royalties: &BigUint = &BigUint::from(self.nft_royalties().get());

		let nft_token: &TokenIdentifier = &self.token_identifier().get();
		let hash: &ManagedBuffer = &ManagedBuffer::new();

		let mut urls = ManagedVec::new();
		urls.push(url);

		// TODO: Maybe use `tags` instead of `category`?
		let nonce: u64 = self.send().esdt_nft_create_as_caller(nft_token, &amount, name, royalties, hash, &{ category }, &urls);

		self.send().direct(address, nft_token, nonce, amount, &[]);

		return self.voting_proxy()
	        .contract(self.voting_sc().get())
			.add_meme(&nonce)
			.async_call();
	}

	#[callback]
	fn init_callback(&self, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
		match result {
			ManagedAsyncCallResult::Ok(token_id) => {
				self.token_identifier().set(&token_id);
			},
			ManagedAsyncCallResult::Err(_) => {
				let caller = self.blockchain().get_owner_address();
				let (returned_tokens, token_id) = self.call_value().payment_token_pair();
				if token_id.is_egld() && returned_tokens > 0 {
					self.send()
						.direct(&caller, &token_id, 0, &returned_tokens, &[]);
				}
			},
		}
	}

	#[proxy]
	fn voting_proxy(&self) -> voting_proxy::Proxy<Self::Api>;

	#[only_owner]
	#[endpoint]
	fn set_voting_sc(&self, sc: &ManagedAddress) -> SCResult<()> {
		self.voting_sc().set(sc);

		Ok(())
	}

	#[only_owner]
	#[endpoint]
	fn set_nft_royalties(&self, royalties: u16) -> SCResult<()> {
		require!(royalties > 100 && royalties < 2500, "Royalties can not be less than 1% and greater than 25%");

		self.nft_royalties().set(&royalties);

		Ok(())
	}

	#[view]
	fn categories_all(&self) -> ManagedMultiResultVec<(u8, ManagedBuffer)> {
		let categories_all: MapMapper<u8, ManagedBuffer> = self.categories();

		if 1 > categories_all.len() {
			return ManagedMultiResultVec::new();
		}

		let mut result: ManagedMultiResultVec<(u8, ManagedBuffer)> = ManagedMultiResultVec::new();
		for (key, value) in categories_all.iter() {
			result.push((key, value));
		}

		return result;
	}

	#[view]
	#[storage_mapper("addressLastMemeTime")]
	fn address_last_meme_time(&self, address: &ManagedAddress) -> SingleValueMapper<u64>;

	#[view]
	#[storage_mapper("votingSc")]
	fn voting_sc(&self) -> SingleValueMapper<ManagedAddress>;

	#[view]
	#[storage_mapper("tokenIdentifier")]
	fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

	#[view]
	#[storage_mapper("nftRoyalties")]
	fn nft_royalties(&self) -> SingleValueMapper<u16>;

	#[storage_mapper("categories")]
	fn categories(&self) -> MapMapper<u8, ManagedBuffer>;
}
