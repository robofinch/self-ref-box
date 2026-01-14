#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod self_ref;
mod data_source;
mod erase;

mod slot;
// mod self_bind_struct;


pub use self::{
    data_source::{CloneableDataSource, DataSource, MutableDataSource, Outlives},
    erase::{DefaultErase, EraseSelfRef, LifetimeErase},
    self_ref::{NeverExclusiveRef, NeverNoRef, NeverSharedRef, SelfRef},
    slot::SelfRefSlot,
};
