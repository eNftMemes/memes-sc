use elrond_wasm::api::ED25519_SIGNATURE_BYTE_LEN;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

// caller + url + some extra
// 32 + (4 + 74) + 16
const MAX_SIGNATURE_DATA_LEN: usize = 126;

pub type Signature<M> = ManagedByteArray<M, ED25519_SIGNATURE_BYTE_LEN>;

#[elrond_wasm::module]
pub trait OwnerModule {
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
                (&[EsdtLocalRole::NftCreate][..]).iter().cloned(),
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
    fn set_period_time(&self, period_time: &u64) {
        self.period_time().set(period_time);
    }

    #[callback]
    fn init_callback(&self, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.token_identifier().set(&token_id);
            }
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_owner_address();
                let returned = self.call_value().egld_or_single_esdt();
                if returned.token_identifier.is_egld() && returned.amount > 0 {
                    self.send()
                        .direct(&caller, &returned.token_identifier, 0, &returned.amount);
                }
            }
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
                (&[EsdtLocalRole::NftBurn][..]).iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[only_owner]
    #[endpoint]
    fn set_staking_sc(&self, sc: &ManagedAddress) {
        self.staking_sc().set(sc);
    }

    #[only_owner]
    #[endpoint]
    fn set_signer(&self, new_signer: ManagedAddress) {
        self.signer().set(&new_signer);
    }

    #[only_owner]
    #[endpoint]
    fn claim_royalties(&self) {
        let caller = self.blockchain().get_caller();

        self.send().direct_egld(
            &caller,
            &self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0)
        );
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
        // TODO: Update to new version after release to mainnet
        // let valid_signature = self.crypto().verify_ed25519(
        //     signer.as_managed_buffer(),
        //     &data,
        //     signature.as_managed_buffer(),
        // );
        let valid_signature = self.crypto().verify_ed25519_legacy_managed::<MAX_SIGNATURE_DATA_LEN>(
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
        // TODO: Update to new version after release to mainnet
        // let valid_signature = self.crypto().verify_ed25519(
        //     signer.as_managed_buffer(),
        //     &data,
        //     signature.as_managed_buffer(),
        // );
        let valid_signature = self.crypto().verify_ed25519_legacy_managed::<MAX_SIGNATURE_DATA_LEN>(
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
    #[storage_mapper("categories")]
    fn categories(&self) -> SetMapper<ManagedBuffer>;

    #[view]
    #[storage_mapper("auctionSc")]
    fn auction_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[view]
    #[storage_mapper("stakingSc")]
    fn staking_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("signer")]
    fn signer(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("periodTime")]
    fn period_time(&self) -> SingleValueMapper<u64>;
}
