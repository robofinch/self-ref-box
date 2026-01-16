// See https://linebender.org/blog/doc-include for this README inclusion strategy
// File links are not supported by rustdoc
//!
//! [LICENSE-APACHE]: https://github.com/robofinch/self-ref-box/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/self-ref-box/blob/main/LICENSE-MIT
//!
//!
//! <style>
//! .rustdoc-hidden { display: none; }
//! </style>
#![cfg_attr(doc, doc = include_str!("../README.md"))]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod erase;
mod variance;
mod uninhabited_ref;

mod slot;
mod self_bind_struct;


pub use self::{
    erase::{DefaultErase, EraseSelfRef, LifetimeErase},
    slot::SelfRefSlot,
    uninhabited_ref::{NeverExclusiveRef, NeverNoRef, NeverSharedRef},
    variance::{Covariant, DataBound, DataVariance, Invariant},
};
