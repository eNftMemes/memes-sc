elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::auction::*;

pub const AUCTION_TIME: u64 = 432000; // 5 days in seconds (time the owner of the NFT has for locking it)

#[elrond_wasm::module]
pub trait OwnerModule {
    #[endpoint]
    #[only_owner]
    #[payable("EGLD")]
    fn issue_token(
        &self,
        #[payment] issue_cost: BigUint,
        token_name: &ManagedBuffer,
        token_ticker: &ManagedBuffer,
    ) {
        require!(self.token_identifier_top().is_empty(), "Token already issued");

        self.send()
            .esdt_system_sc_proxy()
            .issue_non_fungible(
                issue_cost,
                token_name,
                token_ticker,
                NonFungibleTokenProperties {
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_change_owner: false,
                    can_upgrade: false,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().init_callback())
            .call_and_exit()
    }

    #[endpoint]
    #[only_owner]
    fn set_local_roles(&self) {
        require!(!self.token_identifier_top().is_empty(), "Token not issued");

        self
            .send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.token_identifier_top().get(),
                (&[EsdtLocalRole::NftCreate, EsdtLocalRole::NftUpdateAttributes][..]).iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[callback]
    fn init_callback(&self, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.token_identifier_top().set(&token_id);
            }
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_owner_address();
                let returned = self.call_value().egld_or_single_esdt();
                if returned.token_identifier.is_egld() && returned.amount > 0 {
                    self.send()
                        .direct(&caller, &returned.token_identifier, 0, &returned.amount);
                }
            }
        }
    }

    #[only_owner]
    #[endpoint]
    fn set_bid_cut_percentage(&self, bid_cut: u16) {
        require!(bid_cut > 100 && bid_cut < 2500, "Bid cut percentage can not be less than 1% and greater than 25%");

        self.bid_cut_percentage().set(&bid_cut);
    }

    #[only_owner]
    #[endpoint]
    fn set_min_bid_start(&self, min_bid_start: &BigUint) {
        self.min_bid_start().set(min_bid_start);
    }

    #[only_owner]
    #[endpoint]
    fn claim_royalties(&self) {
        let caller = self.blockchain().get_caller();

        let mut to_send = self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);

        to_send = &to_send - &self.auction_funds().get();

        self.send().direct_egld(&caller, &to_send);
    }

    #[only_owner]
    #[endpoint]
    fn set_custom_attributes(&self, nonce: u64, category: &ManagedBuffer, rarity: &u8) {
        require!(*rarity > 10, "Rarity needs to be higher than 10");

        let meme_rarity = self.meme_rarity(nonce);
        let custom_meme_category = self.custom_meme_category(nonce);

        require!(
            meme_rarity.is_empty() && custom_meme_category.is_empty(),
            "Meme rarity and custom category needs to be empty"
        );

        meme_rarity.set(rarity);
        custom_meme_category.set(category);
    }

    #[only_owner]
    #[endpoint]
    fn add_custom_auction(&self, period: u64, nonce: u64, multiplier_opt: OptionalValue<u8>) {
        let block_timestamp = self.blockchain().get_block_timestamp();

        require!(
            block_timestamp - core::cmp::min(block_timestamp, period) < AUCTION_TIME,
            "Auction deadline has passed"
        );
        require!(self.period_meme_auction(period, nonce).is_empty(), "Auction for this period and nonce already exists");

        // If is custom auction for a non existent period
        if self.period_auctioned_memes(period).is_empty() {
            let mut custom_auction_periods = self.custom_auction_periods();

            // Only add new custom period if it doesn't exist yet
            if custom_auction_periods.is_empty() {
                custom_auction_periods.push(&period);
            } else {
                let last_custom_period = custom_auction_periods.get(custom_auction_periods.len());

                if last_custom_period != period {
                    custom_auction_periods.push(&period);
                }
            }
        }

        let bid_cut_percentage: u16 = self.bid_cut_percentage().get();
        let min_bid_start: BigUint = self.min_bid_start().get();
        // 0.5 EGLD default if min bid is 0.025 EGLD
        let multiplier: u8 = if let OptionalValue::Some(multiplier_val) = multiplier_opt { multiplier_val } else { 20 };

        self.add_auction(&period, &bid_cut_percentage, &min_bid_start, &multiplier, &nonce);
    }

    // private

    fn add_auction(&self, period: &u64, bid_cut_percentage: &u16, min_bid_start: &BigUint, multiplier: &u8, nonce: &u64) {
        let mut min_bid: BigUint = BigUint::from(*multiplier);
        min_bid *= min_bid_start;

        let mut top_nonce: u64 = 0;

        if !self.meme_to_top_meme(*nonce).is_empty() {
            top_nonce = self.meme_to_top_meme(*nonce).get();
        }

        let auction = Auction {
            min_bid,
            current_bid: BigUint::zero(),
            current_winner: ManagedAddress::zero(),
            bid_cut_percentage: *bid_cut_percentage,
            original_owner: ManagedAddress::zero(),
            ended: false,
            top_nonce,
        };

        self.period_meme_auction(*period, *nonce).set(&auction);
        self.period_auctioned_memes(*period).push(nonce);
    }

    // views/storage

    #[view]
    #[storage_mapper("bidCutPercentage")]
    fn bid_cut_percentage(&self) -> SingleValueMapper<u16>;

    #[view]
    #[storage_mapper("minBidStart")]
    fn min_bid_start(&self) -> SingleValueMapper<BigUint>;

    #[view]
    #[storage_mapper("periodMemeAuction")]
    fn period_meme_auction(&self, period: u64, nonce: u64) -> SingleValueMapper<Auction<Self::Api>>;

    #[view]
    #[storage_mapper("periodAuctionedMemes")]
    // TODO: Remove this if data is indexed on microservice side?
    fn period_auctioned_memes(&self, period: u64) -> VecMapper<u64>;

    #[view]
    #[storage_mapper("tokenIdentifierTop")]
    fn token_identifier_top(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view]
    #[storage_mapper("memeToTopMeme")]
    fn meme_to_top_meme(&self, nonce: u64) -> SingleValueMapper<u64>;

    #[storage_mapper("auctionFunds")]
    fn auction_funds(&self) -> SingleValueMapper<BigUint>;

    // The rarity of a meme depending on the place the meme was in an auction, to be used in the future
    // If an auction has less than 10 memes, the max rarity is < 10
    // 10 - 1st place, most rare
    // 1 - 10th place, most common
    #[view]
    #[storage_mapper("memeRarity")]
    fn meme_rarity(&self, nonce: u64) -> SingleValueMapper<u8>;

    #[view]
    #[storage_mapper("customMemeCategory")]
    fn custom_meme_category(&self, nonce: u64) -> SingleValueMapper<ManagedBuffer>;

    #[view]
    #[storage_mapper("customAuctionPeriods")]
    fn custom_auction_periods(&self) -> VecMapper<u64>;
}
