use crate::{farm_token, base};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const MAX_PERCENT: u64 = 10_000_000; // 100%, reached in 69.4444 days for 10 per block (144000 per day)
pub const PERCENT_PER_BLOCK: u64 = 10; // 0.0001% per block = 1.44% per day (14400 blocks per day)

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Token<M: ManagedTypeApi> {
    pub start_rewards_block: u64,
    pub end_rewards_block: u64,
    pub total_rewards: BigUint<M>,
    pub reward_per_block: BigUint<M>,
}

#[elrond_wasm::module]
pub trait CustomRewardsModule:
    base::BaseModule
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
        let current_block_nonce = self.blockchain().get_block_nonce();

        self.generate_aggregated_rewards(current_block_nonce);

        let reward_token = self.reward_tokens(&payment_token);

        let reward_per_block = (&payment_amount * &BigUint::from(PERCENT_PER_BLOCK) * &self.division_safety_constant().get())
            / &BigUint::from(MAX_PERCENT);

        // Add new token if it doesn't exist
        if reward_token.is_empty() {
            // If adding the first token, set last_reward_block_nonce to appropriate value
            if self.last_reward_block_nonce().is_empty() {
                self.last_reward_block_nonce().set(current_block_nonce);
            }

            self.all_reward_tokens().push(&payment_token);

            let token = &Token{
                start_rewards_block: current_block_nonce,
                end_rewards_block: current_block_nonce + MAX_PERCENT / PERCENT_PER_BLOCK,
                total_rewards: payment_amount,
                reward_per_block,
            };

            reward_token.set(token);

            self.start_reward_per_share_token(&payment_token).set(self.reward_per_share().get());

            return;
        }

        let token = reward_token.get();

        // If adding tokens after rewards ended, reset token
        if current_block_nonce > token.end_rewards_block {
            reward_token.set(
                &Token{
                    start_rewards_block: current_block_nonce,
                    end_rewards_block: current_block_nonce + MAX_PERCENT / PERCENT_PER_BLOCK,
                    total_rewards: payment_amount,
                    reward_per_block,
                }
            );

            self.start_reward_per_share_token(&payment_token).set(self.reward_per_share().get());
            self.end_reward_per_share_token(&payment_token).clear();

            return;
        }

        // Else calculate the number of blocks to extend by so the rewards per block remain the same
        let new_total_rewards: BigUint = &token.total_rewards + &payment_amount;
        let new_end_rewards_block: BigUint = &new_total_rewards * &BigUint::from(token.end_rewards_block - token.start_rewards_block) / &token.total_rewards
            + &BigUint::from(token.start_rewards_block);

        reward_token.set(
            &Token{
                start_rewards_block: token.start_rewards_block,
                end_rewards_block: new_end_rewards_block.to_u64().unwrap(),
                total_rewards: new_total_rewards,
                reward_per_block: token.reward_per_block
            }
        );
    }

    fn generate_aggregated_rewards(&self, current_block_nonce: u64) {
        let last_reward_nonce_mapper = self.last_reward_block_nonce();

        let initial_last_reward_block = last_reward_nonce_mapper.get();
        let extra_rewards = self.calculate_extra_rewards_since_last_allocation(current_block_nonce, &last_reward_nonce_mapper, initial_last_reward_block);

        if extra_rewards <= 0 {
            return;
        }

        let stake_modifier_total = self.stake_modifier_total().get();
        let reward_per_share_mapper = self.reward_per_share();
        let initial_reward_per_share = reward_per_share_mapper.get();
        let new_reward_per_share = self.update_reward_per_share(&extra_rewards, &stake_modifier_total, &reward_per_share_mapper, &initial_reward_per_share);

        if new_reward_per_share <= 0 {
            return;
        }

        // Check if rewards has ended for every token
        self.update_end_reward_per_share(current_block_nonce, initial_last_reward_block, &initial_reward_per_share, &stake_modifier_total);
    }

    fn calculate_extra_rewards_since_last_allocation(
        &self,
        current_block_nonce: u64,
        last_reward_nonce_mapper: &SingleValueMapper<u64>,
        last_reward_nonce: u64
    ) -> BigUint {
        if current_block_nonce <= last_reward_nonce {
            return BigUint::zero();
        }

        let extra_rewards =
            self.calculate_per_block_rewards(current_block_nonce, last_reward_nonce);

        last_reward_nonce_mapper.set(&current_block_nonce);

        return extra_rewards;
    }

    fn update_reward_per_share(&self, reward_increase: &BigUint, stake_modifier_total: &BigUint, reward_per_share_mapper: &SingleValueMapper<BigUint>, reward_per_share: &BigUint) -> BigUint {
        if *stake_modifier_total <= 0 {
            return BigUint::zero();
        }

        let increase =
            self.calculate_reward_per_share_increase(reward_increase, stake_modifier_total);

        let new_reward_per_share = reward_per_share + &increase;

        reward_per_share_mapper.set(&new_reward_per_share);

        return new_reward_per_share;
    }

    fn update_end_reward_per_share(&self, current_block_nonce: u64, initial_last_reward_block: u64, initial_reward_per_share: &BigUint, stake_modifier_total: &BigUint) {
        for token in self.all_reward_tokens().iter() {
            let end_reward_per_share_mapper = self.end_reward_per_share_token(&token);

            if !end_reward_per_share_mapper.is_empty() {
                continue;
            }

            let token_data = self.reward_tokens(&token).get();

            if token_data.end_rewards_block > current_block_nonce {
                continue;
            }

            let extra_rewards =
                self.calculate_per_block_rewards(token_data.end_rewards_block, initial_last_reward_block);
            let increase =
                self.calculate_reward_per_share_increase(&extra_rewards, &stake_modifier_total);

            let new_reward_per_share = initial_reward_per_share + &increase;

            end_reward_per_share_mapper.set(&new_reward_per_share);
        }
    }

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

        return &reward_token.reward_per_block * &percentage / &BigUint::from(PERCENT_PER_BLOCK) / self.division_safety_constant().get();
    }

    #[view(getDivisionSafetyConstant)]
    #[storage_mapper("division_safety_constant")]
    fn division_safety_constant(&self) -> SingleValueMapper<BigUint>;

    // TODO: Add a view to get all tokens info
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
