elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait OwnerModule {
    #[storage_mapper("periodTime")]
    fn period_time(&self) -> SingleValueMapper<u64>;
}
