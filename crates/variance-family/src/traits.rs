/// A private trait for sealing `ImplyBound`.
trait Sealed {
    /// Ensure that `ImplyBound` is not `dyn`-compatible in order to head off any concerns about
    /// interactions between higher-ranked `dyn` trait objects and implied bounds.
    #[expect(dead_code, reason = "removes `dyn`-compatibility without requiring `Sized`")]
    fn remove_dyn_compatibility() {}
}

/// Provide implied bounds for a `'varying` lifetime, bounding it between
/// `'lower` and `'upper` lifetimes.
#[expect(private_bounds, reason = "intentionally creating a sealed trait")]
pub trait ImplyBound: Sealed {}

impl<'varying> Sealed for (&'varying &'_ (), &'_ &'varying ()) {}
impl<'varying> ImplyBound for (&'varying &'_ (), &'_ &'varying ()) {}

/// An uninhabited type for sealing a `WithLifetime` method.
enum PrivateSeal {}

/// Apply a `'varying` lifetime to a family of types, and provide implied bounds that
/// bound `'varying` between `'lower` and `'upper`.
///
/// ## Lifetimes
///
/// The trait should be implemented for as many values of `'lower` and `'upper` as possible. In
/// particular, even if an implementation does not need a nontrivial `'upper` bound, do not solely
/// implement the trait for `'upper = 'static` (unless it's required that `'lower: 'static`).
///
/// Preserving maximum flexibility in lifetimes is important, as implementing
/// `for<'varying, 'any> WithLifetime<'varying, 'any, 'static>` does not automatically imply
/// implementations of `WithLifetime` for any other combinations of lifetimes, even though,
/// semantically, we can reason that `for<'varying, 'any> WithLifetime<'varying, 'any, 'static>`
/// applies maximally loose lower and upper bounds on `'varying` and should allow for upper bounds
/// shorter than `'static`.
///
/// ## Why not a GAT
///
/// This trait is very similar to a generic associated type (GAT):
/// ```
/// pub trait LifetimeFamily<'lower, 'upper> {
///     type WithLifetime<'varying>: ?Sized
///     where
///         'upper: 'varying,
///         'varying: 'lower;
/// }
/// ```
///
/// However, `for<'varying> <T as LifetimeFamily<'lower, 'upper>>::WithLifetime<'varying>: ..Bounds`
/// would not work very well; the `for<'varying>` binder may still attempt to quantify over
/// lifetimes shorter than `'lower` and longer than `'upper`. For some reason, as of Rust 1.90.0,
/// the `for<'varying> ..: ..Bounds` bound would compile. However, any attempts to *use*
/// whatever has that bound would fail with an opaque "higher-ranked lifetime error".
///
/// In short, `for<'varying> ..` bounds do not work even remotely well with a GAT, greatly
/// limiting any nontrivial uses of a `LifetimeFamily`.
///
/// With this trait's use of implied bounds,
/// `for<'varying> <T as WithLifetime<'varying, 'lower, 'upper>>::Is: ..Bounds` quantifies only
/// over `'varying` lifetimes between `'lower` and `'upper`.
///
/// ## Alias
///
/// Note that `<T as WithLifetime<'varying, 'lower, 'upper>>::Is` is also available as a
/// [`Varying<'varying, 'lower, 'upper, T>`] alias (which is 13 characters shorter, and perhaps
/// easier to read and write).
pub trait WithLifetime<
    'varying, 'lower, 'upper,
    __ImplyBound: ImplyBound = (&'varying &'upper (), &'lower &'varying ()),
> {
    type Is: ?Sized;

    /// Ensure that `WithLifetime` and `LifetimeFamily` are not `dyn`-compatible in order
    /// to head off any concerns about interactions between higher-ranked `dyn` trait objects
    /// and implied bounds.
    #[doc(hidden)]
    #[expect(
        private_interfaces,
        reason = "seal this method; nobody should bother implementing it",
    )]
    fn remove_dyn_compatibility(_seal: PrivateSeal) {}
}

