elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct Meme {
    pub owner: Address,
    pub nft_nonce: u64,
    pub created_at: u64,
    pub category: u8,
}
