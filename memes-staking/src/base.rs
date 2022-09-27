elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const TOP_RARITY: u8 = 10;

// RARE (1) -> 1x multiplier
// RARE (2) -> 1.1x multiplier
// RARE (3) -> 1.2x multiplier
// RARE (9) -> 1.8x multiplier
// RARE (10) -> 2x multiplier
// SUPER RARE (> 10) -> 2.25x multiplier
const BASE_STAKE_MODIFIER: u8 = 100; // 1x
const INCREMENT_STAKE_MODIFIER: u8 = 10; // 0.1x
const TOP_RARITY_STAKE_MODIFIER: u8 = 200; // 2x
const SUPER_RARE_STAKE_MODIFIER: u8 = 225; // 2.25x

// RARE (1) -> 20 referrers
// RARE (2) -> 25 referrers
// RARE (3) -> 30 referrers
// RARE (9) -> 60 referrers
// RARE (10) -> 75 referrers
// SUPER RARE (> 10) -> 100 referrers
const BASE_REFERER_PERSONS: u8 = 20;
const INCREMENT_REFERER_PERSONS: u8 = 5;
const TOP_RARITY_REFERER_PERSONS: u8 = 75;
const SUPER_RARE_REFERER_PERSONS: u8 = 100;

#[elrond_wasm::module]
pub trait BaseModule {
    fn calculate_stake_modifier(&self, rarity: u8) -> u8 {
        // Safety check, should not happen
        if 0 == rarity {
            return 0;
        }

        // Rare (10) NFT
        if TOP_RARITY == rarity {
            return TOP_RARITY_STAKE_MODIFIER;
        }

        // Super Rare NFT
        if TOP_RARITY < rarity {
            return SUPER_RARE_STAKE_MODIFIER;
        }

        // Rare (1-9) NFT
        return BASE_STAKE_MODIFIER + INCREMENT_STAKE_MODIFIER * (rarity - 1);
    }

    fn calculate_max_referals(&self, staked_rarity: u16) -> u8 {
        // Can happen for view functions
        if 0 == staked_rarity {
            return 0;
        }

        // Rare (10) NFT
        if (TOP_RARITY as u16) == staked_rarity {
            return TOP_RARITY_REFERER_PERSONS;
        }

        // Super Rare NFT
        if (TOP_RARITY as u16) < staked_rarity {
            return SUPER_RARE_REFERER_PERSONS;
        }

        // Rare (1-9) NFT
        return BASE_REFERER_PERSONS + INCREMENT_REFERER_PERSONS * ((staked_rarity as u8) - 1);
    }

    #[only_owner]
    #[endpoint]
    fn set_minimum_lock_blocks(&self, blocks: u64) {
        self.minimum_lock_blocks().set(&blocks);
    }

    #[view]
    #[storage_mapper("minimumLockBlocks")]
    fn minimum_lock_blocks(&self) -> SingleValueMapper<u64>;
}
