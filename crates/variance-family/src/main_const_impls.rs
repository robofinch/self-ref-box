use core::mem::transmute;

use crate::invariant_zst;
use crate::traits::{ContravariantFamily, CovariantFamily, LifetimeFamily, Varying, WithLifetime};


// We *could* use the public macros to implement everything besides the function pointer cases,
// but I think it's valuable to show the full `unsafe` code for at least the most crucial cases.

// Note: in below safety comments, "is covariant in" or "is contravariant in" means, more
// precisely, "can be covariantly casted in" or "can be contravariantly casted in".

// ================================================================
//  &'a T
// ================================================================

// Safety summary:
// - `'varying` is covariant in `&'a T<'varying>` if it's covariant in `T<'varying>`.
// - it's contravariant in `&'a T<'varying>` if it's contravariant in `T<'varying>`.

impl<'a, 'varying, 'lower, 'upper, T> WithLifetime<'varying, 'lower, 'upper> for &'a T
where
    T: ?Sized + WithLifetime<'varying, 'lower, 'upper>,
    T::Is: 'a,
{
    type Is = &'a T::Is;
}

impl<'a, 'lower, 'upper, T> LifetimeFamily<'lower, 'upper> for &'a T
where
    T: ?Sized + LifetimeFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'a,
{}

// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `'varying` is covariant in `Self<'varying>`.
//
//   The former implies that `T::covariant_assertions()` does not panic,
//   in which case `'varying` is covariant in `T<'varying>`,
//   implying that `'varying` is covariant in `&'a T<'varying>`.
//
// - No assertions are included other than those in `Self::covariant_assertions()`.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'a, 'lower, 'upper, T> CovariantFamily<'lower, 'upper> for &'a T
where
    T: ?Sized + CovariantFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'a,
{
    #[inline]
    fn covariant_assertions() {
        T::covariant_assertions();
    }

    #[inline]
    fn shorten<'l, 's>(
        long: Varying<'l, 'lower, 'upper, Self>,
    ) -> Varying<'s, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, 'upper, Self>: Sized,
    {
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is just a covariant cast with, possibly, some assertions.
        // Any possible sources of panics in `T::shorten_ref` must be included in
        // `T::covariant_assertions`, which are included in `Self::covariant_assertions`.

        T::shorten_ref(long)
    }

    #[inline]
    fn shorten_ref<'l, 's, 'r>(
        long: &'r Varying<'l, 'lower, 'upper, Self>,
    ) -> &'r Varying<'s, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'l, 'lower, 'upper, Self>: 'r,
        Varying<'s, 'lower, 'upper, Self>: 'r,
    {
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a covariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.

        Self::covariant_assertions();

        let src: &'r &'a Varying<'l, 'lower, 'upper, T> = long;
        // SAFETY: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which is
        // at least as long as `'lower`. We called `T::covariant_assertions()` (within
        // `Self::covariant_assertions()`), so covariantly casting `T<'varying>` is sound.
        let dst: &'r &'a Varying<'s, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
    }
}

// SAFETY:
// - If `Self::contravariant_assertions()` does not panic,
//   then `'varying` is contravariant in `Self<'varying>`.
//
//   The former implies that `T::contravariant_assertions()` does not panic,
//   in which case `'varying` is contravariant in `T<'varying>`,
//   implying that `'varying` is contravariant in `&'a T<'varying>`.
//
// - No assertions are included other than those in `Self::contravariant_assertions()`.
// - The implementation safety requirements of `lengthen` and `lengthen_ref` are met.
unsafe impl<'a, 'lower, 'upper, T> ContravariantFamily<'lower, 'upper> for &'a T
where
    T: ?Sized + ContravariantFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'a,
{
    #[inline]
    fn contravariant_assertions() {
        T::contravariant_assertions();
    }

    #[inline]
    fn lengthen<'s, 'l>(
        short: Varying<'s, 'lower, 'upper, Self>,
    ) -> Varying<'l, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, 'upper, Self>: Sized,
    {
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is just a contravariant cast with, possibly, some assertions.
        // Any possible sources of panics in `T::lengthen_ref` must be included in
        // `T::contravariant_assertions`, which are included in `Self::contravariant_assertions`.

        T::lengthen_ref(short)
    }

    #[inline]
    fn lengthen_ref<'s, 'l, 'r>(
        short: &'r Varying<'s, 'lower, 'upper, Self>,
    ) -> &'r Varying<'l, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'l, 'lower, 'upper, Self>: 'r,
        Varying<'s, 'lower, 'upper, Self>: 'r,
    {
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a contravariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::contravariant_assertions()`
        // call.

        Self::contravariant_assertions();

        let src: &'r &'a Varying<'s, 'lower, 'upper, T> = short;
        // SAFETY: we are lengthening the `'s` lifetime of `T<'s>` to `'l`, which is
        // at most as long as `'upper`. We called `T::contravariant_assertions()` (within
        // `Self::contravariant_assertions()`), so contravariantly casting `T<'varying>` is sound.
        let dst: &'r &'a Varying<'l, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
    }
}


