#![no_std]
#![feature(generic_associated_types)]

use staking::TopMemeAttributes;
use crate::common_structs::Nonce;
use crate::custom_rewards::MAX_PERCENT;
use crate::farm_token::{StakingFarmTokenAttributes, TOP_RARITY};

elrond_wasm::imports!();

mod owner;
mod staking;
mod farm_token;
mod common_structs;
mod custom_rewards;

const NFT_AMOUNT: u32 = 1;

const DEFAULT_MINUMUM_LOCK_BLOCKS: u64 = 5 * 14_400; // ~5 days (3 * 14400 blocks)

const DIVISION_SAFETY_CONSTANT: u32 = 1000000000;

const BASE_REFERER_PERSONS: u8 = 15;
const INCREMENT_REFERER_PERSONS: u8 = 5;
const SUPER_RARE_REFERER_PERSONS: u8 = 100;


#[elrond_wasm::contract]
pub trait StakingContract: owner::OwnerModule
    + elrond_wasm_modules::pause::PauseModule
    + farm_token::FarmTokenModule
    + custom_rewards::CustomRewardsModule
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(&self, voting_contract: &ManagedAddress, auction_contract: &ManagedAddress, token_identifier_top: &TokenIdentifier) {
        self.voting_sc().set(voting_contract);
        self.auction_sc().set(auction_contract);
        self.token_identifier_top().set(token_identifier_top);

        self.minimum_lock_blocks().set_if_empty(DEFAULT_MINUMUM_LOCK_BLOCKS);

        let division_safety_constant = BigUint::from(DIVISION_SAFETY_CONSTANT);

        self.division_safety_constant().set_if_empty(division_safety_constant);
    }

    #[payable("*")]
    #[endpoint]
    fn stake(
        &self,
        #[payment_token] nft_type: EgldOrEsdtTokenIdentifier,
        #[payment_nonce] nonce: u64,
        #[payment_amount] nft_amount: BigUint,
    ) {
        require!(self.not_paused(), "Contract paused, can't stake NFTs");

        require!(nft_amount == NFT_AMOUNT, "Nft amount should be 1");

        let token_identifier_top = self.token_identifier_top().get();

        require!(nft_type == token_identifier_top, "Nft is not of the correct type");

        let own_address: ManagedAddress = self.blockchain().get_sc_address();
        let token_data: EsdtTokenData<Self::Api> = self.blockchain().get_esdt_token_data(
            &own_address,
            &nft_type.into_esdt_option().unwrap(),
            nonce
        );
        let nft_attributes = token_data.decode_attributes::<TopMemeAttributes<Self::Api>>();

        let caller = self.blockchain().get_caller();

        self.staked_rarity(&caller).update(|value| *value += nft_attributes.rarity);

        self.generate_aggregated_rewards();

        let attributes = StakingFarmTokenAttributes {
            reward_per_share: self.reward_per_share().get(),
            rarity: nft_attributes.rarity,
            staker: caller,
            nft_nonce: nonce,
            staked_block: self.blockchain().get_block_nonce(),
        };

        let farm_token = self.farm_token().get_token_id();

        let new_rokens = self.mint_farm_token(farm_token, &attributes);

        self.send().direct_esdt(&self.blockchain().get_caller(), &new_rokens.token_identifier, new_rokens.token_nonce, &new_rokens.amount);
    }

    #[payable("*")]
    #[endpoint]
    fn unstake(
        &self,
        #[payment_token] payment_token_id: EgldOrEsdtTokenIdentifier,
        #[payment_nonce] token_nonce: Nonce,
        #[payment_amount] payment_amount: BigUint,
    ) {
        require!(!self.is_paused(), "Paused");

        let farm_token_id = self.farm_token().get_token_id();

        require!(farm_token_id.is_valid_esdt_identifier(), "No farm token");

        require!(payment_token_id == farm_token_id, "Bad input token");
        require!(payment_amount > 0u32, "Payment amount cannot be zero");

        let token_identifier = &payment_token_id.into_esdt_option().unwrap();
        let own_address: ManagedAddress = self.blockchain().get_sc_address();
        let token_data: EsdtTokenData<Self::Api> = self.blockchain().get_esdt_token_data(
            &own_address,
            token_identifier,
            token_nonce
        );
        let farm_attributes = token_data.decode_attributes::<StakingFarmTokenAttributes<Self::Api>>();

        require!(
            self.blockchain().get_block_nonce() > farm_attributes.staked_block + self.minimum_lock_blocks().get(),
            "Minimum lock time has not yet passed"
        );

        self.generate_aggregated_rewards();

        let caller = self.blockchain().get_caller();
        self.burn_farm_tokens(&token_identifier, token_nonce, &payment_amount, farm_attributes.rarity);

        self.staked_rarity(&farm_attributes.staker).update(|value| *value -= farm_attributes.rarity);

        let nft_amount = BigUint::from(NFT_AMOUNT);
        self.send().direct_esdt(&caller, &self.token_identifier_top().get(), farm_attributes.nft_nonce, &nft_amount);

        // let reward = self.calculate_reward(
        //     &payment_amount,
        //     &self.reward_per_share().get(),
        //     &farm_attributes.reward_per_share,
        // );

        // TODO: Send rewards if any
        // let farm_token_payment = self.create_and_send_unbond_tokens(&caller, farm_token_id, payment_amount);
        //
        // self.send_rewards(&reward_token_id, &reward, &caller);
    }

    #[endpoint]
    fn use_referer(&self, referer_address: &ManagedAddress) {
        let staked_rarity = self.staked_rarity(referer_address);

        require!(!staked_rarity.is_empty(), "Referer doesn't have an NFT staked currently");

        let caller = self.blockchain().get_caller();
        let referer = self.referer(&caller);

        require!(referer.is_empty(), "You already have a referer set");

        let number_of_referals = self.number_of_referals(&referer_address);
        let rarity = staked_rarity.get();

        let mut max_referals = BASE_REFERER_PERSONS;

        if TOP_RARITY < rarity {
            max_referals = SUPER_RARE_REFERER_PERSONS;
        } else if 1 > rarity {
            max_referals = max_referals + INCREMENT_REFERER_PERSONS * (rarity - 1);
        }

        require!(number_of_referals.get() < max_referals, "Maximum number of referals reached for this referer");

        referer.set(referer_address);
        number_of_referals.update(|r| *r = *r + 1);

        // TODO: Call voting & auction contracts
    }

    #[view(calculateRewardsForGivenPosition)]
    fn calculate_rewards_for_given_position(
        &self,
        attributes: StakingFarmTokenAttributes<Self::Api>,
    ) -> BigUint {
        let stake_modifier = self.calculate_stake_modifier(attributes.rarity);

        require!(stake_modifier > 0, "Zero liquidity input");
        let stake_modifier_supply = self.stake_modifier_total().get();
        require!(stake_modifier_supply >= (stake_modifier as u32), "Not enough supply");

        let last_reward_nonce = self.last_reward_block_nonce().get();
        let current_block_nonce = self.blockchain().get_block_nonce();

        let reward_increase =
            self.calculate_per_block_rewards(current_block_nonce, last_reward_nonce);

        let reward_per_share_increase =
            self.calculate_reward_per_share_increase(&reward_increase, &stake_modifier_supply);

        let future_reward_per_share = self.reward_per_share().get() + reward_per_share_increase;

        self.calculate_reward(
            &BigUint::from(stake_modifier),
            &future_reward_per_share,
            &attributes.reward_per_share,
        )
    }

    #[view(calculateRewardsForGivenPositionAndTokens)]
    fn calculate_rewards_for_given_position_and_tokens(
        &self,
        attributes: StakingFarmTokenAttributes<Self::Api>,
    ) -> MultiValueEncoded<(EgldOrEsdtTokenIdentifier, BigUint)> {
        let stake_modifier = self.calculate_stake_modifier(attributes.rarity);

        require!(stake_modifier > 0, "Zero liquidity input");
        let farm_token_supply = self.stake_modifier_total().get();
        require!(farm_token_supply >= (stake_modifier as u32), "Not enough supply");

        let last_reward_nonce = self.last_reward_block_nonce().get();
        let current_block_nonce = self.blockchain().get_block_nonce();

        let reward_increase =
            self.calculate_per_block_rewards(current_block_nonce, last_reward_nonce);

        let reward_per_share_increase =
            self.calculate_reward_per_share_increase(&reward_increase, &farm_token_supply);

        let reward_per_share = self.reward_per_share().get();
        let future_reward_per_share = &reward_per_share + &reward_per_share_increase;

        let accumulated_rewards = &self.accumulated_rewards().get();
        let max_percent = &BigUint::from(MAX_PERCENT);
        let zero = BigUint::zero();

        let mut result: MultiValueEncoded<(EgldOrEsdtTokenIdentifier, BigUint)> = MultiValueEncoded::new();
        for token in self.reward_tokens().iter() {
            let reward_capacity = self.reward_capacity(&token).get();

            let prev_reward_capacity = self.prev_reward_capacity(&token);
            let mut prev_reward_capacity_struct = prev_reward_capacity.get();

            if prev_reward_capacity_struct.end_reward_per_share == zero {
                let percent = &(accumulated_rewards - &prev_reward_capacity_struct.accumulated_rewards);
                let actual_accumulated_rewards = core::cmp::min(max_percent, percent);

                // Send end_reward_per_share to appropriate value
                if actual_accumulated_rewards.eq(max_percent) {
                    prev_reward_capacity_struct.end_reward_per_share = reward_per_share.clone();
                }
            }

            let amount = self.calculate_reward_token(
                &BigUint::from(stake_modifier),
                attributes.staked_block,
                &future_reward_per_share,
                &attributes.reward_per_share,
                &prev_reward_capacity_struct,
                &reward_capacity
            );

            result.push((token, amount));
        }

        return result;
    }

    #[view]
    #[storage_mapper("votingSc")]
    fn voting_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[view]
    #[storage_mapper("auctionSc")]
    fn auction_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[view]
    #[storage_mapper("tokenIdentifierTop")]
    fn token_identifier_top(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view]
    #[storage_mapper("stakedRarity")]
    fn staked_rarity(&self, address: &ManagedAddress) -> SingleValueMapper<u8>;

    #[view]
    #[storage_mapper("referer")]
    fn referer(&self, address: &ManagedAddress) -> SingleValueMapper<ManagedAddress>;

    #[view]
    #[storage_mapper("numberOfReferals")]
    fn number_of_referals(&self, address: &ManagedAddress) -> SingleValueMapper<u8>;
}
