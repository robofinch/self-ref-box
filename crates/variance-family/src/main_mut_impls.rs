use crate::invariant_zst;
use crate::traits::{ContravariantFamily, CovariantFamily, UnvaryingFamily, Varying, WithLifetime};


// We *could* use the public macros to implement everything besides the function pointer cases,
// but I think it's valuable to show the full `unsafe` code for at least the most crucial cases.

// Note: in below safety comments, "is covariant over" or "is contravariant over" means, more
// precisely, "is sound to covariantly (or contravariantly) cast with respect to". That is,
// manually-proven variance (and manually-proven soundness of casts) is the relevant concern,
// not compiler-assigned variance (and compiler-proven soundness of casts).

// ================================================================
//  &'a mut T
// ================================================================

// Safety summary:
// - `&'a mut U` is bivariant over `'varying` (as it's entirely unused). Below, `T<'varying>`
//   families are used which implement `UnvaryingFamily`, making them equivalent to `&'a mut U`
//   for some type `U`. Unsafe transmutes aren't even needed.

impl<'a, 'varying, 'lower, 'upper, T> WithLifetime<'varying, 'lower, 'upper> for &'a mut T
where
    T: ?Sized + WithLifetime<'varying, 'lower, 'upper>,
    T::Is: 'a,
{
    type Is = &'a mut T::Is;
}

// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `Self<'varying>` is covariant over `'varying`.
//
//   `Self::covariant_assertions()` is trivial and never panics, and `Self<'varying>` does not
//   actually use `'varying` at all, making it covariant over `'varying`.
//
// - No assertions are included.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'a, 'lower, 'upper, T> CovariantFamily<'lower, 'upper> for &'a mut T
where
    T: ?Sized + UnvaryingFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'a,
{
    #[inline]
    fn covariant_assertions() {}

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
        // Implementation safety: implementing this with `{ long }` is always safe.

        long
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
        // Implementation safety: implementing this with `{ long }` is always safe.

        long
    }
}

// SAFETY:
// - If `Self::contravariant_assertions()` does not panic,
//   then `Self<'varying>` is covariant over `'varying`.
//
//   `Self::contravariant_assertions()` is trivial and never panics, and `Self<'varying>` does not
//   actually use `'varying` at all, making it contravariant over `'varying`.
//
// - No assertions are included.
// - The implementation safety requirements of `lengthen` and `lengthen_ref` are met.
unsafe impl<'a, 'lower, 'upper, T> ContravariantFamily<'lower, 'upper> for &'a mut T
where
    T: ?Sized + UnvaryingFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'a,
{
    #[inline]
    fn contravariant_assertions() {}

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
        // Implementation safety: implementing this with `{ short }` is always safe.

        short
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
        // Implementation safety: implementing this with `{ short }` is always safe.

        short
    }
}


// ================================================================
//  &'varying mut T    (VaryingRefMut<T>)
// ================================================================

// Safety summary:
// - `&'varying mut U` is covariant over `'varying`. Below, `T<'varying>` families are used which
//   implement `UnvaryingFamily`, making them equivalent to `&'a mut U` for some type `U`.
//   Unsafe transmutes aren't even needed.
// - `&'varying mut T<'varying>` is never contravariant over `'varying`.

invariant_zst!(
    /// The `&'varying mut T<'varying>` lifetime family.
    ///
    /// If `T<'varying>` does not actually use `'varying` at all (making it some fixed type `U`
    /// regardless of `'varying`), then `&'varying mut T<'varying>` is covariant over `'varying`.
    ///
    /// This lifetime family is never contravariant over `'varying`.
    ///
    /// Note that this type itself is just a marker ZST for the family.
    pub struct VaryingRefMut<T: ?Sized>;
);

impl<'varying, 'lower, 'upper, T> WithLifetime<'varying, 'lower, 'upper> for VaryingRefMut<T>
where
    T: ?Sized + WithLifetime<'varying, 'lower, 'upper>,
    T::Is: 'varying,
{
    type Is = &'varying mut T::Is;
}

// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `Self<'varying>` is covariant over `'varying`.
//
//   `Self::covariant_assertions()` is trivial and never panics, and `Self<'varying>` does not
//   actually use `'varying` at all, making it covariant over `'varying`.
//
// - No assertions are included.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'lower, 'upper, T> CovariantFamily<'lower, 'upper> for VaryingRefMut<T>
where
    T: ?Sized + UnvaryingFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'varying,
{
    #[inline]
    fn covariant_assertions() {}

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
        // Implementation safety: implementing this with `{ long }` is always safe.

        long
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
        // Implementation safety: implementing this with `{ long }` is always safe.

        long
    }
}

// `&'varying mut T<'varying>` is never contravariant over `'varying`. It's always at best
// covariant, never bivariant.


