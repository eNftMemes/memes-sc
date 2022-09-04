use crate::{farm_token, owner};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const MAX_PERCENT: u64 = 10_000_000; // 100%, reached in 46.2962 days for 15 per block (216000 per day)
pub const PERCENT_PER_BLOCK: u64 = 15; // 0.00015% per block = 2.16% per day
pub const PERCENT_TO_ADD_MORE: u64 = 100_000; // 10%, if less than this % of rewards remains, inscrease the time for rewards

#[elrond_wasm::module]
pub trait CustomRewardsModule:
    owner::OwnerModule
    + farm_token::FarmTokenModule
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + elrond_wasm_modules::pause::PauseModule
{
    #[only_owner]
    #[payable("*")]
    #[endpoint(topUpRewards)]
    fn top_up_rewards(
        &self,
        #[payment_token] payment_token: EgldOrEsdtTokenIdentifier,
        #[payment_amount] payment_amount: BigUint
    ) {
        // self.generate_aggregated_rewards();
        //
        // let reward_capacity = self.reward_capacity(&payment_token);
        // let all_reward_capacity = self.all_reward_capacity();
        // let current_nonce = self.blockchain().get_block_nonce();
        // let accumulated_rewards = self.accumulated_rewards().get();
        // let reward_per_share = self.reward_per_share().get();
        //
        // self.prev_reward_capacity(&payment_token).set(&RewardCapacity {
        //     block_nonce: current_nonce,
        //     accumulated_rewards: accumulated_rewards.clone(),
        //     reward_per_share,
        //     end_reward_per_share: BigUint::zero(),
        //     reward_capacity: reward_capacity.get(),
        // });
        //
        // // Start producing rewards when topping up first time
        // if all_reward_capacity.is_empty() {
        //     self.last_reward_block_nonce().set(&current_nonce);
        // }
        //
        // if reward_capacity.is_empty() {
        //     all_reward_capacity.update(|r| *r += MAX_PERCENT);
        //
        //     self.reward_tokens().push(&payment_token);
        // } else {
        //     let all_capacity = all_reward_capacity.get();
        //
        //     // Start producing rewards again if topping up after accumulated rewards stopped
        //     if accumulated_rewards >= all_capacity {
        //         self.last_reward_block_nonce().set(&current_nonce);
        //     }
        //
        //     if accumulated_rewards > (all_capacity - PERCENT_TO_ADD_MORE) {
        //         all_reward_capacity.update(|r| *r += MAX_PERCENT);
        //     }
        // }
        //
        // reward_capacity.update(|r| *r += payment_amount);
    }

    #[view(getRewardTokens)]
    #[storage_mapper("reward_tokens")]
    fn reward_tokens(&self) -> VecMapper<EgldOrEsdtTokenIdentifier>;
}
