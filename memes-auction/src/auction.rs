elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct Auction<M: ManagedTypeApi> {
    pub min_bid: BigUint<M>,
    pub current_bid: BigUint<M>,
    pub current_winner: ManagedAddress<M>,
    pub bid_cut_percentage: u16,

    pub original_owner: ManagedAddress<M>,
    pub ended: bool,

    pub top_nonce: u64,
}

#[derive(TopEncode, TopDecode, NestedEncode, TypeAbi)]
pub struct FullAuction<M: ManagedTypeApi> {
    pub nonce: u64,
    pub auction: Auction<M>,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct MemeAttributes<M: ManagedTypeApi> {
    pub period: u64,
    pub category: ManagedBuffer<M>,
    pub creator: ManagedAddress<M>,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct TopMemeAttributes<M: ManagedTypeApi> {
    pub rarity: u8,
    pub original_nonce: u64,
    pub period: u64,
    pub category: ManagedBuffer<M>,
    pub creator: ManagedAddress<M>,
}
