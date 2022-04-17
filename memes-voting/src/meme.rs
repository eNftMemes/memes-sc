elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct AddressVotes {
    pub period: u64,
    pub votes: u8,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem)]
pub struct MemeVotes {
    pub nft_nonce: u64,
    pub votes: u32,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct MemeAttributes<M: ManagedTypeApi> {
    pub period: u64,
    pub category: ManagedBuffer<M>,
    pub creator: ManagedAddress<M>
}
