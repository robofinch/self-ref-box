use core::mem::transmute;

use crate::invariant_zst;
use crate::traits::{BivariantFamily, ContravariantFamily, CovariantFamily, Varying, WithLifetime};


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
// - `&'a mut T<'varying>` is bivariant over `'varying` if `T<'varying>` is bivariant over
//   `'varying`.
//   If the latter holds, then both covariant and contravariant casts of `T<'varying>`
//   are permissible, and since `&'a mut P` does not place any typestate-ish semantics
//   on `P` (which would have significance for data beyond `P` itself), nothing in
//   `&'a mut T<'varying>` actually cares about the `'varying` lifetime (so long as it's within
//   `'lower` and `'upper`), so the lifetime can be changed even in an invariant position.
//   See the documentation of `BivariantFamily` (especially the Further Details on Safety section)
//   for extensive discussion.

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
//   The former implies that neither `T::covariant_assertions()` nor
//   `T::contravariant_assertions()` panic,
//   in which case `T<'varying>` is bivariant over `'varying`,
//   implying that `&'a mut T<'varying>` is bivariant (and thus covariant) over `'varying`.
//   See the above safety summary and the documentation of `BivariantFamily` for more.
//
// - No assertions are included other than those in `Self::covariant_assertions()`.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'a, 'lower, 'upper, T> CovariantFamily<'lower, 'upper> for &'a mut T
where
    T: ?Sized + BivariantFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'a,
{
    #[inline]
    fn covariant_assertions() {
        T::covariant_assertions();
        T::contravariant_assertions();
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
        // Implementation safety: this is a covariant (well, bivariant) cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.

        Self::covariant_assertions();

        let src: &'a mut Varying<'l, 'lower, 'upper, T> = long;
        // SAFETY: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which is
        // at least as long as `'lower`. We called `T::covariant_assertions()` and
        // `T::contravariant_assertions()`, so, as documented in `BivariantFamily`,
        // `CovariantFamily`, and `ContravariantFamily`, it is sound to change the `'varying`
        // lifetime (between `'lower` and `'upper`) even when `T<'varying>` is in an invariant
        // position like this.
        let dst: &'a mut Varying<'s, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
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
        // Implementation safety: this is a covariant (well, bivariant) cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.

        Self::covariant_assertions();

        let src: &'r &'a mut Varying<'l, 'lower, 'upper, T> = long;
        // SAFETY: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which is
        // at least as long as `'lower`. We called `T::covariant_assertions()` and
        // `T::contravariant_assertions()`, so, as documented in `BivariantFamily`,
        // `CovariantFamily`, and `ContravariantFamily`, it is sound to change the `'varying`
        // lifetime (between `'lower` and `'upper`) even when `T<'varying>` is in an invariant
        // position like this.
        let dst: &'r &'a mut Varying<'s, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
    }
}

// SAFETY:
// - If `Self::contravariant_assertions()` does not panic,
//   then `Self<'varying>` is covariant over `'varying`.
//
//   The former implies that neither `T::covariant_assertions()` nor
//   `T::contravariant_assertions()` panic,
//   in which case `T<'varying>` is bivariant over `'varying`,
//   implying that `&'a mut T<'varying>` is bivariant (and thus contravariant) over `'varying`.
//   See the above safety summary and the documentation of `BivariantFamily` for more.
//
// - No assertions are included other than those in `Self::contravariant_assertions()`.
// - The implementation safety requirements of `lengthen` and `lengthen_ref` are met.
unsafe impl<'a, 'lower, 'upper, T> ContravariantFamily<'lower, 'upper> for &'a mut T
where
    T: ?Sized + BivariantFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'a,
{
    #[inline]
    fn contravariant_assertions() {
        T::covariant_assertions();
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
        // Implementation safety: this is a contravariant (well, bivariant) cast with some
        // assertions. There are no possible sources of panics other than the
        // `Self::contravariant_assertions()` call.

        Self::contravariant_assertions();

        let src: &'a mut Varying<'s, 'lower, 'upper, T> = short;
        // SAFETY: we are lengthening the `'s` lifetime of `T<'s>` to `'l`, which is
        // at most as long as `'upper`. We called `T::covariant_assertions()` and
        // `T::contravariant_assertions()`, so, as documented in `BivariantFamily`,
        // `CovariantFamily`, and `ContravariantFamily`, it is sound to change the `'varying`
        // lifetime (between `'lower` and `'upper`) even when `T<'varying>` is in an invariant
        // position like this.
        let dst: &'a mut Varying<'l, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
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
        // Implementation safety: this is a contravariant (well, bivariant) cast with some
        // assertions. There are no possible sources of panics other than the
        // `Self::contravariant_assertions()` call.

        Self::contravariant_assertions();

        let src: &'r &'a mut Varying<'s, 'lower, 'upper, T> = short;
        // SAFETY: we are lengthening the `'s` lifetime of `T<'s>` to `'l`, which is
        // at most as long as `'upper`. We called `T::covariant_assertions()` and
        // `T::contravariant_assertions()`, so, as documented in `BivariantFamily`,
        // `CovariantFamily`, and `ContravariantFamily`, it is sound to change the `'varying`
        // lifetime (between `'lower` and `'upper`) even when `T<'varying>` is in an invariant
        // position like this.
        let dst: &'r &'a mut Varying<'l, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
    }
}


