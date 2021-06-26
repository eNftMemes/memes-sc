#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm_derive::contract]
pub trait Adder {
	#[view(getSum)]
	#[storage_get("sum")]
	fn get_sum(&self) -> Self::BigUint;

	#[storage_set("sum")]
	fn set_sum(&self, sum: Self::BigUint);

	#[init]
	fn init(&self, initial_value: Self::BigUint) {
		self.set_sum(initial_value);
	}

	#[endpoint]
	fn add(&self, value: Self::BigUint) -> SCResult<()> {
		let mut sum = self.get_sum();
		sum += value;
		self.set_sum(sum);

		Ok(())
	}
}
