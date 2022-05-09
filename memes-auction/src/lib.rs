#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();

mod owner;

use owner::AUCTION_TIME;

mod auction;

use auction::*;

const PERCENTAGE_TOTAL: u64 = 10_000;
// 100%
const NFT_AMOUNT: u32 = 1;
const MAX_TOP_MEMES: usize = 10;

const BID_TIME: u64 = 345600; // 4 days in seconds (time users can bid on NFTs)

const ROYALTIES: u16 = 1000; // 10%

#[elrond_wasm::contract]
pub trait MemesAuction: owner::OwnerModule {
    #[init]
    fn init(&self, voting_contract: &ManagedAddress, token_identifier: &TokenIdentifier, min_bid_start: &BigUint) {
        self.voting_contract().set(voting_contract);
        self.token_identifier().set(token_identifier);
        self.min_bid_start().set(min_bid_start);

        if self.bid_cut_percentage().is_empty() {
            let bid_cut: u16 = 1000;
            self.bid_cut_percentage().set(&bid_cut);
        }
    }

    #[endpoint]
    fn start_auction(&self, period: u64, nfts: MultiValueEncoded<u64>) {
        let caller: ManagedAddress = self.blockchain().get_caller();
        require!(
			caller == self.voting_contract().get(),
			"Only voting contract can call this"
		);
        require!(
			nfts.len() <= MAX_TOP_MEMES,
			"There can't be more than 10 nfts"
		);

        let bid_cut_percentage: u16 = self.bid_cut_percentage().get();
        let min_bid_start: BigUint = self.min_bid_start().get();
        let mut multiplier: u8 = nfts.len() as u8;

        // 1st meme is the one on 1st place etc
        for nonce in nfts.into_iter() {
            self.add_auction(&period, &bid_cut_percentage, &min_bid_start, &multiplier, &nonce);

            let meme_rarity = self.meme_rarity(nonce);
            if meme_rarity.is_empty() || multiplier > meme_rarity.get() {
                let rarity: u8 = multiplier;
                meme_rarity.set(&rarity);
            }

            multiplier -= 1;
        }
    }

    #[payable("*")]
    #[endpoint]
    fn lock_token(
        &self,
        period: u64,
        #[payment_token] nft_type: TokenIdentifier,
        #[payment_nonce] nonce: u64,
        #[payment_amount] nft_amount: BigUint,
    ) {
        require!(nft_amount == NFT_AMOUNT, "Nft amount should be 1");

        let block_timestamp = self.blockchain().get_block_timestamp();

        require!(
            block_timestamp > period && block_timestamp - period < AUCTION_TIME,
            "Auction deadline has passed"
        );

        let token_identifier_top = self.token_identifier_top().get();

        require!(nft_type == self.token_identifier().get() || nft_type == token_identifier_top, "Nft is not of the correct type");

        let mut original_nonce: u64 = nonce;
        let mut auction: Auction<Self::Api>;

        if nft_type == token_identifier_top {
            let own_address: ManagedAddress = self.blockchain().get_sc_address();
            let token_data: EsdtTokenData<Self::Api> = self.blockchain().get_esdt_token_data(&own_address, &nft_type, nonce);
            let attributes = token_data.decode_attributes::<TopMemeAttributes<Self::Api>>();

            original_nonce = attributes.original_nonce;

            auction = self.try_get_auction(period, original_nonce);

            auction.top_nonce = nonce;
        } else {
            auction = self.try_get_auction(period, original_nonce);
        }

        let caller = self.blockchain().get_caller();

        auction.original_owner = caller;

        self.period_meme_auction(period, original_nonce).set(&auction);
    }

    #[payable("EGLD")]
    #[endpoint]
    fn bid(
        &self,
        #[payment_amount] payment_amount: BigUint,
        period: u64,
        nonce: u64,
    ) {
        let mut auction = self.try_get_auction(period, nonce);
        let caller = self.blockchain().get_caller();
        let block_timestamp = self.blockchain().get_block_timestamp();

        require!(
			block_timestamp > period && block_timestamp - period < BID_TIME,
            "Auction bidding ended already"
        );
        require!(
            auction.original_owner != caller,
            "Can't bid on your own token"
        );
        require!(auction.current_winner != caller, "Can't outbid yourself");
        require!(
            payment_amount >= auction.min_bid,
            "Bid must be higher than or equal to the min bid"
        );
        require!(
            payment_amount > auction.current_bid,
            "Bid must be higher than the current winning bid"
        );

        let mut funds = self.auction_funds().get();

        // refund losing bid
        if !auction.current_winner.is_zero() {
            funds = &funds - &auction.current_bid;

            self.send().direct_egld(
                &auction.current_winner,
                &auction.current_bid,
                b"bid refund",
            );
        }

        funds = &funds + &payment_amount;

        // update auction bid and winner
        auction.current_bid = payment_amount;
        auction.current_winner = caller;

        self.period_meme_auction(period, nonce).set(&auction);

        self.auction_funds().set(&funds);
    }

