elrond_wasm::imports!();
elrond_wasm::derive_imports!();

// TODO: Maybe remove some fields from this and instead rely on Elrond REST API?
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct Meme {
    pub owner: Address,
    pub nft_nonce: u64,
    pub created_at: u64,
    pub category: u8,
}

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

impl PartialEq for MemeVotes {
    fn eq(&self, other: &Self) -> bool {
        self.nft_nonce == other.nft_nonce
    }
}
