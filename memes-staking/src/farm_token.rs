use crate::base;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const META_NUM_DECIMALS: usize = 0;

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
    pub rarity: u8,
    pub staker: ManagedAddress<M>,
    pub nft_nonce: u64,
    pub staked_block: u64,
    pub reward_per_share: BigUint<M>,
}

#[elrond_wasm::module]
pub trait FarmTokenModule:
    base::BaseModule
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

    fn mint_farm_token(
        &self,
        token_id: TokenIdentifier,
        attributes: &StakingFarmTokenAttributes<Self::Api>,
        update_total: Option<bool>
    ) -> EsdtTokenPayment<Self::Api> {
        let amount = BigUint::from(1u8);
        let new_nonce = self
            .send()
            .esdt_nft_create_compact(&token_id, &amount, attributes);

        if update_total.is_some() && update_total.unwrap() {
            self.stake_modifier_total().update(|x| *x += &BigUint::from(self.calculate_stake_modifier(attributes.rarity)));
        }

        EsdtTokenPayment::new(token_id, new_nonce, amount)
    }

    fn burn_farm_tokens(&self, token_id: &TokenIdentifier, nonce: u64, amount: &BigUint, rarity: Option<u8>) {
        self.send().esdt_local_burn(token_id, nonce, amount);

        if rarity.is_some() {
            self.stake_modifier_total().update(|x| *x -= &BigUint::from(self.calculate_stake_modifier(rarity.unwrap())));
        }
    }

    #[view(getFarmTokenId)]
    #[storage_mapper("farm_token_id")]
    fn farm_token(&self) -> NonFungibleTokenMapper<Self::Api>;

    #[view(getStakeModifierTotal)]
    #[storage_mapper("stake_modifier_total")]
    fn stake_modifier_total(&self) -> SingleValueMapper<BigUint>;
}