// ================================================================
//  &'varying mut T    (VaryingRefMut<T>)
// ================================================================

// Safety summary:
// - `&'varying mut T<'varying>` is covariant over `'varying` if `T<'varying>` is bivariant over
//   `'varying`.
//   If the latter holds, then both covariant and contravariant casts of `T<'varying>`
//   are permissible, and since `&'varying mut P` does not place any typestate-ish semantics
//   on `P` (which would have significance for data beyond `P` itself), nothing in
//   `&'varying mut T<'varying>` actually cares about the `'varying` lifetime (so long as it's
//   within `'lower` and `'upper`), so the lifetime of `T<'varying>` can be changed even in an
//   invariant position. The outer `&'varying mut _` does not allow for contravariant casts,
//   reducing the bivariance of `&'a mut T<'varying>`
//   to covariance in the case of `&'varying mut T<'varying>`.
//   See the documentation of `BivariantFamily` (especially the Further Details on Safety section)
//   for extensive discussion.
// - `&'varying mut T<'varying>` is never contravariant over `'varying`.

invariant_zst!(
    /// The `&'varying mut T<'varying>` lifetime family.
    ///
    /// If `T<'varying>` is bivariant over `'varying` (that is, allows both covariant casts and
    /// contravariant casts of the `'varying` lifetime), then `&'varying mut T<'varying>` is
    /// covariant over `'varying`.
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
//   The former implies that neither `T::covariant_assertions()` nor
//   `T::contravariant_assertions()` panic,
//   in which case `T<'varying>` is bivariant over `'varying`,
//   implying that `&'varying mut T<'varying>` is covariant over `'varying`.
//   See the above safety summary and the documentation of `BivariantFamily` for more.
//
// - No assertions are included other than those in `Self::covariant_assertions()`.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'lower, 'upper, T> CovariantFamily<'lower, 'upper> for VaryingRefMut<T>
where
    T: ?Sized + BivariantFamily<'lower, 'upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: 'varying,
{
    #[inline]
    fn covariant_assertions() {
        T::covariant_assertions();
        T::contravariant_assertions();
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
        // Implementation safety: this is a covariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.

        Self::covariant_assertions();

        let src: &'l mut Varying<'l, 'lower, 'upper, T> = long;
        let src: &'s mut Varying<'l, 'lower, 'upper, T> = src;
        // SAFETY: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which is
        // at least as long as `'lower`. We called `T::covariant_assertions()` and
        // `T::contravariant_assertions()`, so, as documented in `BivariantFamily`,
        // `CovariantFamily`, and `ContravariantFamily`, it is sound to change the `'varying`
        // lifetime (between `'lower` and `'upper`) even when `T<'varying>` is in an invariant
        // position like this.
        let dst: &'s mut Varying<'s, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
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

        let src: &'r &'l mut Varying<'l, 'lower, 'upper, T> = long;
        let src: &'r &'s mut Varying<'l, 'lower, 'upper, T> = src;
        // SAFETY: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which is
        // at least as long as `'lower`. We called `T::covariant_assertions()` and
        // `T::contravariant_assertions()`, so, as documented in `BivariantFamily`,
        // `CovariantFamily`, and `ContravariantFamily`, it is sound to change the `'varying`
        // lifetime (between `'lower` and `'upper`) even when `T<'varying>` is in an invariant
        // position like this.
        let dst: &'r &'s mut Varying<'s, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
    }
}

// `&'varying mut T<'varying>` is never contravariant over `'varying`. It's always at best
// covariant, never bivariant.


// ================================================================
//  *mut T
// ================================================================

// Safety summary:
// - `*mut T<'varying>` is bivariant over `'varying` if `T<'varying>` is bivariant over
//   `'varying`.
//   If the latter holds, then both covariant and contravariant casts of `T<'varying>`
//   are permissible, and since `*mut P` does not place any typestate-ish semantics
//   on `P` (which would have significance for data beyond `P` itself), nothing in
//   `*mut T<'varying>` actually cares about the `'varying` lifetime (so long as it's within
//   `'lower` and `'upper`), so the lifetime can be changed even in an invariant position.
//   See the documentation of `BivariantFamily` (especially the Further Details on Safety section)
//   for extensive discussion.

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
//   The former implies that neither `T::covariant_assertions()` nor
//   `T::contravariant_assertions()` panic,
//   in which case `T<'varying>` is bivariant over `'varying`,
//   implying that `*mut T<'varying>` is bivariant (and thus covariant) over `'varying`.
//   See the above safety summary and the documentation of `BivariantFamily` for more.
//
// - No assertions are included other than those in `Self::covariant_assertions()`.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'lower, 'upper, T> CovariantFamily<'lower, 'upper> for *mut T
where
    T: ?Sized + BivariantFamily<'lower, 'upper>,
{
    #[inline]
    fn covariant_assertions() {
        T::covariant_assertions();
        T::contravariant_assertions();
    }

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
        #![expect(clippy::as_conversions, reason = "`.cast()` requires a `Sized` bound")]

        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a covariant (well, bivariant) cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.
        Self::covariant_assertions();

        let src: *mut Varying<'l, 'lower, 'upper, T> = long;
        // Correctness of guarantee: we are shortening the `'l` lifetime of `T<'l>` to `'s`,
        // which is at least as long as `'lower`. We called `T::covariant_assertions()` and
        // `T::contravariant_assertions()`, so, as documented in `BivariantFamily`,
        // `CovariantFamily`, and `ContravariantFamily`, it is sound to change the `'varying`
        // lifetime (between `'lower` and `'upper`) even when `T<'varying>` is in an invariant
        // position like this.
        src as *mut Varying<'s, 'lower, 'upper, T>
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
        // Implementation safety: this is a covariant (well, bivariant) cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.

        Self::covariant_assertions();

        let src: &'r *mut Varying<'l, 'lower, 'upper, T> = long;
        // SAFETY: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which is
        // at least as long as `'lower`. We called `T::covariant_assertions()` and
        // `T::contravariant_assertions()`, so, as documented in `BivariantFamily`,
        // `CovariantFamily`, and `ContravariantFamily`, it is sound to change the `'varying`
        // lifetime (between `'lower` and `'upper`) even when `T<'varying>` is in an invariant
        // position like this.
        let dst: &'r *mut Varying<'s, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
    }
}

