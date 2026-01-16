#![expect(unsafe_code, reason = "allow `unsafe` code to rely on `DataBound` bounds")]

use core::marker::PhantomData;
use core::fmt::{Debug, Formatter, Result as FmtResult};


trait Seal {}

#[expect(private_bounds, reason = "intentionally creating a sealed trait")]
pub trait DataVariance: Sized + Seal {
    type Data: ?Sized;
}

pub struct Invariant<D: ?Sized> {
    _invariant: PhantomData<fn(D) -> D>,
}

impl<D: ?Sized> Seal for Invariant<D> {}

impl<D: ?Sized> DataVariance for Invariant<D> {
    type Data = D;
}

impl<D: ?Sized> Clone for Invariant<D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<D: ?Sized> Copy for Invariant<D> {}

impl<D: ?Sized> Debug for Invariant<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Invariant").field(&self._invariant).finish()
    }
}

pub struct Covariant<'a, D: ?Sized> {
    _covariant: PhantomData<fn() -> (&'a (), D)>,
}

impl<D: ?Sized> Seal for Covariant<'_, D> {}

impl<D: ?Sized> DataVariance for Covariant<'_, D> {
    type Data = D;
}

impl<D: ?Sized> Clone for Covariant<'_, D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<D: ?Sized> Copy for Covariant<'_, D> {}

impl<D: ?Sized> Debug for Covariant<'_, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Covariant").field(&self._covariant).finish()
    }
}

/// Provide bounds for usage of [`DataSource`] and [`MutDataSource`].
///
/// # Safety
/// Only the two currently provided implementations are permitted. Unsafe code in this crate relies
/// on delicate reasoning about lifetimes that depends on [`DataBound`].
pub unsafe trait DataBound<Target: ?Sized>: DataVariance {}

// SAFETY: This impl is permitted. Unsafe code can reason that this is one of only two impls.
unsafe impl<D: ?Sized, Target: ?Sized> DataBound<Target> for Invariant<D> {}

// SAFETY: This impl is permitted. Unsafe code can reason that this is one of only two impls.
unsafe impl<'a, D: ?Sized, Target: ?Sized + 'a> DataBound<Target> for Covariant<'a, D> {}
