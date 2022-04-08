elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::api::ED25519_SIGNATURE_BYTE_LEN;

// caller + url + some extra
// 32 + (4 + 74) + 16
const MAX_SIGNATURE_DATA_LEN: usize = 126;

pub type Signature<M> = ManagedByteArray<M, ED25519_SIGNATURE_BYTE_LEN>;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct CustomMemeAttributes<M: ManagedTypeApi> {
    pub category: ManagedBuffer<M>,
    pub rarity: u8,
}

#[elrond_wasm::module]
pub trait OwnerModule {
    // TODO: Test this function with Mandos after it is supported to issue tokens
    #[endpoint]
    #[only_owner]
    #[payable("EGLD")]
    fn issue_token(
        &self,
        #[payment] issue_cost: BigUint,
        token_name: &ManagedBuffer,
        token_ticker: &ManagedBuffer,
    ) {
        require!(self.token_identifier().is_empty(), "Token already issued");

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
            .call_and_exit()
    }

    // TODO: Test this function with Mandos after it is supported to issue tokens
    #[endpoint]
    #[only_owner]
    fn set_local_roles(&self) {
        require!(!self.token_identifier().is_empty(), "Token not issued");

        self
            .send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.token_identifier().get(),
                (&[EsdtLocalRole::NftCreate, EsdtLocalRole::NftUpdateAttributes][..]).iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[only_owner]
    #[endpoint]
    fn modify_categories(&self, category: ManagedBuffer) {
        if !self.categories().contains(&category) {
            self.categories().insert(category);
        } else {
            self.categories().remove(&category);
        }
    }

    #[only_owner]
    #[endpoint]
    fn set_nft_royalties(&self, royalties: u16) {
        require!(royalties > 100 && royalties < 2500, "Royalties can not be less than 1% and greater than 25%");

        self.nft_royalties().set(&royalties);
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

    #[only_owner]
    #[endpoint]
    fn set_auction_sc(&self, sc: &ManagedAddress) {
        self.auction_sc().set(sc);

        self
            .send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                sc,
                &self.token_identifier().get(),
                (&[EsdtLocalRole::NftUpdateAttributes][..]).iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[only_owner]
    #[endpoint]
    fn set_custom_attributes(&self, nonce: u64, category: ManagedBuffer, rarity: u8) {
        self.custom_attributes(nonce).set(
            &CustomMemeAttributes { category, rarity }
        );
    }

    #[only_owner]
    #[endpoint]
    fn set_signer(&self, new_signer: ManagedAddress) {
        self.signer().set(&new_signer);
    }

    fn verify_signature_create(
        &self,
        caller: &ManagedAddress,
        url: &ManagedBuffer,
        signature: &Signature<Self::Api>,
    ) {
        let mut data = ManagedBuffer::new();
        data.append(caller.as_managed_buffer());
        data.append(url);

        let signer: ManagedAddress = self.signer().get();
        let valid_signature = self.crypto().verify_ed25519_managed::<MAX_SIGNATURE_DATA_LEN>(
            signer.as_managed_byte_array(),
            &data,
            signature,
        );
        require!(valid_signature, "Invalid signature");
    }

    fn verify_signature_vote(
        &self,
        caller: &ManagedAddress,
        first_vote: &u64,
        nb_votes: &u64,
        signature: &Signature<Self::Api>,
    ) {
        let mut data = ManagedBuffer::new();
        data.append(caller.as_managed_buffer());
        data.append(&self.decimal_to_ascii(*first_vote));
        data.append(&self.decimal_to_ascii(*nb_votes));

        let signer: ManagedAddress = self.signer().get();
        let valid_signature = self.crypto().verify_ed25519_managed::<MAX_SIGNATURE_DATA_LEN>(
            signer.as_managed_byte_array(),
            &data,
            signature,
        );
        require!(valid_signature, "Invalid signature");
    }

    fn decimal_to_ascii(&self, mut number: u64) -> ManagedBuffer {
        const MAX_NUMBER_CHARACTERS: usize = 20;
        const ZERO_ASCII: u8 = b'0';

        let mut vec = ArrayVec::<u8, MAX_NUMBER_CHARACTERS>::new();
        loop {
            vec.push(ZERO_ASCII + (number % 10) as u8);
            number /= 10;

            if number == 0 {
                break;
            }
        }

        vec.reverse();
        vec.as_slice().into()
    }

    #[view]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view]
    #[storage_mapper("nftRoyalties")]
    fn nft_royalties(&self) -> SingleValueMapper<u16>;

    #[view]
    #[storage_mapper("categories")]
    fn categories(&self) -> SetMapper<ManagedBuffer>;

    #[view]
    #[storage_mapper("auctionSc")]
    fn auction_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[view]
    #[storage_mapper("customAttributes")]
    fn custom_attributes(&self, nonce: u64) -> SingleValueMapper<CustomMemeAttributes<Self::Api>>;

    #[storage_mapper("signer")]
    fn signer(&self) -> SingleValueMapper<ManagedAddress>;
}
