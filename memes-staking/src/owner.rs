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

    // From Farm contract Config Module

    #[view(getStakeModifierTotal)]
    #[storage_mapper("stake_modifier_total")]
    fn stake_modifier_total(&self) -> SingleValueMapper<BigUint>;

    #[view(getDivisionSafetyConstant)]
    #[storage_mapper("division_safety_constant")]
    fn division_safety_constant(&self) -> SingleValueMapper<BigUint>;

    #[view(getLastRewardBlockNonce)]
    #[storage_mapper("last_reward_block_nonce")]
    fn last_reward_block_nonce(&self) -> SingleValueMapper<u64>;
}
