elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, TypeAbi)]
pub struct Auction<M: ManagedTypeApi> {
    pub min_bid: BigUint<M>,
    pub current_bid: BigUint<M>,
    pub current_winner: ManagedAddress<M>,
    pub bid_cut_percentage: u16,
    pub owner_payed: bool,
}
