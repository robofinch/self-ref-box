use core::{
    fmt::{Debug, Formatter, Result as FmtResult},
    mem::{ManuallyDrop, MaybeUninit},
};

use variance_family::{CovariantFamily, Varying};

use crate::slot::SelfRefSlot;
use super::EraseSelfRef;


pub struct LifetimeErase<'erased, N, S, E>
where
    S: for<'lower> CovariantFamily<'lower, &'erased ()>,
    Varying<'erased, 'erased, &'erased (), S>: Sized,
    E: for<'lower> CovariantFamily<'lower, &'erased ()>,
    Varying<'erased, 'erased, &'erased (), E>: Sized,
{
    /// # Safety Invariant
    /// For some `'varying` lifetime, the value in `maybe_dangling` must be initialized to a valid
    /// value of type `SelfRefSlot<'varying, N, S, E, Self::Upper>`, except in the destructor;
    /// the `maybe_dangling` field is dropped in the `Drop::drop` implementation of this type,
    /// after which point the field should be considered uninitialized.
    ///
    /// Note that the methods of this type are careful not to expose the dangling `'erased`
    /// lifetime outside `MaybeUninit`.
    ///
    /// `MaybeUninit` is used here solely to remove `dereferenceable` (and possibly `noalias`)
    /// requirements from the `SelfRefSlot` value. This can be relevant in destructors of
    /// self-referential structs, where the incorrect `'erased` lifetime could otherwise cause
    /// unsoundness.
    maybe_dangling: MaybeUninit<SelfRefSlot<'erased, N, S, E, &'erased ()>>,
}

// SAFETY: The implementation has the correct semantics; it's not pathological, the methods
// do what they say.
unsafe impl<'erased, N, S, E> EraseSelfRef<N, S, E> for LifetimeErase<'erased, N, S, E>
where
    S: for<'lower> CovariantFamily<'lower, &'erased ()>,
    for<'lower, 'varying> Varying<'varying, 'lower, &'erased (), S>: Sized,
    E: for<'lower> CovariantFamily<'lower, &'erased ()>,
    for<'lower, 'varying> Varying<'varying, 'lower, &'erased (), E>: Sized,
{
    type Upper = &'erased ();

    unsafe fn erase(slot: SelfRefSlot<'_, N, S, E, Self::Upper>) -> Self {
        // This has an `'erased` lifetime.
        let mut maybe_uninit = MaybeUninit::<SelfRefSlot<'erased, N, S, E, &'erased ()>>::uninit();
        let dst = maybe_uninit.as_mut_ptr().cast::<SelfRefSlot<'_, N, S, E, Self::Upper>>();
        // SAFETY:
        // This effectively performs a lifetime transmute (from the `'varying` source to the
        // `'erased` destination). Since we wrap it in `MaybeUninit` and are careful not to let the
        // dangling `'erased` lifetime cause any problems, no unsoundness results from that
        // transmute. In particular, anything with an `'erased` lifetime is always kept behind
        // `MaybeUninit`.
        //
        // By-the-book requirements:
        // - the return value of `maybe_uninit.as_mut_ptr()` is non-null
        // - `dst` is dereferenceable, since it's valid for writes of type
        //   `SelfRefSlot<'erased, N, S, E, Self::Upper>`.
        // - `dst` is properly aligned for `SelfRefSlot<'erased, N, S, E, Self::Upper>` and thus
        //   also for `SelfRefSlot<'_, N, S, E, Self::Upper>`.
        // Note that the last two points rely on `SelfRefSlot<'any, N, S, E, Self::Upper>`
        // having the same size and alignment regardless of the `'any` lifetime.
        unsafe {
            dst.write(slot);
        };

        Self {
            // SAFETY INVARIANT: `maybe_uninit` has been initialized (ignoring the
            // `'erased` lifetime).
            maybe_dangling: maybe_uninit,
        }
    }

    unsafe fn unerase<'varying: 'varying>(
        slot: Self,
    ) -> SelfRefSlot<'varying, N, S, E, Self::Upper>
    where
        Self::Upper: 'varying,
    {
        let slot = ManuallyDrop::new(slot);
        let maybe_dangling = slot.maybe_dangling.as_ptr();
        let not_dangling = maybe_dangling.cast::<SelfRefSlot<'varying, N, S, E, &'erased ()>>();

        // SAFETY: this is basically `MaybeUninit::assume_init`, except we can't use that exact
        // function, since the relevant `MaybeUninit` has the wrong lifetime *and* is behind a
        // `ManuallyDrop`. This is sound because, as per the safety invariant, `self.maybe_dangling`
        // is initialized (ignoring the `'erased` lifetime), and the caller asserts that using
        // a `'varying` lifetime is sound. Note that anything with an `'erased` lifetime is always
        // kept behind `MaybeUninit`.
        //
        // By-the-book requirements:
        unsafe { not_dangling.read() }
    }

    unsafe fn unerase_ref<'varying: 'varying>(
        slot: &Self,
    ) -> &SelfRefSlot<'varying, N, S, E, Self::Upper>
    where
        Self::Upper: 'varying,
    {
        let maybe_dangling = slot.maybe_dangling.as_ptr();
        let not_dangling = maybe_dangling.cast::<SelfRefSlot<'varying, N, S, E, &'erased ()>>();

        // SAFETY: this is basically `MaybeUninit::assume_init_ref`, except we can't use that exact
        // function, since the relevant `MaybeUninit` has the wrong lifetime *and* is behind a
        // `ManuallyDrop`. This is sound because, as per the safety invariant, `self.maybe_dangling`
        // is initialized (ignoring the `'erased` lifetime), and the caller asserts that using
        // a `'varying` lifetime is sound. Note that anything with an `'erased` lifetime is always
        // kept behind `MaybeUninit`.
        //
        // By-the-book requirements:
        unsafe { &*not_dangling }
    }

    unsafe fn unerase_mut<'varying: 'varying>(
        slot: &mut Self,
    ) -> &mut SelfRefSlot<'varying, N, S, E, Self::Upper>
    where
        Self::Upper: 'varying,
    {
        let maybe_dangling = slot.maybe_dangling.as_mut_ptr();
        let not_dangling = maybe_dangling.cast::<SelfRefSlot<'varying, N, S, E, &'erased ()>>();

        // SAFETY: this is basically `MaybeUninit::assume_init_mut`, except we can't use that exact
        // function, since the relevant `MaybeUninit` has the wrong lifetime *and* is behind a
        // `ManuallyDrop`. This is sound because, as per the safety invariant, `self.maybe_dangling`
        // is initialized (ignoring the `'erased` lifetime), and the caller asserts that using
        // a `'varying` lifetime is sound. Note that anything with an `'erased` lifetime is always
        // kept behind `MaybeUninit`.
        //
        // By-the-book requirements:
        unsafe { &mut *not_dangling }
    }
}