// ================================================================
//  *mut T
// ================================================================

// Safety summary:
// - `*mut U` is bivariant over `'varying` (as it's entirely unused). Below, `T<'varying>`
//   families are used which implement `UnvaryingFamily`, making them equivalent to `*mut U`
//   for some type `U`. Unsafe transmutes aren't even needed.

impl<'varying, 'lower, 'upper, T> WithLifetime<'varying, 'lower, 'upper> for *mut T
where
    T: ?Sized + WithLifetime<'varying, 'lower, 'upper>,
{
    type Is = *mut T::Is;
}

// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `Self<'varying>` is covariant over `'varying`.
//
//   `Self::covariant_assertions()` is trivial and never panics, and `Self<'varying>` does not
//   actually use `'varying` at all, making it covariant over `'varying`.
//
// - No assertions are included.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'lower, 'upper, T> CovariantFamily<'lower, 'upper> for *mut T
where
    T: ?Sized + UnvaryingFamily<'lower, 'upper>,
{
    #[inline]
    fn covariant_assertions() {}

    /// Shorten the `'varying` lifetime of `*mut T<'varying>`.
    ///
    /// If the given pointer points to a valid value of type `Varying<'l, 'lower, 'upper, T>`
    /// (also referred to as `T<'l>`), the returned pointer (which is the given pointer with a
    /// casted type) points to a valid value of type  `Varying<'s, 'lower, 'upper, T>`
    /// (also referred to as `T<'s>`).
    ///
    /// As the returned pointer is not modified (other than to change its type), any other
    /// qualities relevant for reads or writes through the pointer (such as alignment or provenance)
    /// are unchanged.
    ///
    /// `unsafe` code can rely on this guarantee.
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
        // Implementation safety: implementing this with `{ long }` is always safe.

        long
    }

    /// Shorten the `'varying` lifetime of `&(*mut T<'varying>)`.
    ///
    /// If the referenced pointer points to a valid value of type `Varying<'l, 'lower, 'upper, T>`
    /// (also referred to as `T<'l>`), that pointer (whose reference is returned with a casted
    /// type) also points to a valid value of type `Varying<'s, 'lower, 'upper, T>` (also referred
    /// to as `T<'s>`).
    ///
    /// As the returned reference to the pointer is not modified (other than to change the
    /// pointer's type), any other qualities relevant for reads or writes through the pointer
    /// (such as alignment or provenance) are unchanged.
    ///
    /// `unsafe` code can rely on this guarantee.
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
        // Implementation safety: implementing this with `{ long }` is always safe.

        long
    }
}

// SAFETY:
// - If `Self::contravariant_assertions()` does not panic,
//   then `Self<'varying>` is covariant over `'varying`.
//
//   `Self::contravariant_assertions()` is trivial and never panics, and `Self<'varying>` does not
//   actually use `'varying` at all, making it contravariant over `'varying`.
//
// - No assertions are included.
// - The implementation safety requirements of `lengthen` and `lengthen_ref` are met.
unsafe impl<'lower, 'upper, T> ContravariantFamily<'lower, 'upper> for *mut T
where
    T: ?Sized + UnvaryingFamily<'lower, 'upper>,
{
    #[inline]
    fn contravariant_assertions() {}

    /// Lengthen the `'varying` lifetime of `*const T<'varying>`.
    ///
    /// If the given pointer points to a valid value of type `Varying<'s, 'lower, 'upper, T>`
    /// (also referred to as `T<'s>`), the returned pointer (which is the given pointer with a
    /// casted type) points to a valid value of type `Varying<'l, 'lower, 'upper, T>`
    /// (also referred to as `T<'l>`).
    ///
    /// As the returned pointer is not modified (other than to change its type), any other
    /// qualities relevant for reads or writes through the pointer (such as alignment or provenance)
    /// are unchanged.
    ///
    /// `unsafe` code can rely on this guarantee.
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
        // Implementation safety: implementing this with `{ short }` is always safe.

        short
    }

    /// Lenghten the `'varying` lifetime of `&(*const T<'varying>)`.
    ///
    /// If the referenced pointer points to a valid value of type `Varying<'s, 'lower, 'upper, T>`
    /// (also referred to as `T<'s>`), that pointer (whose reference is returned with a casted
    /// type) also points to a valid value of type `Varying<'l, 'lower, 'upper, T>` (also referred
    /// to as `T<'l>`).
    ///
    /// As the returned reference to the pointer is not modified (other than to change the
    /// pointer's type), any other qualities relevant for reads or writes through the pointer
    /// (such as alignment or provenance) are unchanged.
    ///
    /// `unsafe` code can rely on this guarantee.
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
        // Implementation safety: implementing this with `{ short }` is always safe.

        short
    }
}