/// A slightly shorter and more legible alias for
/// `<T as WithLifetime<'varying, 'lower, 'upper>>::Is`.
pub type Varying<'varying, 'lower, 'upper, T> = <T as WithLifetime<'varying, 'lower, 'upper>>::Is;

/// A family of types which are parameterized by a `'varying` lifetime.
///
/// In order to support non-`'static` references interacting with `'varying` in complicated ways,
/// lower and upper bounds are placed on the possible lifetimes that `'varying` may be.
///
/// You should ensure that users of your implementation can use weaker lifetime bounds. In
/// particular, provide the strongest guarantees you can (implement `WithLifetime` with as many
/// lifetime values as possible, including weaker / more restrictive bounds) and use the weakest
/// bounds you can (as few lifetime values as possible) when bounding by `LifetimeFamily`.
///
/// Note that this trait is effectively a trait alias for
/// `for<'varying> WithLifetime<'varying, 'lower, 'upper>`; all possible implementations of this
/// trait are provided, and you should implement [`WithLifetime`] for your types.
pub trait LifetimeFamily<'lower, 'upper>: for<'varying> WithLifetime<'varying, 'lower, 'upper> {}

impl<'lower, 'upper, T> LifetimeFamily<'lower, 'upper> for T
where
    T: ?Sized + for<'varying> WithLifetime<'varying, 'lower, 'upper>,
{}

/// A trivial "lifetime family" of types parameterized by a `'varying` lifetime which don't
/// actually use the `'varying` parameter.
///
/// For any `'varying` lifetime between `'lower` and `'upper`, the type
/// `Varying<'varying, 'lower, 'upper, Self>` is simply equal to `Self::WithAnyLifetime`.
///
/// All possible implementations of this trait are already provided.
///
/// # Note on Lower Bound
/// While the maximally loose `'upper` bound is `'static`, there's no special lifetime which
/// serves as a lower bound for all other lifetimes. Instead,
/// `for<'lower> UnvaryingFamily<'lower, 'upper>` uses a maximally loose lower bound (and
/// implied bounds ensure that this works regardless of what `'upper` is).
pub trait UnvaryingFamily<'lower, 'upper>:
    LifetimeFamily<'lower, 'upper>
        + for<'varying> WithLifetime<'varying, 'lower, 'upper, Is = Self::WithAnyLifetime>
{
    type WithAnyLifetime: ?Sized;
}

impl<'lower, 'upper, T, U> UnvaryingFamily<'lower, 'upper> for T
where
    T: ?Sized
        + LifetimeFamily<'lower, 'upper>
        + for<'varying> WithLifetime<'varying, 'lower, 'upper, Is = U>,
    U: ?Sized,
{
    type WithAnyLifetime = U;
}

