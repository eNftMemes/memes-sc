use crate::common_structs::Nonce;
elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait OwnerModule {
    #[only_owner]
    #[endpoint]
    fn set_minimum_lock_epochs(&self, epochs: u8) {
        self.minimum_lock_epochs().set(&epochs);
    }

    #[view]
    #[storage_mapper("minimumLockEpochs")]
    fn minimum_lock_epochs(&self) -> SingleValueMapper<u8>;

    // From Farm contract Config Module

    #[view(getFarmTokenSupply)]
    #[storage_mapper("farm_token_supply")]
    fn farm_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[view(getDivisionSafetyConstant)]
    #[storage_mapper("division_safety_constant")]
    fn division_safety_constant(&self) -> SingleValueMapper<BigUint>;

    #[view(getLastRewardBlockNonce)]
    #[storage_mapper("last_reward_block_nonce")]
    fn last_reward_block_nonce(&self) -> SingleValueMapper<Nonce>;
}
