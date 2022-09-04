use crate::{farm_token, owner};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const MAX_PERCENT: u64 = 10_000_000; // 100%, reached in 69.4444 days for 10 per block (144000 per day)
pub const PERCENT_PER_BLOCK: u64 = 10; // 0.0001% per block = 1.44% per day (14400 blocks per day)

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Token<M: ManagedTypeApi> {
    pub start_rewards_block: u64,
    pub end_rewards_block: u64,
    pub total_rewards: BigUint<M>,
}

#[elrond_wasm::module]
pub trait CustomRewardsModule:
    owner::OwnerModule
    + farm_token::FarmTokenModule
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + elrond_wasm_modules::pause::PauseModule
{
    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn top_up_rewards(
        &self,
        #[payment_token] payment_token: EgldOrEsdtTokenIdentifier,
        #[payment_amount] payment_amount: BigUint
    ) {
        let reward_token = self.reward_tokens(&payment_token);

        let current_block = self.blockchain().get_block_nonce();

        // Add new token if it doesn't exist
        if reward_token.is_empty() {
            // If adding the first token, set last_reward_block_nonce to appropriate value
            if self.last_reward_block_nonce().is_empty() {
                self.last_reward_block_nonce().set(current_block);
            }

            self.all_reward_tokens().push(&payment_token);

            let token = &Token{
                start_rewards_block: current_block,
                end_rewards_block: current_block + MAX_PERCENT / PERCENT_PER_BLOCK,
                total_rewards: payment_amount
            };

            reward_token.set(token);

            self.start_reward_per_share_token(&payment_token).set(self.reward_per_share().get());

            return;
        }

        let token = reward_token.get();

        // If adding tokens after rewards ended, reset token
        if current_block > token.end_rewards_block {
            reward_token.set(
                &Token{
                    start_rewards_block: current_block,
                    end_rewards_block: current_block + MAX_PERCENT / PERCENT_PER_BLOCK,
                    total_rewards: payment_amount
                }
            );

            return;
        }

        // Else calculate the number of blocks to extend by so the rewards per block remain the same
        let new_total_rewards: BigUint = &token.total_rewards + &payment_amount;
        let new_end_rewards_block: BigUint = &new_total_rewards * &BigUint::from(token.end_rewards_block) / &token.total_rewards;

        reward_token.set(
            &Token{
                start_rewards_block: token.start_rewards_block,
                end_rewards_block: new_end_rewards_block.to_u64().unwrap(),
                total_rewards: new_total_rewards,
            }
        );

        self.start_reward_per_share_token(&payment_token).set(self.reward_per_share().get());
    }

    // TODO: Add generate_aggregated_rewards() function which gets called in multiple places

    fn calculate_per_block_rewards(
        &self,
        current_block_nonce: u64,
        last_reward_block_nonce: u64,
    ) -> BigUint {
        if current_block_nonce <= last_reward_block_nonce || self.is_paused() {
            return BigUint::zero();
        }

        let block_nonce_diff = current_block_nonce - last_reward_block_nonce;

        BigUint::from(PERCENT_PER_BLOCK) * block_nonce_diff
    }

    fn calculate_reward_per_share_increase(
        &self,
        reward_increase: &BigUint,
        stake_modifier_total: &BigUint,
    ) -> BigUint {
        &(reward_increase * &self.division_safety_constant().get()) / stake_modifier_total
    }

    fn calculate_reward(
        &self,
        stake_modifier: &BigUint,
        staked_block: u64,
        current_reward_per_share: &BigUint,
        initial_reward_per_share: &BigUint,
        token: &EgldOrEsdtTokenIdentifier,
        reward_token: &Token<Self::Api>
    ) -> BigUint {
        if current_reward_per_share <= initial_reward_per_share {
            return BigUint::zero();
        }

        // Staked after rewards ended
        if staked_block > reward_token.end_rewards_block {
            return BigUint::zero();
        }

        let start_reward_per_share = &self.start_reward_per_share_token(token).get();
        let end_reward_per_share_mapper = self.end_reward_per_share_token(token);
        let end_reward_per_share_value = &end_reward_per_share_mapper.get();
        let end_reward_per_share = if !end_reward_per_share_mapper.is_empty() { end_reward_per_share_value } else { current_reward_per_share };

        let token_current_reward_per_share = if current_reward_per_share > end_reward_per_share { end_reward_per_share } else { current_reward_per_share };
        let token_initial_reward_per_share = if initial_reward_per_share < start_reward_per_share { start_reward_per_share } else { initial_reward_per_share };

        let reward_per_share_diff = token_current_reward_per_share - token_initial_reward_per_share;

        let percentage = stake_modifier * &reward_per_share_diff / self.division_safety_constant().get();

        return &reward_token.total_rewards * &percentage / &BigUint::from(MAX_PERCENT);
    }

    #[view(getDivisionSafetyConstant)]
    #[storage_mapper("division_safety_constant")]
    fn division_safety_constant(&self) -> SingleValueMapper<BigUint>;

    #[view]
    #[storage_mapper("reward_tokens")]
    fn reward_tokens(&self, token: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<Token<Self::Api>>;

    #[view]
    #[storage_mapper("all_reward_tokens")]
    fn all_reward_tokens(&self) -> VecMapper<EgldOrEsdtTokenIdentifier>;

    #[view]
    #[storage_mapper("last_reward_block_nonce")]
    fn last_reward_block_nonce(&self) -> SingleValueMapper<u64>;

    // Is stored in percentage increases, where MAX_PERCENT is 100%
    #[view(getRewardPerShare)]
    #[storage_mapper("reward_per_share")]
    fn reward_per_share(&self) -> SingleValueMapper<BigUint>;

    #[view]
    #[storage_mapper("start_reward_per_share_token")]
    fn start_reward_per_share_token(&self, token: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view]
    #[storage_mapper("end_reward_per_share_token")]
    fn end_reward_per_share_token(&self, token: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<BigUint>;
}