/// A "lifetime family" of types parameterized by a `'varying` lifetime such that performing
/// covariant casts on the `'varying` lifetime is sound.
///
/// Note that "being able to be covariantly casted" is a slightly broader condition than
/// "being covariant (as far as the compiler is concerned)". See the Examples section. In
/// documentation throughout this crate, "covariance" may actually refer to
/// "the ability to soundly be covariantly casted" instead of the variance assigned by the compiler.
///
/// # Note on Lower Bound
/// While the maximally loose `'upper` bound is `'static`, there's no special lifetime which
/// serves as a lower bound for all other lifetimes. Instead,
/// `for<'lower> CovariantFamily<'lower, 'upper>` uses a maximally loose lower bound (and
/// implied bounds ensure that this works regardless of what `'upper` is).
///
/// As covariant lifetimes are usually freely shrinkable (such as `&'varying mut [u8]`) with
/// only unusual exceptions (such as `&'a &'varying u8`, which requires `'varying: 'a`), common
/// use cases will likely require `for<'lower> CovariantFamily<'lower, 'upper>` bounds.
///
/// # Safety of Use
/// Code can always use safe methods to change the `'varying` lifetime, including
/// [`CovariantFamily::shorten`], [`CovariantFamily::shorten_ref`], and the compiler's
/// covariant coercion.
///
/// However, before performing any covariant casts on the `'varying` lifetime through `unsafe`
/// means (such as [`transmute`]), the [`CovariantFamily::covariant_assertions`] must be called
/// and not panic. The other two methods are not guaranteed to internally call
/// `covariant_assertions`.
///
/// # Implementation
///
/// **You should probably not need to directly and unsafely implement this trait.**
///
/// The `variance-family` crate includes a large number of `unsafe` implementations of the marker
/// traits for generic types for the sake of ergonomics for users -- in particular, for the sake
/// of limiting how many times that others must unsafely implement the marker traits. When that
/// does not suffice, there are also many helper macros.
///
/// You should first try to express your desired lifetime as a composition of other lifetime
/// families, such as `(Cow<'a, str>, &'varying mut [u8], MyStruct)` becoming
/// `(Cow<'a, str>, VaryingRefMut<[u8]>, Unvarying<MyStruct>)`.
///
// TODO: describe next steps.
///
/// # Implementation Safety
///
/// The following three conditions must be met.
///
/// - If [`CovariantFamily::covariant_assertions`] does not panic, then `'varying` must be sound
///   to cast covariantly in `T<'varying>` (where `T<'varying>` is shorthand for
///   `Varying<'varying, 'lower, 'upper, T>`, and `'varying` is bounded by `'lower` and `'upper`).
///
/// - No assertions not included within `covariant_assertions` may be used.
///
/// - The implementation safety requirements of `shorten` and `shorten_ref` must be met.
///
/// ## Precise Elaboration
/// For any implementation of this type, it must be sound to cast the `'varying` lifetime of
/// `Varying<'varying, 'lower, 'upper, T>` to any shorter lifetime which is at least as long as
/// `'lower`.
///
/// Compile-time assertions (possibly resulting in post-monomorphization errors) may be placed
/// in [`CovariantFamily::covariant_assertions`], which serve as additional preconditions for the
/// family of types being covariant. Runtime assertions could also be included there, though their
/// utility would seem questionable.
///
/// Provided that `covariant_assertions` does not panic, covariant casts on `'varying` may be
/// performed via [`transmute`] or similar means, not necessarily via the
/// [`CovariantFamily::shorten`] and [`CovariantFamily::shorten_ref`] methods.
/// `shorten` and `shorten_ref` are provided in part for ergonomics and in part to help confirm
/// that an implementation of this trait is sound.
///
/// # Examples
///
/// If the compiler considers the lifetime family to be covariant over `'varying`, then this trait
/// can be soundly implemented. For instance, `&'a &'varying str`, `&'varying &'a str`, and
/// `fn(&'a fn(&'varying str))` can soundly implement this trait with appropriate `'lower` and
/// `'upper` bounds.
///
/// If `'varying` is entirely unused in the lifetime family, meaning that the "family" consists of
/// a single type, this trait can be soundly implemented. Examples include `u8`, `[u8]`, and
/// `&'a [u8]`.
///
/// Additionally, the family might have some non-covariant variance over `'varying` assigned by the
/// compiler, but it may still be sound to implement this trait. A type might, for instance, gate
/// any parts of its interface that would normally rely on contravariance or invariance behind
/// `unsafe` functions with safety comments properly ensuring that a type can be treated as
/// covariant. The below is a more trivial example where the type does not actually rely on
/// contravariance or invariance whatsoever.
///
/// ```
/// # use variance_family::{CovariantFamily, LifetimeFamily, WithLifetime, Varying};
/// # use core::marker::PhantomData;
/// /// # Variance
/// /// Even though `'a` is invariant, covariant casts on `'a` are provided.
/// /// Users should not rely on this type's invariance over `'a`.
/// struct CouldBeCovariant<'a>(&'a str, PhantomData<fn(&'a ()) -> &'a ()>);
/// struct CouldBeCovariantFamily;
///
/// impl<'varying> WithLifetime<'varying, '_, '_> for CouldBeCovariantFamily {
///     type Is = CouldBeCovariant<'varying>;
/// }
///
/// /// # Safety
/// /// `CouldBeCovariant<'varying>` can be treated as covariant over `'varying`; the invariance of
/// /// `'varying` is utterly unimportant for safety. Semantically, it varies the same as
/// /// `&'varying str`.
/// unsafe impl<'lower, 'upper> CovariantFamily<'lower, 'upper> for CouldBeCovariantFamily {
///     /// Performs no assertions.
///     #[inline]
///     fn covariant_assertions() {}
///
///     #[inline]
///     fn shorten<'l, 's>(
///         long: Varying<'l, 'lower, 'upper, Self>,
///     ) -> Varying<'s, 'lower, 'upper, Self>
///     where
///         'upper: 'l,
///         'l: 's,
///         's: 'lower,
///         for<'varying> Varying<'varying, 'lower, 'upper, Self>: Sized,
///     {
///         CouldBeCovariant(long.0, PhantomData)
///     }
///
///     #[inline]
///     fn shorten_ref<'l, 's, 'r>(
///         long: &'r Varying<'l, 'lower, 'upper, Self>,
///     ) -> &'r Varying<'s, 'lower, 'upper, Self>
///     where
///         'upper: 'l,
///         'l: 's,
///         's: 'lower,
///         Varying<'l, 'lower, 'upper, Self>: 'r,
///         Varying<'s, 'lower, 'upper, Self>: 'r,
///     {
///         let long: &'r CouldBeCovariant<'l> = long;
///         // SAFETY: this shortens the lifetime of the pointee. Shortening `&'l str` to
///         // `&'s str` is sound, since that's covariant; meanwhile, `PhantomData` is a ZST that
///         // attaches no semantic meaning to its type parameter. Additionally, `CouldBeCovariant`
///         // doesn't *actually* use the invariance of its lifetime for anything important.
///         // Moreover, to avoid the hypothetical situation where someone may use
///         // `CouldBeCovariant` to cause invariance of `'l` and rely on that invariance for
///         // correct semantics (or maybe even soundness) in some way that would be broken by
///         // this cast, `CouldBeCovariant` documents that its lifetime parameter is
///         // treated as covariant. It's on authors of other `unsafe` code to read its docs.
///         let transmuted: &'r CouldBeCovariant<'s> = unsafe { core::mem::transmute(long) };
///         transmuted
///     }
/// }
/// ```
///
/// [`transmute`]: core::mem::transmute
pub unsafe trait CovariantFamily<'lower, 'upper>: LifetimeFamily<'lower, 'upper> {
    /// Perform compile-time assertions, which may cause post-monomorphization errors.
    ///
    /// (The function could, hypothetically, also include runtime assertions.)
    // TODO: `const` block example
    #[inline]
    fn covariant_assertions() {}

