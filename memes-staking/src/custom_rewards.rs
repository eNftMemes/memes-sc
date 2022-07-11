use crate::{farm_token, owner};
use crate::common_structs::Nonce;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const BLOCKS_IN_YEAR: u64 = 31_536_000 / 6; // seconds_in_year / 6_seconds_per_block

pub const BLOCKS_IN_DAY: u64 = 86_400 / 6; // seconds_in_day / 6_seconds_per_block

pub const MAX_PERCENT: u64 = 1_000_000; // 100%
pub const PERCENT_PER_BLOCK: u64 = 2; // 0.0002% per block = 2.88% per day

#[elrond_wasm::module]
pub trait CustomRewardsModule:
    owner::OwnerModule
    + farm_token::FarmTokenModule
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + elrond_wasm_modules::pause::PauseModule
{
    // fn calculate_extra_rewards_since_last_allocation(&self) -> BigUint {
    //     let current_block_nonce = self.blockchain().get_block_nonce();
    //     let last_reward_nonce = self.last_reward_block_nonce().get();
    //
    //     if current_block_nonce > last_reward_nonce {
    //         let extra_rewards_unbounded =
    //             self.calculate_per_block_rewards(current_block_nonce, last_reward_nonce);
    //
    //         let farm_token_supply = self.farm_token_supply().get();
    //         let extra_rewards_apr_bounded_per_block =
    //             self.get_amount_apr_bounded(&farm_token_supply);
    //
    //         let block_nonce_diff = current_block_nonce - last_reward_nonce;
    //         let extra_rewards_apr_bounded = extra_rewards_apr_bounded_per_block * block_nonce_diff;
    //
    //         self.last_reward_block_nonce().set(&current_block_nonce);
    //
    //         core::cmp::min(extra_rewards_unbounded, extra_rewards_apr_bounded)
    //     } else {
    //         BigUint::zero()
    //     }
    // }
    //
    // fn generate_aggregated_rewards(&self) {
    //     let mut extra_rewards = self.calculate_extra_rewards_since_last_allocation();
    //
    //     if extra_rewards > 0 {
    //         let mut accumulated_rewards = self.accumulated_rewards().get();
    //         let total_rewards = &accumulated_rewards + &extra_rewards;
    //         let reward_capacity = self.reward_capacity().get();
    //         if total_rewards > reward_capacity {
    //             let amount_over_capacity = total_rewards - reward_capacity;
    //             extra_rewards -= amount_over_capacity;
    //         }
    //
    //         accumulated_rewards += &extra_rewards;
    //         self.accumulated_rewards().set(&accumulated_rewards);
    //
    //         self.update_reward_per_share(&extra_rewards);
    //     }
    // }

    #[only_owner]
    #[payable("*")]
    #[endpoint(topUpRewards)]
    fn top_up_rewards(
        &self,
        #[payment_token] payment_token: EgldOrEsdtTokenIdentifier,
        #[payment_amount] payment_amount: BigUint
    ) {
        self.reward_capacity(payment_token).update(|r| *r += payment_amount);
    }

    // #[only_owner]
    // #[endpoint(setPerBlockRewardAmount)]
    // fn set_per_block_rewards(&self, per_block_amount: BigUint) {
    //     require!(per_block_amount != 0, "Amount cannot be zero");
    //
    //     self.generate_aggregated_rewards();
    //     self.per_block_reward_amount().set(&per_block_amount);
    // }

    // #[only_owner]
    // #[endpoint(setMaxApr)]
    // fn set_max_apr(&self, max_apr: BigUint) {
    //     require!(max_apr != 0, "Max APR cannot be zero");
    //
    //     self.generate_aggregated_rewards();
    //     self.max_annual_percentage_rewards().set(&max_apr);
    // }

    #[only_owner]
    #[endpoint(setMinUnbondEpochs)]
    fn set_min_unbond_epochs(&self, min_unbond_epochs: u64) {
        self.min_unbond_epochs().set(&min_unbond_epochs);
    }

    fn calculate_per_block_rewards(
        &self,
        current_block_nonce: Nonce,
        last_reward_block_nonce: Nonce,
    ) -> BigUint {
        if current_block_nonce <= last_reward_block_nonce || !self.is_paused() {
            return BigUint::zero();
        }

        let block_nonce_diff = current_block_nonce - last_reward_block_nonce;

        BigUint::from(PERCENT_PER_BLOCK) * block_nonce_diff
    }

    fn update_reward_per_share(&self, reward_increase: &BigUint) {
        let farm_token_supply = self.farm_token_supply().get();
        if farm_token_supply > 0 {
            let increase =
                self.calculate_reward_per_share_increase(reward_increase, &farm_token_supply);
            self.reward_per_share().update(|r| *r += increase);
        }
    }

    fn calculate_reward_per_share_increase(
        &self,
        reward_increase: &BigUint,
        farm_token_supply: &BigUint,
    ) -> BigUint {
        &(reward_increase * &self.division_safety_constant().get()) / farm_token_supply
    }

    fn calculate_reward(
        &self,
        amount: &BigUint,
        current_reward_per_share: &BigUint,
        initial_reward_per_share: &BigUint,
    ) -> BigUint {
        if current_reward_per_share > initial_reward_per_share {
            let reward_per_share_diff = current_reward_per_share - initial_reward_per_share;
            amount * &reward_per_share_diff / self.division_safety_constant().get()
        } else {
            BigUint::zero()
        }
    }

    // fn get_amount_apr_bounded(&self, amount: &BigUint) -> BigUint {
    //     amount * &max_apr / MAX_PERCENT / BLOCKS_IN_YEAR
    // }

    #[view(getRewardPerShare)]
    #[storage_mapper("reward_per_share")]
    fn reward_per_share(&self) -> SingleValueMapper<BigUint>;

    #[view(getAccumulatedRewards)]
    #[storage_mapper("accumulatedRewards")]
    fn accumulated_rewards(&self) -> SingleValueMapper<BigUint>;

    #[view(getRewardCapacity)]
    #[storage_mapper("reward_capacity")]
    fn reward_capacity(&self, token: EgldOrEsdtTokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getMinUnbondEpochs)]
    #[storage_mapper("minUnbondEpochs")]
    fn min_unbond_epochs(&self) -> SingleValueMapper<u64>;
}
