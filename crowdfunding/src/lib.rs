#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq, Clone, Copy)]
pub enum Status {
    FundingPeriod,
    Successful,
    Failed,
}

#[elrond_wasm_derive::contract]
pub trait Crowdfunding {
	#[init]
    fn init(&self, target: Self::BigUint, deadline: u64) {
        // let my_address: Address = self.blockchain().get_caller();
        // self.blockchain().get_owner_address(),
        self.set_target(target);
        self.set_deadline(deadline);
    }

    #[storage_set("target")]
    fn set_target(&self, target: Self::BigUint);

    #[view]
    #[storage_get("target")]
    fn get_target(&self) -> Self::BigUint;

    #[storage_set("deadline")]
    fn set_deadline(&self, deadline: u64);

    #[view]
    #[storage_get("deadline")]
    fn get_deadline(&self) -> u64;

    #[storage_set("deposit")]
    fn set_deposit(&self, donor: &Address, amount: Self::BigUint);

    #[view]
    #[storage_get("deposit")]
    fn get_deposit(&self, donor: &Address) -> Self::BigUint;

    #[view]
    fn status(&self) -> Status {
        if self.blockchain().get_block_timestamp() <= self.get_deadline() {
            Status::FundingPeriod
        } else if self.blockchain().get_sc_balance() >= self.get_target() {
            Status::Successful
        } else {
            Status::Failed
        }
    }

    #[payable("EGLD")]
    #[endpoint]
    fn fund(&self, #[payment] payment: Self::BigUint) -> SCResult<()> {
        require!(
            self.status() == Status::FundingPeriod,
            "cannot fund after deadline"
        );
        let caller = self.blockchain().get_caller();
        let mut deposit = self.get_deposit(&caller);
        deposit += payment;
        self.set_deposit(&caller, deposit);

        Ok(())
    }

    #[endpoint]
    fn claim(&self) -> SCResult<()> {
        match self.status() {
            Status::FundingPeriod => {
                sc_error!("cannot claim before deadline")
            },
            Status::Successful => {
                let caller = self.blockchain().get_caller();
                require!(
					caller == self.blockchain().get_owner_address(),
					"only owner can claim successful funding"
				);
                self.send().direct_egld(&caller, &self.blockchain().get_sc_balance(), b"funding success");
                Ok(())
            },
            Status::Failed => {
                let caller = self.blockchain().get_caller();
                let deposit = self.get_deposit(&caller);

                if deposit > 0 {
                    self.set_deposit(&caller, Self::BigUint::zero());
                    self.send().direct_egld(&caller, &deposit, b"reclaim failed funding");
                }

                Ok(())
            },
        }
    }
}