    /// Soundly shorten the `'varying` lifetime of an owned `Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// It is always sound to implement this function with the body `{ long }`,
    /// relying on implicit covariant coercion (if possible).
    ///
    /// The function's body **MUST** be equivalent to
    /// ```
    /// # struct Foo;
    /// # impl Foo {
    /// #     fn subset_of_assertions_in_covariant_assertions() {}
    /// #     fn shorten(long: u8) -> u8
    /// {
    ///     // Usually just `Self::covariant_assertions();`
    ///     Self::subset_of_assertions_in_covariant_assertions();
    ///     // SAFETY: ..
    ///     unsafe { ::core::mem::transmute(long) }
    /// }
    /// # }
    /// ```
    ///
    /// Any assertions (or other possible causes of panics) in `Self::shorten` must be included in
    /// `Self::covariant_assertions()`.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    #[must_use]
    fn shorten<'l, 's>(
        long: Varying<'l, 'lower, 'upper, Self>,
    ) -> Varying<'s, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, 'upper, Self>: Sized;

    /// Soundly shorten the `'varying` lifetime of `&Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// It is always sound to implement this function with the body `{ long }`,
    /// relying on implicit covariant coercion (if possible).
    ///
    /// The function's body **MUST** be equivalent to
    /// ```
    /// # struct Foo;
    /// # impl Foo {
    /// #     fn subset_of_assertions_in_covariant_assertions() {}
    /// #     fn shorten(long: u8) -> u8
    /// {
    ///     // Usually just `Self::covariant_assertions();`
    ///     Self::subset_of_assertions_in_covariant_assertions();
    ///     // SAFETY: ..
    ///     unsafe { ::core::mem::transmute(long) }
    /// }
    /// # }
    /// ```
    ///
    /// Any assertions (or other possible causes of panics) in `Self::shorten` must be included in
    /// `Self::covariant_assertions()`.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    #[must_use]
    fn shorten_ref<'l, 's, 'r>(
        long: &'r Varying<'l, 'lower, 'upper, Self>,
    ) -> &'r Varying<'s, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'l, 'lower, 'upper, Self>: 'r,
        Varying<'s, 'lower, 'upper, Self>: 'r;
}