    #[endpoint]
    fn end_auction(&self, period: u64, nonce: u64) {
        let mut auction = self.try_get_auction(period, nonce);
        let block_timestamp = self.blockchain().get_block_timestamp();

        require!(
            block_timestamp > period && block_timestamp - period >= AUCTION_TIME,
            "Auction deadline has not passed"
        );
        require!(
			!auction.ended,
			"Auction was already ended"
		);

        auction.ended = true;

        self.period_meme_auction(period, nonce).set(&auction);

        self.distribute_tokens_after_auction_end(&nonce, &auction);
    }

    #[payable("*")]
    #[endpoint]
    fn upgrade_token(
        &self,
        #[payment_token] nft_type: TokenIdentifier,
        #[payment_nonce] nonce: u64,
        #[payment_amount] nft_amount: BigUint,
    ) {
        require!(nft_amount == NFT_AMOUNT, "Nft amount should be 1");
        require!(nft_type == self.token_identifier().get() || nft_type == self.token_identifier_top().get(), "Nft is not of the correct type");

        if nft_type == self.token_identifier_top().get() {
            let own_address: ManagedAddress = self.blockchain().get_sc_address();
            let token_data: EsdtTokenData<Self::Api> = self.blockchain().get_esdt_token_data(&own_address, &nft_type, nonce);
            let attributes = token_data.decode_attributes::<TopMemeAttributes<Self::Api>>();

            let original_nonce: u64 = attributes.original_nonce;

            require!(!self.meme_rarity(original_nonce).is_empty(), "Nft can't be upgraded");

            self.update_nft_attributes(
                &self.blockchain().get_caller(),
                &original_nonce,
                &nonce,
                b"nft upgraded",
            );

            return;
        }

        self.convert_to_top_nft(
            &self.blockchain().get_caller(),
            &nonce,
            b"nft upgraded",
        );
    }

    // private

    fn try_get_auction(&self, period: u64, nonce: u64) -> Auction<Self::Api> {
        let auction = self.period_meme_auction(period, nonce);

        require!(
            !auction.is_empty(),
            "Auction does not exist"
        );

        auction.get()
    }

    fn distribute_tokens_after_auction_end(&self, nft_nonce: &u64, auction: &Auction<Self::Api>) {
        let mut funds = self.auction_funds().get();

        if auction.original_owner.is_zero() {
            if auction.current_winner.is_zero() {
                return;
            }

            funds = &funds - &auction.current_bid;

            self.auction_funds().set(funds);

            // return money to current winner
            self.send().direct_egld(
                &auction.current_winner,
                &auction.current_bid,
                b"bid refund",
            );

            return;
        }

        if auction.current_winner.is_zero() {
            // return nft to original owner

            if auction.top_nonce != 0 {
                self.update_nft_attributes(&auction.original_owner, nft_nonce, &auction.top_nonce, b"returned token");

                return;
            }

            self.convert_to_top_nft(&auction.original_owner, nft_nonce, b"returned token");

            return;
        }

        funds = &funds - &auction.current_bid;

        self.auction_funds().set(&funds);

        let bid_cut_percentage = BigUint::from(auction.bid_cut_percentage);
        let bid_cut = &auction.current_bid * &bid_cut_percentage / PERCENTAGE_TOTAL;
        let mut owner_cut = auction.current_bid.clone();
        owner_cut -= &bid_cut;

        // send part as cut for contract owner
        let owner = self.blockchain().get_owner_address();
        self.send().direct_egld(
            &owner,
            &bid_cut,
            b"bid cut for sold token",
        );

        // send rest of the bid to original owner
        self.send().direct_egld(
            &auction.original_owner,
            &owner_cut,
            b"sold token",
        );

        if auction.top_nonce != 0 {
            self.update_nft_attributes(&auction.current_winner, nft_nonce, &auction.top_nonce, b"bought token at auction");

            return;
        }

        // send NFT to auction winner
        self.convert_to_top_nft(&auction.current_winner, nft_nonce, b"bought token at auction");
    }

