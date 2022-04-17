elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::auction::*;

pub const AUCTION_TIME: u64 = 432000; // 5 days in seconds (time the owner of the NFT has for locking it)

// TODO: Update this
// #[derive(TopEncode, TopDecode, TypeAbi)]
// pub struct CustomMemeAttributes<M: ManagedTypeApi> {
//     pub category: ManagedBuffer<M>,
//     pub rarity: u8,
// }

#[elrond_wasm::module]
pub trait OwnerModule {
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
    fn add_custom_auction(&self, period: u64, nonce: u64) {
        let block_timestamp = self.blockchain().get_block_timestamp();

        require!(
            block_timestamp > period && block_timestamp - period < AUCTION_TIME,
            "Auction deadline has passed"
        );
        require!(!self.period_auctioned_memes(period).is_empty(), "Period auction does not exist");
        require!(self.period_meme_auction(period, nonce).is_empty(), "Auction for this period and nonce already exists");

        let bid_cut_percentage: u16 = self.bid_cut_percentage().get();
        let min_bid_start: BigUint = self.min_bid_start().get();
        let multiplier: u8 = 10;

        self.add_auction(&period, &bid_cut_percentage, &min_bid_start, &multiplier, &nonce);
    }

    // private

    fn add_auction(&self, period: &u64, bid_cut_percentage: &u16, min_bid_start: &BigUint, multiplier: &u8, nonce: &u64) {
        let mut min_bid: BigUint = BigUint::from(*multiplier);
        min_bid *= min_bid_start;

        let auction = Auction {
            min_bid,
            current_bid: BigUint::zero(),
            current_winner: ManagedAddress::zero(),
            bid_cut_percentage: *bid_cut_percentage,
            original_owner: ManagedAddress::zero(),
            ended: false,
        };

        self.period_meme_auction(*period, *nonce).set(&auction);
        self.period_auctioned_memes(*period).push(nonce);
    }

    // TODO
    // #[only_owner]
    // #[endpoint]
    // fn set_custom_attributes(&self, nonce: u64, category: ManagedBuffer, rarity: u8) {
    //     self.custom_attributes(nonce).set(
    //         &CustomMemeAttributes { category, rarity }
    //     );
    // }

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

    // TODO
    // #[view]
    // #[storage_mapper("customAttributes")]
    // fn custom_attributes(&self, nonce: u64) -> SingleValueMapper<CustomMemeAttributes<Self::Api>>;
}
