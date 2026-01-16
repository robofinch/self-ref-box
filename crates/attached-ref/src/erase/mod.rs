#![expect(unsafe_code, reason = "erasing lifetimes requires unsafe")]

mod lifetime_erase;
mod layout_erase;
mod heap_erase;


use variance_family::LendFamily;

use crate::slot::SelfRefSlot;

pub use self::lifetime_erase::LifetimeErase;


/// A good default implementation for [`EraseSelfRef`].
///
/// It requires no additional lifetime or type parameters and stores data inline rather than
/// requiring an additional allocation, but in exchange, it does not work for every possible `S`
/// and `E` types; `<S as WithLifetime<'static, 'lower, Upper>>::Is` must be well-formed for any
/// `'lower` and some `'static` `Upper` bound (and likewise for `E`).
///
/// In particular, [`DefaultErase`] would not work if `S<'varying>` is `&'varying &'a u8` for some
/// non-`'static` lifetime `'a`. However, it is sufficient for most common cases.
///
/// Turn to other options only if [`DefaultErase`] is not sufficient.
pub type DefaultErase<N, S, E> = LifetimeErase<'static, N, S, E>;


/// Erase the `'varying` lifetime of a [`SelfRefSlot`].
///
/// # Safety
/// The methods must be implemented correctly. In particular, up to lifetimes,
/// - `unerase(erase(slot))` must always return `slot`,
/// - `erase(unerase(erased))` must always return `erased`,
/// - `unerase_ref(&erase(slot))` must always return `&slot`,
/// - `unerase_mut(&mut erase(slot))` must always return `&mut slot`.
///
/// and global state or other means must not be abused to cause incorrect behavior when
/// the methods are used in conjunction with each other.
pub unsafe trait EraseSelfRef<N, S, E>
where
    S: LendFamily<Self::Upper>,
    E: LendFamily<Self::Upper>,
{
    /// An upper bound for `'varying` required of the `S` and `E` lifetime families.
    type Upper: ?Sized;

    /// Erase the `'varying` lifetime of a [`SelfRefSlot`].
    ///
    /// The returned value is safe to use (and therefore must not support much beyond
    /// being moved or dropped, requiring unsafe unerasure to be useful).
    ///
    /// # Safety
    /// It must be sound to drop the returned `Self` value (when it is dropped, if ever).
    /// The destructor is permitted to unerase the `Self` value to
    /// `SelfRefSlot<'within_drop_function, N, S, E, Self::Upper>` for a
    /// `'within_drop_function` lifetime which is limited to the body of a `Drop::drop` impl. All
    /// validity and soundness burdens for that `'within_drop_function` lifetime fall on the caller
    /// of this function.
    unsafe fn erase(slot: SelfRefSlot<'_, N, S, E, Self::Upper>) -> Self;

    /// Return a `'varying` lifetime to an erased [`SelfRefSlot`].
    ///
    /// # Safety
    /// This function, in conjunction with [`EraseSelfRef::erase`], can arbitrarily transmute a
    /// `'varying` lifetime. All validity and soundness burdens for that `'varying` lifetime
    /// fall on the caller of this function.
    ///
    /// For instance, `SelfRefSlot<'varying, N, S, E, Self::Upper>` should generally not contain
    /// dangling references.
    unsafe fn unerase<'varying: 'varying>(
        slot: Self,
    ) -> SelfRefSlot<'varying, N, S, E, Self::Upper>
    where
        Self::Upper: 'varying;

    /// Return a `'varying` lifetime to an erased [`SelfRefSlot`] behind a reference.
    ///
    /// # Safety
    /// This function, in conjunction with [`EraseSelfRef::erase`], can arbitrarily transmute a
    /// `'varying` lifetime. All validity and soundness burdens for that `'varying` lifetime
    /// fall on the caller of this function.
    ///
    /// For instance, `SelfRefSlot<'varying, N, S, E, Self::Upper>` should generally not contain
    /// dangling references.
    unsafe fn unerase_ref<'varying: 'varying>(
        slot: &Self,
    ) -> &SelfRefSlot<'varying, N, S, E, Self::Upper>
    where
        Self::Upper: 'varying;

    /// Return a `'varying` lifetime to an erased [`SelfRefSlot`] behind a mutable reference.
    ///
    /// # Safety
    /// This function, in conjunction with [`EraseSelfRef::erase`], can arbitrarily transmute a
    /// `'varying` lifetime. All validity and soundness burdens for that `'varying` lifetime
    /// fall on the caller of this function.
    ///
    /// For instance, `SelfRefSlot<'varying, N, S, E, Self::Upper>` should generally not contain
    /// dangling references.
    unsafe fn unerase_mut<'varying: 'varying>(
        slot: &mut Self,
    ) -> &mut SelfRefSlot<'varying, N, S, E, Self::Upper>
    where
        Self::Upper: 'varying;
}
