elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct TopMemeAttributes<M: ManagedTypeApi> {
    pub rarity: u8,
    pub original_nonce: u64,
    pub period: u64,
    pub category: ManagedBuffer<M>,
    pub creator: ManagedAddress<M>,
}
