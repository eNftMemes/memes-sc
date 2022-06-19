#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait StakingContract {
    #[init]
    fn init(&self) {}
}
