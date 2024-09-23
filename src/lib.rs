#![no_std]

#[macro_use]
extern crate alloc;

pub mod coffer_inst;
pub mod coffer_call;
pub mod message_call;

pub(crate) mod memory;
pub(crate) mod enclave_id;
pub(crate) mod vcpu;
pub(crate) mod enclave;
