#![expect(unsafe_code, reason = "trivially sound implementations of variance family traits")]

use core::convert::Infallible;

use variance_family::{ContravariantFamily, CovariantFamily, Varying, WithLifetime};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NeverNoRef {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NeverSharedRef {}

impl<Upper: ?Sized> WithLifetime<'_, '_, Upper> for NeverSharedRef {
    type Is = Infallible;
}

// SAFETY: `Varying<'varying, 'lower, Upper, Self>` doesn't use 'varying` whatsoever
// (making covariant casts sound and even safe), no assertions are used, and trivial method
// bodies known to be sound are used.
// TODO: use a macro instead of a direct unsafe impl.
unsafe impl<'lower, Upper: ?Sized> CovariantFamily<'lower, Upper> for NeverSharedRef {
    #[inline]
    fn covariant_assertions() {}

    fn shorten<'l, 's>(
        long: Varying<'l, 'lower, Upper, Self>,
    ) -> Varying<'s, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, Upper, Self>: Sized,
    {
        long
    }

    #[expect(clippy::uninhabited_references, reason = "yes, this function is unreachable")]
    fn shorten_ref<'l, 's, 'r>(
        long: &'r Varying<'l, 'lower, Upper, Self>,
    ) -> &'r Varying<'s, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'l, 'lower, Upper, Self>: 'r,
        Varying<'s, 'lower, Upper, Self>: 'r,
    {
       long
    }
}

// SAFETY: `Varying<'varying, 'lower, Upper, Self>` doesn't use 'varying` whatsoever
// (making contravariant casts sound and even safe), no assertions are used, and trivial method
// bodies known to be sound are used.
// TODO: use a macro instead of a direct unsafe impl.
unsafe impl<'lower, Upper: ?Sized> ContravariantFamily<'lower, Upper> for NeverSharedRef {
    #[inline]
    fn contravariant_assertions() {}

    fn lengthen<'s, 'l>(
        short: Varying<'s, 'lower, Upper, Self>,
    ) -> Varying<'l, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, Upper, Self>: Sized,
    {
        short
    }

    #[expect(clippy::uninhabited_references, reason = "yes, this function is unreachable")]
    fn lengthen_ref<'s, 'l, 'r>(
        short: &'r Varying<'s, 'lower, Upper, Self>,
    ) -> &'r Varying<'l, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'s, 'lower, Upper, Self>: 'r,
        Varying<'l, 'lower, Upper, Self>: 'r,
    {
        short
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NeverExclusiveRef {}

impl<Upper: ?Sized> WithLifetime<'_, '_, Upper> for NeverExclusiveRef {
    type Is = Infallible;
}

// SAFETY: `Varying<'varying, 'lower, Upper, Self>` doesn't use 'varying` whatsoever
// (making covariant casts sound and even safe), no assertions are used, and trivial method
// bodies known to be sound are used.
// TODO: use a macro instead of a direct unsafe impl.
unsafe impl<'lower, Upper: ?Sized> CovariantFamily<'lower, Upper> for NeverExclusiveRef {
    #[inline]
    fn covariant_assertions() {}

    fn shorten<'l, 's>(
        long: Varying<'l, 'lower, Upper, Self>,
    ) -> Varying<'s, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, Upper, Self>: Sized,
    {
        long
    }

    #[expect(clippy::uninhabited_references, reason = "yes, this function is unreachable")]
    fn shorten_ref<'l, 's, 'r>(
        long: &'r Varying<'l, 'lower, Upper, Self>,
    ) -> &'r Varying<'s, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'l, 'lower, Upper, Self>: 'r,
        Varying<'s, 'lower, Upper, Self>: 'r,
    {
       long
    }
}

// SAFETY: `Varying<'varying, 'lower, Upper, Self>` doesn't use 'varying` whatsoever
// (making contravariant casts sound and even safe), no assertions are used, and trivial method
// bodies known to be sound are used.
// TODO: use a macro instead of a direct unsafe impl.
unsafe impl<'lower, Upper: ?Sized> ContravariantFamily<'lower, Upper> for NeverExclusiveRef {
    #[inline]
    fn contravariant_assertions() {}

    fn lengthen<'s, 'l>(
        short: Varying<'s, 'lower, Upper, Self>,
    ) -> Varying<'l, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, Upper, Self>: Sized,
    {
        short
    }

    #[expect(clippy::uninhabited_references, reason = "yes, this function is unreachable")]
    fn lengthen_ref<'s, 'l, 'r>(
        short: &'r Varying<'s, 'lower, Upper, Self>,
    ) -> &'r Varying<'l, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'s, 'lower, Upper, Self>: 'r,
        Varying<'l, 'lower, Upper, Self>: 'r,
    {
        short
    }
}
