elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait OwnerModule {
    #[only_owner]
    #[endpoint]
    fn set_minimum_lock_blocks(&self, blocks: u64) {
        self.minimum_lock_blocks().set(&blocks);
    }

    #[view]
    #[storage_mapper("minimumLockBlocks")]
    fn minimum_lock_blocks(&self) -> SingleValueMapper<u64>;
}
