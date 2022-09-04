use crate::{farm_token, owner};
use crate::common_structs::Nonce;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const MAX_PERCENT: u64 = 10_000_000; // 100%, reached in 46.2962 days for 15 per block (216000 per day)
pub const PERCENT_PER_BLOCK: u64 = 15; // 0.00015% per block = 2.16% per day
pub const PERCENT_TO_ADD_MORE: u64 = 100_000; // 10%, if less than this % of rewards remains, inscrease the time for rewards

#[derive(TopEncode, TopDecode, TypeAbi )]
pub struct RewardCapacity<M: ManagedTypeApi> {
    pub block_nonce: Nonce,
    pub accumulated_rewards: BigUint<M>,
    pub reward_per_share: BigUint<M>,
    pub end_reward_per_share: BigUint<M>,
    pub reward_capacity: BigUint<M>,
}

#[elrond_wasm::module]
pub trait CustomRewardsModule:
    owner::OwnerModule
    + farm_token::FarmTokenModule
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + elrond_wasm_modules::pause::PauseModule
{
    fn calculate_extra_rewards_since_last_allocation(&self) -> BigUint {
        let current_block_nonce = self.blockchain().get_block_nonce();
        let last_reward_nonce = self.last_reward_block_nonce().get();

        if current_block_nonce > last_reward_nonce {
            let extra_rewards = self.calculate_per_block_rewards(current_block_nonce, last_reward_nonce);

            self.last_reward_block_nonce().set(&current_block_nonce);

            extra_rewards
        } else {
            BigUint::zero()
        }
    }

    fn generate_aggregated_rewards(&self) {
        let mut extra_rewards = self.calculate_extra_rewards_since_last_allocation();

        if extra_rewards > 0 {
            let mut accumulated_rewards = self.accumulated_rewards().get();
            let total_rewards = &accumulated_rewards + &extra_rewards;
            let reward_capacity = self.all_reward_capacity().get();

            if total_rewards > reward_capacity {
                let amount_over_capacity = total_rewards - reward_capacity;
                extra_rewards -= amount_over_capacity;
            }

            accumulated_rewards += &extra_rewards;

            self.accumulated_rewards().set(&accumulated_rewards);

            // Check if rewards has ended for every token
            self.update_end_reward_per_share();

            self.update_reward_per_share(&extra_rewards);
        }
    }

    /*
        TODO: The first top up should set the rewards per block, and then subsequent top ups should use those rewards per block and maybe end of rewards time.
        The calculation of the stake should NOT take into consideration the reward_capacity!
     */
    #[only_owner]
    #[payable("*")]
    #[endpoint(topUpRewards)]
    fn top_up_rewards(
        &self,
        #[payment_token] payment_token: EgldOrEsdtTokenIdentifier,
        #[payment_amount] payment_amount: BigUint
    ) {
        self.generate_aggregated_rewards();

        let reward_capacity = self.reward_capacity(&payment_token);
        let all_reward_capacity = self.all_reward_capacity();
        let current_nonce = self.blockchain().get_block_nonce();
        let accumulated_rewards = self.accumulated_rewards().get();
        let reward_per_share = self.reward_per_share().get();

        self.prev_reward_capacity(&payment_token).set(&RewardCapacity {
            block_nonce: current_nonce,
            accumulated_rewards: accumulated_rewards.clone(),
            reward_per_share,
            end_reward_per_share: BigUint::zero(),
            reward_capacity: reward_capacity.get(),
        });

        // Start producing rewards when topping up first time
        if all_reward_capacity.is_empty() {
            self.last_reward_block_nonce().set(&current_nonce);
        }

        if reward_capacity.is_empty() {
            all_reward_capacity.update(|r| *r += MAX_PERCENT);

            self.reward_tokens().push(&payment_token);
        } else {
            let all_capacity = all_reward_capacity.get();

            // Start producing rewards again if topping up after accumulated rewards stopped
            if accumulated_rewards >= all_capacity {
                self.last_reward_block_nonce().set(&current_nonce);
            }

            if accumulated_rewards > (all_capacity - PERCENT_TO_ADD_MORE) {
                all_reward_capacity.update(|r| *r += MAX_PERCENT);
            }
        }

        reward_capacity.update(|r| *r += payment_amount);
    }

    fn calculate_per_block_rewards(
        &self,
        current_block_nonce: Nonce,
        last_reward_block_nonce: Nonce,
    ) -> BigUint {
        if current_block_nonce <= last_reward_block_nonce || self.is_paused() {
            return BigUint::zero();
        }

        let block_nonce_diff = current_block_nonce - last_reward_block_nonce;

        BigUint::from(PERCENT_PER_BLOCK) * block_nonce_diff
    }

    fn update_reward_per_share(&self, reward_increase: &BigUint) {
        let farm_token_supply = self.stake_modifier_total().get();
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
        if current_reward_per_share <= initial_reward_per_share {
            return BigUint::zero();
        }

        let reward_per_share_diff = current_reward_per_share - initial_reward_per_share;

        amount * &reward_per_share_diff / self.division_safety_constant().get()
    }

    /*
        TODO: Test all the cases that this works fine for existing stakers, new stakers, old stakers!
     */
    fn calculate_reward_token(
        &self,
        amount: &BigUint,
        staked_block: Nonce,
        current_reward_per_share: &BigUint,
        initial_reward_per_share: &BigUint,
        prev_reward_capacity: &RewardCapacity<Self::Api>,
        reward_capacity: &BigUint,
    ) -> BigUint {
        if current_reward_per_share <= initial_reward_per_share {
            return BigUint::zero();
        }

        let division_safety_constant = &self.division_safety_constant().get();

        let zero = BigUint::zero();

        // Nothing to claim if stake was from after the rewards ended
        if prev_reward_capacity.end_reward_per_share != zero && initial_reward_per_share > &prev_reward_capacity.end_reward_per_share {
            return BigUint::zero();
        }

        let end_reward_per_share = if prev_reward_capacity.end_reward_per_share != zero {
            &prev_reward_capacity.end_reward_per_share
        } else {
            current_reward_per_share
        };

        // In case he staked after rewards were added
        if prev_reward_capacity.block_nonce < staked_block {
            let reward_per_share_diff = end_reward_per_share - initial_reward_per_share;
            let real_amount = amount * &reward_per_share_diff;

            return (reward_capacity - &prev_reward_capacity.reward_capacity) * &(&real_amount / MAX_PERCENT) / division_safety_constant;
        }

        // In case he was staked also before rewards were added
        let mut reward = BigUint::zero();

        let reward_per_share_diff_before = &prev_reward_capacity.reward_per_share - initial_reward_per_share;
        let real_amount_before = amount * &reward_per_share_diff_before;

        reward = &reward + &(&prev_reward_capacity.reward_capacity * &(&real_amount_before / MAX_PERCENT));

        let reward_per_share_diff_after = end_reward_per_share - &prev_reward_capacity.reward_per_share;
        let real_amount_after = amount * &reward_per_share_diff_after;

        reward = &reward + &((reward_capacity - &prev_reward_capacity.reward_capacity) * &(&real_amount_after / MAX_PERCENT));

        return reward / division_safety_constant;
    }

    fn update_end_reward_per_share(&self) {
        let accumulated_rewards = &self.accumulated_rewards().get();
        let reward_per_share = self.reward_per_share().get();
        let max_percent = &BigUint::from(MAX_PERCENT);
        let zero = BigUint::zero();

        for token in self.reward_tokens().iter() {
            let prev_reward_capacity = self.prev_reward_capacity(&token);
            let prev_reward_capacity_struct = prev_reward_capacity.get();

            if prev_reward_capacity_struct.end_reward_per_share != zero {
                continue;
            }

            let percent = &(accumulated_rewards - &prev_reward_capacity_struct.accumulated_rewards);
            let actual_accumulated_rewards = core::cmp::min(max_percent, percent);

            // Send end_reward_per_share to appropriate value
            if actual_accumulated_rewards.eq(max_percent) {
                prev_reward_capacity.set(&RewardCapacity {
                    block_nonce: prev_reward_capacity_struct.block_nonce,
                    accumulated_rewards: prev_reward_capacity_struct.accumulated_rewards,
                    reward_per_share: prev_reward_capacity_struct.reward_per_share,
                    end_reward_per_share: reward_per_share.clone(),
                    reward_capacity: prev_reward_capacity_struct.reward_capacity,
                })
            }
        }
    }

    #[view(getRewardPerShare)]
    #[storage_mapper("reward_per_share")]
    fn reward_per_share(&self) -> SingleValueMapper<BigUint>;

    #[view(getAccumulatedRewards)]
    #[storage_mapper("accumulatedRewards")]
    fn accumulated_rewards(&self) -> SingleValueMapper<BigUint>;

    #[view(getRewardTokens)]
    #[storage_mapper("reward_tokens")]
    fn reward_tokens(&self) -> VecMapper<EgldOrEsdtTokenIdentifier>;

    #[view(getRewardCapacity)]
    #[storage_mapper("reward_capacity")]
    fn reward_capacity(&self, token: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getPrevRewardCapacity)]
    #[storage_mapper("prev_reward_capacity")]
    fn prev_reward_capacity(&self, token: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<RewardCapacity<Self::Api>>;

    #[view(getAllRewardCapacity)]
    #[storage_mapper("all_reward_capacity")]
    fn all_reward_capacity(&self) -> SingleValueMapper<BigUint>;
}