// ================================================================
//  &'varying T    (VaryingRef<T>)
// ================================================================

// Safety summary:
// - `'varying` is covariant in `&'varying T<'varying>` if it's covariant in `T<'varying>`
// - `'varying` is never contravariant in `&'varying T<'varying>`.

invariant_zst!(
    /// The `&'varying T<'varying>` lifetime family.
    ///
    /// If `'varying` is covariant in `T<'varying>`, then `'varying` is covariant in
    /// `&'varying T<'varying>`.
    ///
    /// `'varying` is never contravariant in this lifetime family.
    ///
    /// Note that this type itself is just a marker ZST for the family.
    pub struct VaryingRef<T: ?Sized>;
);

impl<'varying, 'lower, 'upper, T> WithLifetime<'varying, 'lower, 'upper> for VaryingRef<T>
where
    T: ?Sized + WithLifetime<'varying, 'lower, 'upper>,
    T::Is: 'varying,
{
    type Is = &'varying T::Is;
}

impl<'lower, 'upper, T> LifetimeFamily<'lower, 'upper> for VaryingRef<T>
where
    T: ?Sized + LifetimeFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'varying,
{}

// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `'varying` is covariant in `Self<'varying>`.
//
//   The former implies that `T::covariant_assertions()` does not panic,
//   in which case `'varying` is covariant in `T<'varying>`,
//   implying that `'varying` is covariant in `&'varying T<'varying>`.
//
// - No assertions are included other than those in `Self::covariant_assertions()`.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'lower, 'upper, T> CovariantFamily<'lower, 'upper> for VaryingRef<T>
where
    T: ?Sized + CovariantFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'varying,
{
    #[inline]
    fn covariant_assertions() {
        T::covariant_assertions();
    }

    #[inline]
    fn shorten<'l, 's>(
        long: Varying<'l, 'lower, 'upper, Self>,
    ) -> Varying<'s, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, 'upper, Self>: Sized,
    {
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is just a covariant cast with, possibly, some assertions.
        // Any possible sources of panics in `T::shorten_ref` must be included in
        // `T::covariant_assertions`, which are included in `Self::covariant_assertions`.

        T::shorten_ref(long)
    }

    #[inline]
    fn shorten_ref<'l, 's, 'r>(
        long: &'r Varying<'l, 'lower, 'upper, Self>,
    ) -> &'r Varying<'s, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'l, 'lower, 'upper, Self>: 'r,
        Varying<'s, 'lower, 'upper, Self>: 'r,
    {
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a covariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.

        Self::covariant_assertions();

        let src: &'r &'l Varying<'l, 'lower, 'upper, T> = long;
        let src: &'r &'s Varying<'l, 'lower, 'upper, T> = src;
        // SAFETY: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which is
        // at least as long as `'lower`. We called `T::covariant_assertions()` (within
        // `Self::covariant_assertions()`), so covariantly casting `T<'varying>` is sound.
        let dst: &'r &'s Varying<'s, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
    }
}

// `'varying` is never contravariant in `&'varying T<'varying>`. It's always at best covariant.


// ================================================================
//  *const T
// ================================================================

// Safety summary:
// - `'varying` is covariant in `*const T<'varying>` if it's covariant in `T<'varying>`
// - it's contravariant in `*const T<'varying>` if it's contravariant in `T<'varying>`.

impl<'varying, 'lower, 'upper, T> WithLifetime<'varying, 'lower, 'upper> for *const T
where
    T: ?Sized + WithLifetime<'varying, 'lower, 'upper>,
{
    type Is = *const T::Is;
}

impl<'lower, 'upper, T> LifetimeFamily<'lower, 'upper> for *const T
where
    T: ?Sized + LifetimeFamily<'lower, 'upper>,
{}

// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `'varying` is covariant in `Self<'varying>`.
//
//   The former implies that `T::covariant_assertions()` does not panic,
//   in which case `'varying` is covariant in `T<'varying>`,
//   implying that `'varying` is covariant in `*const T<'varying>`.
//
// - No assertions are included other than those in `Self::covariant_assertions()`.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'lower, 'upper, T> CovariantFamily<'lower, 'upper> for *const T
where
    T: ?Sized + CovariantFamily<'lower, 'upper>,
{
    #[inline]
    fn covariant_assertions() {
        T::covariant_assertions();
    }

    /// Shorten the `'varying` lifetime of `*const T<'varying>`.
    ///
    /// If the source pointer points to a valid value of type `Varying<'l, 'lower, 'upper, T>`
    /// (also referred to as `T<'l>`), that pointer (which is returned with a casted type)
    /// also points to a valid value of type  `Varying<'s, 'lower, 'upper, T>`
    /// (also referred to as `T<'s>`).
    ///
    /// `unsafe` code can rely on that guarantee.
    fn shorten<'l, 's>(
        long: Varying<'l, 'lower, 'upper, Self>,
    ) -> Varying<'s, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, 'upper, Self>: Sized,
    {
        #![expect(clippy::as_conversions, reason = "`.cast()` requires a `Sized` bound")]

        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a covariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.
        Self::covariant_assertions();

        let src: *const Varying<'l, 'lower, 'upper, T> = long;
        // Correctness of guarantee: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which
        // is at least as long as `'lower`. We called `T::covariant_assertions()` (within
        // `Self::covariant_assertions()`), so covariantly casting `T<'varying>` is sound.
        src as *const Varying<'s, 'lower, 'upper, T>
    }

    /// Shorten the `'varying` lifetime of `&(*const T<'varying>)`.
    ///
    /// If the referenced pointer points to a valid value of type `Varying<'l, 'lower, 'upper, T>`
    /// (also referred to as `T<'l>`), that pointer (whose reference is returned with a casted
    /// type) also points to a valid value of type `Varying<'s, 'lower, 'upper, T>` (also referred
    /// to as `T<'s>`).
    ///
    /// `unsafe` code can rely on that guarantee.
    fn shorten_ref<'l, 's, 'r>(
        long: &'r Varying<'l, 'lower, 'upper, Self>,
    ) -> &'r Varying<'s, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'l, 'lower, 'upper, Self>: 'r,
        Varying<'s, 'lower, 'upper, Self>: 'r,
    {
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a covariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.

        Self::covariant_assertions();

        let src: &'r *const Varying<'l, 'lower, 'upper, T> = long;
        // SAFETY: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which is
        // at least as long as `'lower`. We called `T::covariant_assertions()` (within
        // `Self::covariant_assertions()`), so covariantly casting `T<'varying>` is sound.
        let dst: &'r *const Varying<'s, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
    }
}

// SAFETY:
// - If `Self::contravariant_assertions()` does not panic,
//   then `'varying` is contravariant in `Self<'varying>`.
//
//   The former implies that `T::contravariant_assertions()` does not panic,
//   in which case `'varying` is contravariant in `T<'varying>`,
//   implying that `'varying` is contravariant in `*const T<'varying>`.
//
// - No assertions are included other than those in `Self::contravariant_assertions()`.
// - The implementation safety requirements of `lengthen` and `lengthen_ref` are met.
unsafe impl<'lower, 'upper, T> ContravariantFamily<'lower, 'upper> for *const T
where
    T: ?Sized + ContravariantFamily<'lower, 'upper>,
{
    #[inline]
    fn contravariant_assertions() {
        T::contravariant_assertions();
    }

    #[inline]
    fn lengthen<'s, 'l>(
        short: Varying<'s, 'lower, 'upper, Self>,
    ) -> Varying<'l, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, 'upper, Self>: Sized,
    {
        #![expect(clippy::as_conversions, reason = "`.cast()` requires a `Sized` bound")]

        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a contravariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::contravariant_assertions()`
        // call.
        Self::contravariant_assertions();

        let src: *const Varying<'s, 'lower, 'upper, T> = short;
        // Correctness of guarantee: we are lengthening the `'s` lifetime of `T<'s>` to `'l`, which
        // is at most as long as `'upper`. We called `T::contravariant_assertions()` (within
        // `Self::contravariant_assertions()`), so contravariantly casting `T<'varying>` is sound.
        src as *const Varying<'l, 'lower, 'upper, T>
    }

    #[inline]
    fn lengthen_ref<'s, 'l, 'r>(
        short: &'r Varying<'s, 'lower, 'upper, Self>,
    ) -> &'r Varying<'l, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'l, 'lower, 'upper, Self>: 'r,
        Varying<'s, 'lower, 'upper, Self>: 'r,
    {
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a contravariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::contravariant_assertions()`
        // call.

        Self::contravariant_assertions();

        let src: &'r *const Varying<'s, 'lower, 'upper, T> = short;
        // SAFETY: we are lengthening the `'s` lifetime of `T<'s>` to `'l`, which is
        // at most as long as `'upper`. We called `T::contravariant_assertions()` (within
        // `Self::contravariant_assertions()`), so contravariantly casting `T<'varying>` is sound.
        let dst: &'r *const Varying<'l, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
    }
}
