elrond_wasm::imports!();
elrond_wasm::derive_imports!();

// const ESDT_ROLE_NFT_UPDATE_ATTRIBUTES: &[u8] = b"ESDTRoleNFTUpdateAttributes";
//
// // TODO: ^ Use EsdtLocalRole when it is created for ESDTNFTUpdateAttributes
// let esdt_system_sc_address = self.send().esdt_system_sc_proxy().esdt_system_sc_address();
// let mut contract_call: ContractCall<Self::Api, ()> = ContractCall::new(
// esdt_system_sc_address,
// ManagedBuffer::new_from_bytes(b"setSpecialRole"),
// );
//
// contract_call.push_endpoint_arg(&self.token_identifier_top().get());
// contract_call.push_endpoint_arg(&self.blockchain().get_sc_address());
// contract_call.push_argument_raw_bytes(EsdtLocalRole::NftCreate.as_role_name());
// contract_call.push_argument_raw_bytes(ESDT_ROLE_NFT_UPDATE_ATTRIBUTES);
//
// Ok(contract_call.async_call())

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

    #[view]
    #[storage_mapper("bidCutPercentage")]
    fn bid_cut_percentage(&self) -> SingleValueMapper<u16>;

    #[view]
    #[storage_mapper("minBidStart")]
    fn min_bid_start(&self) -> SingleValueMapper<BigUint>;
}
