elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const ESDT_ROLE_NFT_UPDATE_ATTRIBUTES: &[u8] = b"ESDTRoleNFTUpdateAttributes";

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

        // Ok(self
        //     .send()
        //     .esdt_system_sc_proxy()
        //     .set_special_roles(
        //         &self.blockchain().get_sc_address(),
        //         &self.token_identifier().get(),
        //         (&[EsdtLocalRole::NftCreate][..]).iter().cloned(),
        //     )
        //     .async_call()
        // )

        let sc = &self.blockchain().get_sc_address();

        // TODO: ^ Use EsdtLocalRole when it is created for ESDTNFTUpdateAttributes
        let esdt_system_sc_address = self.send().esdt_system_sc_proxy().esdt_system_sc_address();
        let mut contract_call: ContractCall<Self::Api, ()> = ContractCall::new(
            esdt_system_sc_address,
            ManagedBuffer::new_from_bytes(b"setSpecialRole"),
        );

        contract_call.push_endpoint_arg(&self.token_identifier().get());
        contract_call.push_endpoint_arg(sc);
        contract_call.push_argument_raw_bytes(EsdtLocalRole::NftCreate.as_role_name());
        contract_call.push_argument_raw_bytes(ESDT_ROLE_NFT_UPDATE_ATTRIBUTES);

        Ok(contract_call.async_call())
    }

    #[only_owner]
    #[endpoint]
    fn modify_categories(&self, category: ManagedBuffer) -> SCResult<()> {
        if !self.categories().contains(&category) {
            self.categories().insert(category);
        } else {
            self.categories().remove(&category);
        }

        Ok(())
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
    fn set_auction_sc(&self, sc: &ManagedAddress) -> SCResult<AsyncCall> {
        self.auction_sc().set(sc);

        // TODO: ^ Use EsdtLocalRole when it is created for ESDTNFTUpdateAttributes
        let esdt_system_sc_address = self.send().esdt_system_sc_proxy().esdt_system_sc_address();
        let mut contract_call: ContractCall<Self::Api, ()> = ContractCall::new(
            esdt_system_sc_address,
            ManagedBuffer::new_from_bytes(b"setSpecialRole"),
        );

        contract_call.push_endpoint_arg(&self.token_identifier().get());
        contract_call.push_endpoint_arg(sc);
        contract_call.push_argument_raw_bytes(ESDT_ROLE_NFT_UPDATE_ATTRIBUTES);

        Ok(contract_call.async_call())
    }

    #[only_owner]
    #[endpoint]
    fn set_custom_attributes(&self, nonce: u64, category: ManagedBuffer, rarity: u8) {
        self.custom_attributes(nonce).set(
            &CustomMemeAttributes { category, rarity }
        );
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
}
