elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait OwnerModule {
    #[only_owner]
    #[endpoint]
    fn set_bid_cut_percentage(&self, bid_cut: u16) {
        require!(bid_cut > 100 && bid_cut < 2500, "Bid cut percentage can not be less than 1% and greater than 25%");

        self.bid_cut_percentage().set(&bid_cut);
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
