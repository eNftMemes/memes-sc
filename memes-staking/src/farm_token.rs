use crate::common_structs::{FarmTokenAttributes, Nonce};
use crate::owner;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const META_NUM_DECIMALS: usize = 0;

#[derive(ManagedVecItem, Clone)]
pub struct FarmToken<M: ManagedTypeApi> {
    pub payment: EsdtTokenPayment<M>,
    pub attributes: FarmTokenAttributes<M>,
}

#[derive(
    ManagedVecItem,
    TopEncode,
    TopDecode,
    NestedEncode,
    NestedDecode,
    TypeAbi,
    Clone,
    PartialEq,
    Debug,
)]
pub struct StakingFarmTokenAttributes<M: ManagedTypeApi> {
    pub reward_per_share: BigUint<M>,
    pub rarity: u8,
    pub staker: ManagedAddress<M>,
    pub nft_nonce: Nonce,
    pub staked_block: u64
}

pub const TOP_RARITY: u8 = 10;

const BASE_STAKE_MODIFIER: u8 = 100; // 1x
const INCREMENT_STAKE_MODIFIER: u8 = 10; // 0.1x
const TOP_RARITY_STAKE_MODIFIER: u8 = 200; // 2x
const SUPER_RARE_STAKE_MODIFIER: u8 = 225; // 2.25x

#[elrond_wasm::module]
pub trait FarmTokenModule:
    owner::OwnerModule
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(registerFarmToken)]
    fn register_farm_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer
    ) {
        let payment_amount = self.call_value().egld_value();
        self.farm_token().issue_and_set_all_roles(
            EsdtTokenType::Meta,
            payment_amount,
            token_display_name,
            token_ticker,
            META_NUM_DECIMALS,
            None,
        );
    }

    fn burn_farm_tokens_from_payments(&self, payments: &ManagedVec<EsdtTokenPayment<Self::Api>>) {
        let mut total_amount = BigUint::zero();
        for entry in payments.iter() {
            total_amount += &entry.amount;
            self.send()
                .esdt_local_burn(&entry.token_identifier, entry.token_nonce, &entry.amount);
        }

        self.stake_modifier_total().update(|x| *x -= total_amount);
    }

    fn mint_farm_token(
        &self,
        token_id: TokenIdentifier,
        attributes: &StakingFarmTokenAttributes<Self::Api>,
    ) -> EsdtTokenPayment<Self::Api> {
        let amount = BigUint::from(1u8);
        let new_nonce = self
            .send()
            .esdt_nft_create_compact(&token_id, &amount, attributes);
        self.stake_modifier_total().update(|x| *x += &BigUint::from(self.calculate_stake_modifier(attributes.rarity)));

        EsdtTokenPayment::new(token_id, new_nonce, amount)
    }

    fn calculate_stake_modifier(&self, rarity: u8) -> u8 {
        // Rare(10) NFT
        if TOP_RARITY == rarity {
            return TOP_RARITY_STAKE_MODIFIER;
        }

        // Super Rare NFT
        if TOP_RARITY < rarity {
            return SUPER_RARE_STAKE_MODIFIER;
        }

        // Rare(1-9) NFT
        return BASE_STAKE_MODIFIER + INCREMENT_STAKE_MODIFIER * (rarity - 1);
    }

    fn burn_farm_tokens(&self, token_id: &TokenIdentifier, nonce: Nonce, amount: &BigUint, rarity: u8) {
        self.send().esdt_local_burn(token_id, nonce, amount);
        self.stake_modifier_total().update(|x| *x -= &BigUint::from(self.calculate_stake_modifier(rarity)));
    }

    fn get_farm_token_attributes<T: TopDecode>(
        &self,
        token_id: &TokenIdentifier,
        token_nonce: u64,
    ) -> T {
        let token_info = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            token_id,
            token_nonce,
        );

        token_info.decode_attributes()
    }

    #[view(getFarmTokenId)]
    #[storage_mapper("farm_token_id")]
    fn farm_token(&self) -> NonFungibleTokenMapper<Self::Api>;
}
