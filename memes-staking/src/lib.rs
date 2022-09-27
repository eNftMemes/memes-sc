#![no_std]
#![feature(generic_associated_types)]

use crate::farm_token::{StakingFarmTokenAttributes};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod base;
mod farm_token;
mod custom_rewards;

const NFT_AMOUNT: u32 = 1;
const DEFAULT_MINUMUM_LOCK_BLOCKS: u64 = 5 * 14_400; // ~5 days (3 * 14400 blocks)
const DIVISION_SAFETY_CONSTANT: u32 = 1000000000;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct TopMemeAttributes<M: ManagedTypeApi> {
    pub rarity: u8,
    pub original_nonce: u64,
    pub period: u64,
    pub category: ManagedBuffer<M>,
    pub creator: ManagedAddress<M>,
}

#[elrond_wasm::contract]
pub trait StakingContract: base::BaseModule
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
    fn stake(&self) {
        require!(self.not_paused(), "Contract paused, can't stake NFTs");

        let (nft_type, nonce, nft_amount) = self.call_value().single_esdt().into_tuple();

        require!(nft_amount == NFT_AMOUNT, "Nft amount should be 1");

        let token_identifier_top = self.token_identifier_top().get();

        require!(nft_type == token_identifier_top, "Nft is not of the correct type");

        let own_address: ManagedAddress = self.blockchain().get_sc_address();
        let token_data: EsdtTokenData<Self::Api> = self.blockchain().get_esdt_token_data(
            &own_address,
            &nft_type,
            nonce
        );
        let nft_attributes = token_data.decode_attributes::<TopMemeAttributes<Self::Api>>();

        let caller = self.blockchain().get_caller();

        self.staked_rarity(&caller).update(|value| *value += nft_attributes.rarity as u16);

        let current_block_nonce = self.blockchain().get_block_nonce();

        self.generate_aggregated_rewards(current_block_nonce);

        let attributes = StakingFarmTokenAttributes {
            rarity: nft_attributes.rarity,
            staker: caller,
            nft_nonce: nonce,
            staked_block: current_block_nonce,
            reward_per_share: self.reward_per_share().get(),
        };

        let farm_token = self.farm_token().get_token_id();

        let new_rokens = self.mint_farm_token(farm_token, &attributes, Option::Some(true));

        self.send().direct_esdt(&self.blockchain().get_caller(), &new_rokens.token_identifier, new_rokens.token_nonce, &new_rokens.amount);
    }

    #[payable("*")]
    #[endpoint]
    fn unstake(
        &self,
        #[payment_token] payment_token_id: EgldOrEsdtTokenIdentifier,
        #[payment_nonce] token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
    ) {
        require!(self.not_paused(), "Contract paused, can't unstake NFTs");

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

        let current_block_nonce = self.blockchain().get_block_nonce();

        require!(
            current_block_nonce > farm_attributes.staked_block + self.minimum_lock_blocks().get(),
            "Minimum lock time has not yet passed"
        );

        self.generate_aggregated_rewards(current_block_nonce);

        let caller = self.blockchain().get_caller();
        let current_reward_per_share = self.reward_per_share().get();
        let mut payments = self.claim_rewards_common(
            &caller,
            &farm_attributes,
            &current_reward_per_share
        );

        self.burn_farm_tokens(&token_identifier, token_nonce, &payment_amount, Option::Some(farm_attributes.rarity));

        self.staked_rarity(&farm_attributes.staker).update(|value| *value -= farm_attributes.rarity as u16);

        payments.push(EsdtTokenPayment::new(self.token_identifier_top().get(), farm_attributes.nft_nonce, BigUint::from(NFT_AMOUNT)));

        self.send().direct_multi(&caller, &payments);
    }

    #[payable("*")]
    #[endpoint]
    fn claim_rewards(&self) {
        require!(self.not_paused(), "Contract paused, can't claim rewards");

        let farm_token_id = self.farm_token().get_token_id();

        require!(farm_token_id.is_valid_esdt_identifier(), "No farm token");

        let (token_identifier, token_nonce, payment_amount) = self.call_value().single_esdt().into_tuple();
        require!(payment_amount > 0u32, "Zero amount");
        require!(token_identifier == farm_token_id, "Unknown farm token");

        let current_block_nonce = self.blockchain().get_block_nonce();

        self.generate_aggregated_rewards(current_block_nonce);

        let own_address: ManagedAddress = self.blockchain().get_sc_address();
        let token_data: EsdtTokenData<Self::Api> = self.blockchain().get_esdt_token_data(
            &own_address,
            &token_identifier,
            token_nonce
        );
        let attributes = token_data.decode_attributes::<StakingFarmTokenAttributes<Self::Api>>();

        let caller = self.blockchain().get_caller();
        let current_reward_per_share = self.reward_per_share().get();
        let mut payments = self.claim_rewards_common(
            &caller,
            &attributes,
            &current_reward_per_share
        );

        let new_attributes = StakingFarmTokenAttributes {
            rarity: attributes.rarity,
            staker: attributes.staker,
            nft_nonce: attributes.nft_nonce,
            staked_block: current_block_nonce,
            reward_per_share: current_reward_per_share,
        };

        self.burn_farm_tokens(&token_identifier, token_nonce, &payment_amount, Option::None);

        let new_rokens = self.mint_farm_token(token_identifier, &new_attributes, Option::None);

        payments.push(EsdtTokenPayment::new(new_rokens.token_identifier, new_rokens.token_nonce, new_rokens.amount));

        self.send().direct_multi(&caller, &payments);
    }

    #[endpoint]
    fn use_referer(&self, referer_address: &ManagedAddress) {
        require!(self.not_paused(), "Contract paused, can't use referer");

        let staked_rarity = self.staked_rarity(referer_address).get();

        require!(staked_rarity > 0, "Referer doesn't have any NFT staked currently");

        let caller = self.blockchain().get_caller();

        require!(&caller != referer_address, "You can not refer yourself");

        let referer = self.referer(&caller);

        require!(referer.is_empty(), "You already have a referer set");

        let number_of_referals = self.number_of_referals(&referer_address);
        let max_referals = self.calculate_max_referals(staked_rarity);

        require!(number_of_referals.get() < max_referals, "Maximum number of referals reached for this referer");

        referer.set(referer_address);
        number_of_referals.update(|r| *r = *r + 1);

        // TODO: Call voting & auction contracts
    }

    fn claim_rewards_common(
        &self,
        caller: &ManagedAddress,
        attributes: &StakingFarmTokenAttributes<Self::Api>,
        current_reward_per_share: &BigUint,
    ) -> ManagedVec<EsdtTokenPayment<Self::Api>> {
        let stake_modifier = self.calculate_stake_modifier(attributes.rarity);

        let mut payments = ManagedVec::new();

        for token in self.all_reward_tokens().iter() {
            let reward_token = self.reward_tokens(&token).get();

            let amount = self.calculate_reward(
                &BigUint::from(stake_modifier as u16),
                attributes.staked_block,
                current_reward_per_share,
                &attributes.reward_per_share,
                &token,
                &reward_token
            );

            if amount > 0 {
                if token.is_egld() {
                    self.send().direct_egld(caller, &amount);
                } else {
                    payments.push(EsdtTokenPayment::new(token.into_esdt_option().unwrap(), 0, amount));
                }
            }
        }

        return payments;
    }

    #[view]
    fn calculate_rewards_for_multiple_positions(
        &self,
        all_attributes: MultiValueEncoded<StakingFarmTokenAttributes<Self::Api>>
    ) -> MultiValueEncoded<MultiValueEncoded<(EgldOrEsdtTokenIdentifier, BigUint)>> {
        let mut rewards_for_positions = MultiValueEncoded::new();

        for attributes in all_attributes.into_iter() {
            rewards_for_positions.push(self.calculate_rewards_for_given_position(&attributes));
        }

        return rewards_for_positions;
    }

    #[view]
    fn calculate_rewards_for_given_position(
        &self,
        attributes: &StakingFarmTokenAttributes<Self::Api>,
    ) -> MultiValueEncoded<(EgldOrEsdtTokenIdentifier, BigUint)> {
        let stake_modifier = self.calculate_stake_modifier(attributes.rarity);
        let stake_modifier_total = self.stake_modifier_total().get();
        require!(stake_modifier_total >= (stake_modifier as u32), "Not enough supply");

        let last_reward_nonce = self.last_reward_block_nonce().get();
        let current_block_nonce = self.blockchain().get_block_nonce();

        let reward_increase = self.calculate_per_block_rewards(current_block_nonce, last_reward_nonce);
        let reward_per_share_increase =
            self.calculate_reward_per_share_increase(&reward_increase, &stake_modifier_total);

        let future_reward_per_share = self.reward_per_share().get() + reward_per_share_increase;

        let mut result: MultiValueEncoded<(EgldOrEsdtTokenIdentifier, BigUint)> = MultiValueEncoded::new();

        for token in self.all_reward_tokens().iter() {
            let reward_token = self.reward_tokens(&token).get();

            // Prevent accumulation of too many rewards
            if current_block_nonce > reward_token.end_rewards_block {
                let token_reward_increase = self.calculate_per_block_rewards(reward_token.end_rewards_block, last_reward_nonce);
                let token_reward_per_share_increase =
                    self.calculate_reward_per_share_increase(&token_reward_increase, &stake_modifier_total);

                let token_future_reward_per_share = self.reward_per_share().get() + token_reward_per_share_increase;

                let amount = self.calculate_reward(
                    &BigUint::from(stake_modifier as u16),
                    attributes.staked_block,
                    &token_future_reward_per_share,
                    &attributes.reward_per_share,
                    &token,
                    &reward_token
                );

                result.push((token, amount));

                continue;
            }

            let amount = self.calculate_reward(
                &BigUint::from(stake_modifier as u16),
                attributes.staked_block,
                &future_reward_per_share,
                &attributes.reward_per_share,
                &token,
                &reward_token
            );

            result.push((token, amount));
        }

        return result;
    }

    #[view]
    fn get_stake_modifier_info(&self, rarities: MultiValueEncoded<u8>) -> MultiValueEncoded<u8> {
        let mut stake_modifiers = MultiValueEncoded::new();

        for rarity in rarities.into_iter() {
            stake_modifiers.push(self.calculate_stake_modifier(rarity));
        }

        return stake_modifiers;
    }

    #[view]
    fn get_max_referals_info(&self, address: &ManagedAddress) -> MultiValue2<u8, u8> {
        let staked_rarity = self.staked_rarity(address).get();
        let number_of_referals = self.number_of_referals(address).get();
        let max_referals = self.calculate_max_referals(staked_rarity);

        return MultiValue2::from((number_of_referals, max_referals));
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
    fn staked_rarity(&self, address: &ManagedAddress) -> SingleValueMapper<u16>;

    #[view]
    #[storage_mapper("referer")]
    fn referer(&self, address: &ManagedAddress) -> SingleValueMapper<ManagedAddress>;

    #[view]
    #[storage_mapper("numberOfReferals")]
    fn number_of_referals(&self, address: &ManagedAddress) -> SingleValueMapper<u8>;
}