    fn convert_to_top_nft(&self, send_to: &ManagedAddress, nft_nonce: &u64, text: &[u8]) {
        let meme_rarity = self.meme_rarity(*nft_nonce);

        require!(!meme_rarity.is_empty(), "Meme rarity is empty");

        let nft_token = &self.token_identifier().get();
        let nft_token_top = &self.token_identifier_top().get();
        let amount = BigUint::from(NFT_AMOUNT);

        let own_address: ManagedAddress = self.blockchain().get_sc_address();
        let token_data: EsdtTokenData<Self::Api> = self.blockchain().get_esdt_token_data(&own_address, nft_token, *nft_nonce);
        let attributes = token_data.decode_attributes::<MemeAttributes<Self::Api>>();

        let top_royalties: &BigUint = &BigUint::from(ROYALTIES);

        let rarity = meme_rarity.get();
        meme_rarity.clear();

        // Set custom category if set
        let mut category = attributes.category;
        let custom_meme_category = self.custom_meme_category(*nft_nonce);
        if !custom_meme_category.is_empty() {
            category = custom_meme_category.get();
            custom_meme_category.clear();
        }

        let new_attributes = TopMemeAttributes {
            rarity,
            original_nonce: *nft_nonce,
            period: attributes.period,
            category,
            creator: attributes.creator,
        };

        let nft_nonce_top: u64 = self.send().esdt_nft_create(
            nft_token_top,
            &amount,
            &token_data.name,
            top_royalties,
            &token_data.hash,
            &new_attributes,
            &token_data.uris,
        );

        self.meme_to_top_meme(*nft_nonce).set(nft_nonce_top);

        self.send().esdt_local_burn(nft_token, *nft_nonce, &amount);

        self.send().direct(
            send_to,
            nft_token_top,
            nft_nonce_top,
            &amount,
            text,
        );
    }

    fn update_nft_attributes(&self, send_to: &ManagedAddress, original_nonce: &u64, nft_nonce: &u64, text: &[u8]) {
        let nft_token = &self.token_identifier_top().get();
        let amount = BigUint::from(NFT_AMOUNT);

        let own_address: ManagedAddress = self.blockchain().get_sc_address();
        let token_data: EsdtTokenData<Self::Api> = self.blockchain().get_esdt_token_data(&own_address, nft_token, *nft_nonce);
        let mut new_attributes = token_data.decode_attributes::<TopMemeAttributes<Self::Api>>();

        let mut attributes_updated = false;

        let meme_rarity = self.meme_rarity(*original_nonce);

        if !meme_rarity.is_empty() && meme_rarity.get() > new_attributes.rarity {
            new_attributes.rarity = meme_rarity.get();

            attributes_updated = true;
        }

        // Set custom category if set
        let custom_meme_category = self.custom_meme_category(*original_nonce);
        if !custom_meme_category.is_empty() {
            new_attributes.category = custom_meme_category.get();
            custom_meme_category.clear();

            attributes_updated = true;
        }

        if attributes_updated {
            self.send().nft_update_attributes(
                nft_token,
                *nft_nonce,
                &new_attributes,
            );
        }

        meme_rarity.clear();

        self.send().direct(
            send_to,
            nft_token,
            *nft_nonce,
            &amount,
            text,
        );
    }

    // views/storage

    #[view]
    fn period_auctions_memes_all(&self, period: u64) -> MultiValueEncoded<FullAuction<Self::Api>> {
        let period_auctioned_memes = self.period_auctioned_memes(period);

        let mut result: MultiValueEncoded<FullAuction<Self::Api>> = MultiValueEncoded::new();
        for index in 1..=period_auctioned_memes.len() {
            let nonce = period_auctioned_memes.get(index);
            let auction = self.period_meme_auction(period, nonce).get();

            let full_auction = FullAuction {
                nonce,
                auction,
            };

            result.push(full_auction);
        }

        return result;
    }

    #[view]
    #[storage_mapper("votingContract")]
    fn voting_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[view]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;
}