/// A "lifetime family" of types parameterized by a `'varying` lifetime such that performing
/// contravariant casts on the `'varying` lifetime is sound.
///
/// Note that "being able to be contravariantly casted" is a slightly broader condition than
/// "being contravariant". See the Examples section. In documentation throughout this crate,
/// "contravariance" may actually refer to "the ability to soundly be contravariantly casted"
/// instead of the variance assigned by the compiler.
///
/// # Note on Lower Bound
/// While the maximally loose `'upper` bound is `'static`, there's no special lifetime which
/// serves as a lower bound for all other lifetimes. Instead,
/// `for<'lower> ContravariantFamily<'lower, 'upper>` uses a maximally loose lower bound (and
/// implied bounds ensure that this works regardless of what `'upper` is).
///
/// # Safety of Use
/// Code can always use safe methods to change the `'varying` lifetime, including
/// [`ContravariantFamily::lengthen`], [`ContravariantFamily::lengthen_ref`], and the compiler's
/// contravariant coercion.
///
/// However, before performing any contravariant casts on the `'varying` lifetime through `unsafe`
/// means (such as [`transmute`]), the [`ContravariantFamily::contravariant_assertions`] method
/// must be called and not panic. The other two methods are not guaranteed to internally call
/// `contravariant_assertions`.
///
/// # Implementation
///
/// **You should probably not need to directly and unsafely implement this trait.**
///
/// The `variance-family` crate includes a large number of `unsafe` implementations of the marker
/// traits for the sake of ergonomics for users -- in particular, for the sake
/// of limiting how many times that others must unsafely implement the marker traits. When that
/// does not suffice, there are also many helper macros.
///
/// You should first try to express your desired lifetime as a composition of other lifetime
/// families, such as `(Cow<'a, str>, fn(&'varying mut [u8]) -> MyStruct)` becoming
/// `(Cow<'a, str>, fn(VaryingRefMut<[u8]>) -> Unvarying<MyStruct>)`.
///
// TODO: describe next steps.
///
/// # Implementation Safety
/// The following three conditions must be met.
///
/// - If [`ContravariantFamily::contravariant_assertions`] does not panic, then `'varying` must be
///   sound to cast contravariantly in `T<'varying>` (where `T<'varying>` is shorthand for
///   `Varying<'varying, 'lower, 'upper, T>`, and `'varying` is bounded by `'lower` and `'upper`).
///
/// - No assertions not included within `contravariant_assertions` may be used.
///
/// - The implementation safety requirements of `lengthen` and `lengthen_ref` must be met.
///
/// ## Precise Elaboration
/// For any implementation of this type, it must be sound to cast the `'varying` lifetime of
/// `Varying<'varying, 'lower, 'upper, T>` to any longer lifetime which is at most as long as
/// `'upper`.
///
/// Compile-time assertions (possibly resulting in post-monomorphization errors) may be placed
/// in [`ContravariantFamily::contravariant_assertions`], which serve as additional preconditions
/// for the family of types being contravariant. Runtime assertions could also be included there,
/// though their utility would seem questionable.
///
/// Provided that `contravariant_assertions` does not panic, contravariant casts on `'varying` may
/// be performed via [`transmute`] or similar means, not necessarily via the
/// [`ContravariantFamily::lengthen`], [`ContravariantFamily::lengthen_ref`] methods.
/// `lengthen` and `lengthen_ref` are provided in part for ergonomics and in part to help confirm
/// that an implementation of this trait is sound.
///
/// ## Examples
///
/// If the compiler considers the lifetime family to be contravarint over `'varying`, then this
/// trait can be soundly implemented. For instance, `fn(&'a &'varying str)` and
/// `fn(&'varying &'a str)` can soundly implement this trait, with appropriate `'lower` and
/// `'upper` bounds.
///
/// If `'varying` is entirely unused in the lifetime family, meaning that the "family" consists of
/// a single type, this trait can be soundly implemented. Examples include `u8`, `[u8]`, and
/// `&'a [u8]`.
///
/// Additionally, the family might have some non-contravariant variance over `'varying` assigned by
/// the compiler, but it may still be sound to implement this trait. A type might, for instance,
/// gate any parts of its interface that would normally rely on covariance or invariance behind
/// `unsafe` functions with safety comments properly ensuring that a type can be treated as
/// contravariant.
///
/// [`transmute`]: core::mem::transmute
pub unsafe trait ContravariantFamily<'lower, 'upper>: LifetimeFamily<'lower, 'upper> {
    /// Perform compile-time assertions, which may cause post-monomorphization errors.
    ///
    /// (The function could, hypothetically, also include runtime assertions.)
    // TODO: `const` block example
    #[inline]
    fn contravariant_assertions() {}

