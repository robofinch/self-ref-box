use crate::invariant_zst;
use crate::traits::{
    ContravariantFamily, CovariantFamily, LifetimeFamily, UnvaryingFamily, Varying, WithLifetime,
};


// We *could* use the public macros to implement everything besides the function pointer cases,
// but I think it's valuable to show the full `unsafe` code for at least the most crucial cases.

// Note: in below safety comments, "is covariant in" or "is contravariant in" means, more
// precisely, "can be covariantly casted in" or "can be contravariantly casted in".

// ================================================================
//  &'a mut T
// ================================================================

// Safety summary:
// - `'varying` is unused (and thus capable of being covariantly or contravariantly casted) in
//   `&'a T<'varying>` if it's unused in `T<'varying>`.
//   I'm tempted to extend this condition to some sort of "used but irrelevant" lifetime family
//   trait, but that has to be an extremely niche situation. Moreover, covariance and contravariance
//   do not together imply that `'varying` is unused, and since `U <: V` and `V <: U` do not
//   together imply `U == V`
//   (see: higher-ranked function pointers without canonical normalizations),
//   I'm extremely wary of casting between `&'a mut T<'v1>` and `&'a mut T<'v2>`.

impl<'a, 'varying, 'lower, 'upper, T> WithLifetime<'varying, 'lower, 'upper> for &'a mut T
where
    T: ?Sized + WithLifetime<'varying, 'lower, 'upper>,
    T::Is: 'a,
{
    type Is = &'a mut T::Is;
}

impl<'a, 'lower, 'upper, T> LifetimeFamily<'lower, 'upper> for &'a mut T
where
    T: ?Sized + LifetimeFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'a,
{}

// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `'varying` is covariant in `Self<'varying>`.
//
//   `Self::covariant_assertions()` is trivial and never panics. The bound we use implies
//   that `'varying` is entirely unused in `T<'varying>` and thus also in `&'a mut T<'varying>`,
//   so none of the casts actually do anything. Covariant casts are trivially sound.
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
        // Using implicit covariant coercion to implement this function always fulfills the
        // safety requirement.
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
        // Using implicit covariant coercion to implement this function always fulfills the
        // safety requirement.
        long
    }
}

// SAFETY:
// - If `Self::contravariant_assertions()` does not panic,
//   then `'varying` is contravariant in `Self<'varying>`.
//
//   `Self::contravariant_assertions()` is trivial and never panics. The bound we use implies
//   that `'varying` is entirely unused in `T<'varying>` and thus also in `&'a mut T<'varying>`,
//   so none of the casts actually do anything. Contravariant casts are trivially sound.
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
        // Using implicit contravariant coercion to implement this function always fulfills the
        // safety requirement.
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
        // Using implicit contravariant coercion to implement this function always fulfills the
        // safety requirement.
        short
    }
}


// ================================================================
//  &'varying mut T    (VaryingRefMut<T>)
// ================================================================

// Safety summary:
// - `'varying` is covariant in `&'varying T<'varying>` if it's entirely unused in `T<'varying>`.
//   See `&'a mut T` for more on that.
// - `'varying` is never contravariant in `&'varying T<'varying>`.

invariant_zst!(
    /// The `&'varying mut T<'varying>` lifetime family.
    ///
    /// If `'varying` is entirely unused in `T<'varying>`, then `'varying` is covariant
    /// in `&'varying mut T<'varying>` (which is really just `&'varying mut U` for some `U`).
    ///
    /// `'varying` is never contravariant in this lifetime family.
    ///
    /// Note that this type itself is just a marker ZST for the family.
    pub struct VaryingRefMut<T: ?Sized>;
);

impl<'varying, 'lower, 'upper, T> WithLifetime<'varying, 'lower, 'upper> for VaryingRefMut<T>
where
    T: ?Sized + WithLifetime<'varying, 'lower, 'upper>,
    T::Is: 'varying,
{
    type Is = &'varying T::Is;
}

impl<'lower, 'upper, T> LifetimeFamily<'lower, 'upper> for VaryingRefMut<T>
where
    T: ?Sized + LifetimeFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'varying,
{}

// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `'varying` is covariant in `Self<'varying>`.
//
//   `Self::covariant_assertions()` is trivial and never panics. The bound we use implies
//   that `'varying` is entirely unused in `T<'varying>` and thus also in
//   `&'varying mut T<'varying>`, so none of the casts actually do anything.
//   Covariant casts are trivially sound.
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
        // Using implicit covariant coercion to implement this function always fulfills the
        // safety requirement.
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
        // Using implicit covariant coercion to implement this function always fulfills the
        // safety requirement.
        long
    }
}

// `'varying` is never contravariant in `&'varying T<'varying>`. It's always at best covariant.


// ================================================================
//  *mut T
// ================================================================

// Safety summary:
// - `'varying` is unused (and thus capable of being covariantly or contravariantly casted) in
//   `*const T<'varying>` if it's unused in `T<'varying>`. See `&'a mut T` for more on that.

impl<'varying, 'lower, 'upper, T> WithLifetime<'varying, 'lower, 'upper> for *mut T
where
    T: ?Sized + WithLifetime<'varying, 'lower, 'upper>,
{
    type Is = *mut T::Is;
}

impl<'lower, 'upper, T> LifetimeFamily<'lower, 'upper> for *mut T
where
    T: ?Sized + LifetimeFamily<'lower, 'upper>,
{}
// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `'varying` is covariant in `Self<'varying>`.
//
//   `Self::covariant_assertions()` is trivial and never panics. The bound we use implies
//   that `'varying` is entirely unused in `T<'varying>` and thus also in `*mut T<'varying>`,
//   so none of the casts actually do anything. Covariant casts are trivially sound.
// - No assertions are included.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'lower, 'upper, T> CovariantFamily<'lower, 'upper> for *mut T
where
    T: ?Sized + UnvaryingFamily<'lower, 'upper>,
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
        // Using implicit covariant coercion to implement this function always fulfills the
        // safety requirement.
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
        // Using implicit covariant coercion to implement this function always fulfills the
        // safety requirement.
        long
    }
}

// SAFETY:
// - If `Self::contravariant_assertions()` does not panic,
//   then `'varying` is contravariant in `Self<'varying>`.
//
//   `Self::contravariant_assertions()` is trivial and never panics. The bound we use implies
//   that `'varying` is entirely unused in `T<'varying>` and thus also in `*mut T<'varying>`,
//   so none of the casts actually do anything. Contravariant casts are trivially sound.
// - No assertions are included.
// - The implementation safety requirements of `lengthen` and `lengthen_ref` are met.
unsafe impl<'lower, 'upper, T> ContravariantFamily<'lower, 'upper> for *mut T
where
    T: ?Sized + UnvaryingFamily<'lower, 'upper>,
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
        // Using implicit contravariant coercion to implement this function always fulfills the
        // safety requirement.
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
        // Using implicit contravariant coercion to implement this function always fulfills the
        // safety requirement.
        short
    }
}
