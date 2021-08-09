elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct AddressVotes {
    pub period: u64,
    pub votes: u8,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct MemeVotes {
    pub nft_nonce: u64,
    pub votes: u32,
}
