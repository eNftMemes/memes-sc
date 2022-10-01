elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const MAX_REFERER_VOTES: u8 = 45;

const DEFAULT_VOTES_PER_ADDRESS_PER_PERIOD: u8 = 10; // TODO: Modify this to 5 in the future?

pub const EXTRA_VOTES_IF_REFERRED: u8 = 15;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct AddressVotes {
    pub period: u64,
    pub votes: u8,
}

#[elrond_wasm::module]
pub trait BaseModule {
    fn calculate_extra_votes_for_referers(&self, number_of_referals: u8) -> u8 {
        // Safety check, should not happen
        if 0 == number_of_referals {
            return 0;
        }

        if MAX_REFERER_VOTES < number_of_referals {
            return MAX_REFERER_VOTES;
        }

        return number_of_referals;
    }

    fn get_address_total_votes(&self, address: &ManagedAddress) -> u8 {
        return DEFAULT_VOTES_PER_ADDRESS_PER_PERIOD + self.address_extra_votes_per_period(address).get();
    }

    fn add_address_votes(&self, address: &ManagedAddress, votes: u8) {
        let address_votes = self.address_votes(&address);
        let address_extra_votes = self.address_extra_votes_per_period(address);

        if !address_votes.is_empty() {
            let address_votes_value: AddressVotes = address_votes.get();

            // Add extra votes to address who already voted
            address_votes.set(&AddressVotes {
                period: address_votes_value.period,
                votes: votes - address_extra_votes.get() + address_votes_value.votes,
            });
        }

        address_extra_votes.set(votes);
    }

    #[view]
    #[storage_mapper("addressVotes")]
    fn address_votes(&self, address: &ManagedAddress) -> SingleValueMapper<AddressVotes>;

    #[view]
    #[storage_mapper("addressExtraVotesPerPeriod")]
    fn address_extra_votes_per_period(&self, address: &ManagedAddress) -> SingleValueMapper<u8>;
}
