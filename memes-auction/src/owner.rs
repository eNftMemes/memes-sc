elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const ESDT_ROLE_NFT_UPDATE_ATTRIBUTES: &[u8] = b"ESDTRoleNFTUpdateAttributes";

#[elrond_wasm::module]
pub trait OwnerModule {
    #[only_owner]
    #[endpoint]
    fn set_bid_cut_percentage(&self, bid_cut: u16) -> SCResult<()> {
        require!(bid_cut > 100 && bid_cut < 2500, "Bid cut percentage can not be less than 1% and greater than 25%");

        self.bid_cut_percentage().set(&bid_cut);

        Ok(())
    }

    #[only_owner]
    #[endpoint]
    fn set_min_bid_start(&self, min_bid_start: &BigUint) -> SCResult<()> {
        self.min_bid_start().set(min_bid_start);

        Ok(())
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
        require!(self.token_identifier_top().is_empty(), "Token already issued");

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
        require!(!self.token_identifier_top().is_empty(), "Token not issued");

        // Ok(self
        //     .send()
        //     .esdt_system_sc_proxy()
        //     .set_special_roles(
        //         &self.blockchain().get_sc_address(),
        //         &self.token_identifier_top().get(),
        //         (&[EsdtLocalRole::NftCreate, EsdtLocalRole::NftUpdateAttributes][..]).iter().cloned(),
        //     )
        //     .async_call()
        // )

        // TODO: ^ Use EsdtLocalRole when it is created for ESDTNFTUpdateAttributes
        let esdt_system_sc_address = self.send().esdt_system_sc_proxy().esdt_system_sc_address();
        let mut contract_call: ContractCall<Self::Api, ()> = ContractCall::new(
            esdt_system_sc_address,
            ManagedBuffer::new_from_bytes(b"setSpecialRole"),
        );

        contract_call.push_endpoint_arg(&self.token_identifier_top().get());
        contract_call.push_endpoint_arg(&self.blockchain().get_sc_address());
        contract_call.push_argument_raw_bytes(EsdtLocalRole::NftCreate.as_role_name());
        contract_call.push_argument_raw_bytes(ESDT_ROLE_NFT_UPDATE_ATTRIBUTES);

        Ok(contract_call.async_call())
    }

    #[callback]
    fn init_callback(&self, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.token_identifier_top().set(&token_id);
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

    #[view]
    #[storage_mapper("bidCutPercentage")]
    fn bid_cut_percentage(&self) -> SingleValueMapper<u16>;

    #[view]
    #[storage_mapper("minBidStart")]
    fn min_bid_start(&self) -> SingleValueMapper<BigUint>;

    #[view]
    #[storage_mapper("tokenIdentifierTop")]
    fn token_identifier_top(&self) -> SingleValueMapper<TokenIdentifier>;
}
