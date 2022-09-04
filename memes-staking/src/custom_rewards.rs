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
            self.all_reward_tokens().push(&payment_token);

            let token = &Token{
                start_rewards_block: current_block,
                end_rewards_block: current_block + MAX_PERCENT / PERCENT_PER_BLOCK,
                total_rewards: payment_amount
            };

            reward_token.set(token);

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
    }

    #[view]
    #[storage_mapper("reward_tokens")]
    fn reward_tokens(&self, token: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<Token<Self::Api>>;

    #[view]
    #[storage_mapper("all_reward_tokens")]
    fn all_reward_tokens(&self) -> VecMapper<EgldOrEsdtTokenIdentifier>;
}
