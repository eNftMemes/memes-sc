#![no_std]
#![feature(generic_associated_types)]

use staking::TopMemeAttributes;
use farm_token::*;

elrond_wasm::imports!();

mod owner;
mod staking;
mod farm_token;
mod common_structs;
mod custom_rewards;

const NFT_AMOUNT: u32 = 1;

const BASE_STAKE_MODIFIER: u32 = 100; // 1x
const INCREMENT_STAKE_MODIFIER: u32 = 10; // 0.1x
const TOP_RARITY_STAKE_MODIFIER: u32 = 200; // 2x
const SUPER_RARE_STAKE_MODIFIER: u32 = 220; // 2.2x
const TOP_RARITY: u32 = 10;

pub const DEFAULT_MINUMUM_LOCK_EPOCHS: u8 = 3; // ~3 days

pub const DIVISION_SAFETY_CONSTANT: u32 = 1000000000;

#[elrond_wasm::contract]
pub trait StakingContract: owner::OwnerModule
    + elrond_wasm_modules::pause::PauseModule
    + farm_token::FarmTokenModule
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(&self, voting_contract: &ManagedAddress, auction_contract: &ManagedAddress, token_identifier_top: &TokenIdentifier) {
        self.voting_sc().set(voting_contract);
        self.auction_sc().set(auction_contract);
        self.token_identifier_top().set(token_identifier_top);

        self.minimum_lock_epochs().set_if_empty(DEFAULT_MINUMUM_LOCK_EPOCHS);

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

        // self.generate_aggregated_rewards();

        let attributes = StakingFarmTokenAttributes {
            // reward_per_share: self.reward_per_share().get(),
            staker: caller,
            nft_nonce: nonce
        };

        let farm_token = self.farm_token().get_token_id();

        let rarity = nft_attributes.rarity as u32;
        let mut amount = BASE_STAKE_MODIFIER;

        if TOP_RARITY == rarity {
            amount = TOP_RARITY_STAKE_MODIFIER;
        } else if TOP_RARITY < rarity {
            amount = SUPER_RARE_STAKE_MODIFIER;
        } else if 1 > rarity {
            amount = amount + INCREMENT_STAKE_MODIFIER * (rarity - 1);
        }

        let new_rokens = self.mint_farm_tokens(farm_token, BigUint::from(amount), &attributes);

        self.send().direct_esdt(&self.blockchain().get_caller(), &new_rokens.token_identifier, new_rokens.token_nonce, &new_rokens.amount);
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
}