// SAFETY:
// - If `Self::contravariant_assertions()` does not panic,
//   then `Self<'varying>` is covariant over `'varying`.
//
//   The former implies that neither `T::covariant_assertions()` nor
//   `T::contravariant_assertions()` panic,
//   in which case `T<'varying>` is bivariant over `'varying`,
//   implying that `*mut T<'varying>` is bivariant (and thus contravariant) over `'varying`.
//   See the above safety summary and the documentation of `BivariantFamily` for more.
//
// - No assertions are included other than those in `Self::contravariant_assertions()`.
// - The implementation safety requirements of `lengthen` and `lengthen_ref` are met.
unsafe impl<'lower, 'upper, T> ContravariantFamily<'lower, 'upper> for *mut T
where
    T: ?Sized + BivariantFamily<'lower, 'upper>,
{
    #[inline]
    fn contravariant_assertions() {
        T::covariant_assertions();
        T::contravariant_assertions();
    }

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
        #![expect(clippy::as_conversions, reason = "`.cast()` requires a `Sized` bound")]

        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a contravariant (well, bivariant) cast with some
        // assertions. There are no possible sources of panics other than the
        // `Self::contravariant_assertions()` call.
        Self::contravariant_assertions();

        let src: *mut Varying<'s, 'lower, 'upper, T> = short;
        // Correctness of guarantee: we are lengthening the `'s` lifetime of `T<'s>` to `'l`,
        // which is at most as long as `'upper`. We called `T::covariant_assertions()` and
        // `T::contravariant_assertions()`, so, as documented in `BivariantFamily`,
        // `CovariantFamily`, and `ContravariantFamily`, it is sound to change the `'varying`
        // lifetime (between `'lower` and `'upper`) even when `T<'varying>` is in an invariant
        // position like this.
        src as *mut Varying<'l, 'lower, 'upper, T>
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
        // Implementation safety: this is a contravariant (well, bivariant) cast with some
        // assertions. There are no possible sources of panics other than the
        // `Self::contravariant_assertions()` call.

        Self::contravariant_assertions();

        let src: &'r *mut Varying<'s, 'lower, 'upper, T> = short;
        // SAFETY: we are lengthening the `'s` lifetime of `T<'s>` to `'l`, which is
        // at most as long as `'upper`. We called `T::covariant_assertions()` and
        // `T::contravariant_assertions()`, so, as documented in `BivariantFamily`,
        // `CovariantFamily`, and `ContravariantFamily`, it is sound to change the `'varying`
        // lifetime (between `'lower` and `'upper`) even when `T<'varying>` is in an invariant
        // position like this.
        let dst: &'r *mut Varying<'l, 'lower, 'upper, T> = unsafe { transmute(src) };
        dst
    }
}
