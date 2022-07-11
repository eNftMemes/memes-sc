#![no_std]
#![feature(generic_associated_types)]

use staking::TopMemeAttributes;
use farm_token::*;
use crate::common_structs::Nonce;

elrond_wasm::imports!();

mod owner;
mod staking;
mod farm_token;
mod common_structs;
mod custom_rewards;

const NFT_AMOUNT: u32 = 1;

const BASE_STAKE_MODIFIER: u16 = 100; // 1x
const INCREMENT_STAKE_MODIFIER: u16 = 10; // 0.1x
const TOP_RARITY_STAKE_MODIFIER: u16 = 200; // 2x
const SUPER_RARE_STAKE_MODIFIER: u16 = 220; // 2.2x
const TOP_RARITY: u16 = 10;

const DEFAULT_MINUMUM_LOCK_BLOCKS: u64 = 3 * 14_400; // ~3 days (3 * 14400 blocks)

const DIVISION_SAFETY_CONSTANT: u32 = 1000000000;

const BASE_REFERER_PERSONS: u16 = 15;
const INCREMENT_REFERER_PERSONS: u16 = 5;
const SUPER_RARE_REFERER_PERSONS: u16 = 100;


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

        let caller = self.blockchain().get_caller();
        let staked_rarity = self.staked_rarity(&caller);

        require!(staked_rarity.is_empty(), "Address already has an NFT staked");

        let own_address: ManagedAddress = self.blockchain().get_sc_address();
        let token_data: EsdtTokenData<Self::Api> = self.blockchain().get_esdt_token_data(
            &own_address,
            &nft_type.into_esdt_option().unwrap(),
            nonce
        );
        let nft_attributes = token_data.decode_attributes::<TopMemeAttributes<Self::Api>>();

        staked_rarity.set(nft_attributes.rarity);

        self.generate_aggregated_rewards();

        let farm_token = self.farm_token().get_token_id();

        let rarity = nft_attributes.rarity as u16;
        let mut amount = BASE_STAKE_MODIFIER;

        if TOP_RARITY == rarity {
            amount = TOP_RARITY_STAKE_MODIFIER;
        } else if TOP_RARITY < rarity {
            amount = SUPER_RARE_STAKE_MODIFIER;
        } else if 1 > rarity {
            amount = amount + INCREMENT_STAKE_MODIFIER * (rarity - 1);
        }

        let attributes = StakingFarmTokenAttributes {
            reward_per_share: self.reward_per_share().get(),
            current_farm_amount: BigUint::from(amount),
            staker: caller,
            nft_nonce: nonce,
            staked_block: self.blockchain().get_block_nonce(),
        };

        let new_rokens = self.mint_farm_tokens(farm_token, BigUint::from(amount), &attributes);

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

        require!(payment_amount == farm_attributes.current_farm_amount, "Can only unstake the whole farm amount");
        require!(
            self.blockchain().get_block_nonce() > farm_attributes.staked_block + self.minimum_lock_blocks().get(),
            "Minimum lock time has not yet passed"
        );

        self.generate_aggregated_rewards();

        let caller = self.blockchain().get_caller();
        self.burn_farm_tokens(&token_identifier, token_nonce, &payment_amount);

        self.staked_rarity(&farm_attributes.staker).clear();

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
        let rarity = staked_rarity.get() as u16;

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
        amount: BigUint,
        attributes: StakingFarmTokenAttributes<Self::Api>,
    ) -> BigUint {
        require!(amount > 0, "Zero liquidity input");
        let farm_token_supply = self.farm_token_supply().get();
        require!(farm_token_supply >= amount, "Not enough supply");

        let last_reward_nonce = self.last_reward_block_nonce().get();
        let current_block_nonce = self.blockchain().get_block_nonce();

        let reward_increase =
            self.calculate_per_block_rewards(current_block_nonce, last_reward_nonce);

        let reward_per_share_increase =
            self.calculate_reward_per_share_increase(&reward_increase, &farm_token_supply);

        let future_reward_per_share = self.reward_per_share().get() + reward_per_share_increase;

        self.calculate_reward(
            &amount,
            &future_reward_per_share,
            &attributes.reward_per_share,
        )
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
    fn number_of_referals(&self, address: &ManagedAddress) -> SingleValueMapper<u16>;
}