    /// Soundly lengthen the `'varying` lifetime of an owned `Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// It is always sound to implement this function with the body `{ short }`,
    /// relying on implicit contravariant coercion (if possible).
    ///
    /// The function's body **MUST** be equivalent to
    /// ```
    /// # struct Foo;
    /// # impl Foo {
    /// #     fn subset_of_assertions_in_contravariant_assertions() {}
    /// #     fn lengthen(short: u8) -> u8
    /// {
    ///     // Usually just `Self::contravariant_assertions();`
    ///     Self::subset_of_assertions_in_contravariant_assertions();
    ///     // SAFETY: ..
    ///     unsafe { ::core::mem::transmute(short) }
    /// }
    /// # }
    /// ```
    ///
    /// Any assertions (or other possible causes of panics) in `Self::lengthen` must be included in
    /// `Self::contravariant_assertions()`.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    #[must_use]
    fn lengthen<'s, 'l>(
        short: Varying<'s, 'lower, 'upper, Self>,
    ) -> Varying<'l, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l:     's,
        's:     'lower,
        for<'varying> Varying<'varying, 'lower, 'upper, Self>: Sized;

    /// Soundly lengthen the `'varying` lifetime of `&Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// It is always sound to implement this function with the body `{ short }`,
    /// relying on implicit contravariant coercion (if possible).
    ///
    /// The function's body **MUST** be equivalent to
    /// ```
    /// # struct Foo;
    /// # impl Foo {
    /// #     fn subset_of_assertions_in_contravariant_assertions() {}
    /// #     fn lengthen(short: u8) -> u8
    /// {
    ///     // Usually just `Self::contravariant_assertions();`
    ///     Self::subset_of_assertions_in_contravariant_assertions();
    ///     // SAFETY: ..
    ///     unsafe { ::core::mem::transmute(short) }
    /// }
    /// # }
    /// ```
    ///
    /// Any assertions (or other possible causes of panics) in `Self::lengthen` must be included in
    /// `Self::contravariant_assertions()`.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    #[must_use]
    fn lengthen_ref<'s, 'l, 'r>(
        short: &'r Varying<'s, 'lower, 'upper, Self>,
    ) -> &'r Varying<'l, 'lower, 'upper, Self>
    where
        'upper: 'l,
        'l:     's,
        's:     'lower,
        Varying<'s, 'lower, 'upper, Self>: 'r,
        Varying<'l, 'lower, 'upper, Self>: 'r;
}