impl<'erased, N, S, E> Debug for LifetimeErase<'erased, N, S, E>
where
    S: for<'lower> CovariantFamily<'lower, &'erased ()>,
    Varying<'erased, 'erased, &'erased (), S>: Sized,
    E: for<'lower> CovariantFamily<'lower, &'erased ()>,
    Varying<'erased, 'erased, &'erased (), E>: Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Debugging the inner value would require `unsafe`.
        f.debug_struct("LifetimeErase").finish_non_exhaustive()
    }
}

impl<'erased, N, S, E> Drop for LifetimeErase<'erased, N, S, E>
where
    S: for<'lower> CovariantFamily<'lower, &'erased ()>,
    Varying<'erased, 'erased, &'erased (), S>: Sized,
    E: for<'lower> CovariantFamily<'lower, &'erased ()>,
    Varying<'erased, 'erased, &'erased (), E>: Sized,
{
    fn drop(&mut self) {
        let maybe_dangling = self.maybe_dangling.as_mut_ptr();
        // This `'_` lifetime can be inferred as a short lifetime within this function's body.
        // There's no way to explicitly require that the lifetime be that short, but between
        // this cast and using `drop_in_place` to avoid materializing a reference, the compiler
        // should not (possibly) wrongly assume that references within `self.maybe_dangling` are
        // dereferenceable for more than just this function body.
        let not_dangling = maybe_dangling.cast::<SelfRefSlot<'_, N, S, E, &'erased ()>>();

        // SAFETY:
        // We do not use `self.maybe_dangling` after this call to `drop_in_place`, either in this
        // destructor *or* in the drop glue for fields of `Self`, because `MaybeUninit` has no
        // drop glue.
        //
        // By-the-book requirements:
        // - `not_dangling` is valid for both reads and writes:
        //   - `not_dangling` is non-null, since the return value of `MaybeUninit::as_mut_ptr`
        //     is always non-null,
        //   - it is dereferenceable, as `maybe_dangling` is certainly valid for reads and writes
        //     of size `size_of::<SelfRefSlot<'erased, N, S, E, &'erased ()>>()` at this time,
        //     and changing the `'erased` lifetime to `'_` does not affect the size of the type.
        // - `not_dangling` is properly aligned, since the return value of
        //   `MaybeUninit::<SelfRefSlot<'erased, N, S, E, &'erased ()>>::as_mut_ptr` is
        //   properly aligned for `SelfRefSlot<'any, N, S, E, &'erased ()>`.
        // - `not_dangling` is non-null (see above).
        // - `not_dangling` must be a valid value of type `SelfRefSlot<'_, N, S, E, &'erased ()>`
        //   (by the safety invariant of this type, it's valid for some lifetime; by the safety
        //   contract of `LifetimeErase::erase`, which is the sole constructor for this type, it's
        //   valid for this `'_` lifetime limited to this function body). Therefore, it's valid
        //   for dropping.
        // - while `drop_in_place` is executing, nothing else accesses parts of `not_dangling`;
        //   since `not_dangling` is derived from an exclusive reference that we hold in this
        //   destructor, this condition could only be violated if other code had UB (perhaps
        //   leading to calling a destructor twice).
        unsafe {
            not_dangling.drop_in_place();
        };
    }
}
