trait Sealed {}

/// Provide implied bounds for a `'varying` lifetime, bounding it between
/// `'lower` and `'upper` lifetimes.
#[expect(private_bounds, reason = "intentionally creating a sealed trait")]
pub trait ImplyBound: Sealed {}

impl<'varying> Sealed for (&'varying &'_ (), &'_ &'varying ()) {}
impl<'varying> ImplyBound for (&'varying &'_ (), &'_ &'varying ()) {}

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

/// A "lifetime family" of types parameterized by a `'varying` lifetime such that the `'varying`
/// parameter can soundly be transmuted to *any* lifetime between `'lower` and `'upper`.
///
/// This is called bivariance: the ability to soundly perform either covariant or contravariant
/// casts on the lifetime. (In Rust, bivariance of a type parameter allows a type to be coerced
/// into any other type. For lifetimes, either `'a: 'b` or `'b: 'a` (or both), so covariance
/// and contravariance should cover every case.)
///
/// Trivial lifetime families which don't actually use the `'varying` parameter, such as `u32`,
/// implement this trait. Other examples include `struct Foo<'varying>(&'varying ())`, where `Foo`
/// unsafely promises not to attach any semantic importance to its `'varying` lifetime. (Otherwise,
/// perhaps `Foo` could signify proof that you have access to some other data for at least
/// lifetime `'varying`, or something, which would make bivariant casts potentially unsound.)
///
/// If you find any useful implementation of this trait which actually uses the `'varying`
/// parameter, let us know! There doesn't seem to be any useful way to use a `'varying` lifetime
/// which can be freely changed (for distinct `'lower` and `'upper`).
///
/// # Note on Lower Bound
/// While the maximally loose `'upper` bound is `'static`, there's no special lifetime which
/// serves as a lower bound for all other lifetimes. Instead,
/// `for<'lower> BivariantFamily<'lower, 'upper>` uses a maximally loose lower bound (and
/// implied bounds ensure that this works regardless of what `'upper` is).
///
/// # Safety of Use
/// In addition to all abilities provided by [`CovariantFamily`] and [`ContravariantFamily`],
/// if `T<'varying>` implements [`BivariantFamily<'lower, 'upper>`] and neither
/// [`CovariantFamily::covariant_assertions`] nor
/// [`ContravariantFamily::contravariant_assertions`] panic, then bivariant casts of the
/// `'varying` lifetime of `T<'varying>` can be performed (between `'lower` and `'upper`).
///
/// In particular, the `'varying` lifetime of `&'a mut T<'varying>` can be transmuted to any
/// lifetime between `'lower` and `'upper`, and the same goes when `T<'varying>` is in other
/// invariant positions.
///
/// # Safety of Implementation
/// This trait is effectively an alias for `CovariantFamily + ContravariantFamily`; all possible
/// implementations of this trait are already provided, and this itself trait is safe. Any
/// soundness burden of this trait is the respondibility of the impls of `CovariantFamily` and
/// `ContravariantFamily`.
///
/// # Further Details on Safety
/// Regardless of the finer details of soundness below, `variance-family` only non-recursively
/// implements both `CovariantFamily` and `ContravariantFamily` for lifetime families which leave
/// `'varying` entirely unused, such as `str`. Therefore, if any unsoundness manages to result
/// from a lifetime family that uses `'varying` (such as `Foo<'varying>(*mut &'varying ())`)
/// deciding to implement both `CovariantFamily` and `ContravariantFamily`, it technically wouldn't
/// be the fault of `variance-family`, except for reasoning in `variance-family` potentially
/// misleading the author of the unsound code.
///
/// The additional concern on top of `CovariantFamily + ContravariantFamily` is whether
/// `Invariant<T<'v1>>` can be soundly transmuted to `Invariant<T<'v2>>` where `Invariant<P>`
/// is invariant over its parameter `P` and `T<'varying>` does not leave `'varying` entirely
/// unused.
///
/// If a `T<'v2>` is read from an `Invariant<T<'v1>>` (via reading a `T<'v1>`) where `'v2: 'v1`,
/// that's a contravariant cast of `T<'v1>`, which is acceptable. If `'v1: 'v2`, it's a
/// covariant cast, which is also acceptable. If `Invariant<T<'v1>>` is instead used to write a
/// value of type `T<'v2>` (via writing a `T<'v1>`), acting similarly to `fn(T<'v1>)` or
/// `impl FnMut(T<'v1>) + 'a`, then the `T<'v2>` argument is effectively transmuted to `T<'v1>`
/// when passed. If `'v2: 'v1`, that's a covariant cast of `T<'v2`, which is acceptable. If
/// `'v1: 'v2`, it's a contravariant cast, which is also acceptable. Therefore, in all four
/// relevant cases where `Invariant<T<'varying>>` is used to read or write a `T<'varying>`
/// (possibly with a different lifetime), the cast should be sound solely based on the knowledge
/// that covariant and contravariant casts are permissible.
///
/// Note that there is another important case besides reads and writes: some typestate-ish
/// importance might be placed on the `'varying` lifetime. I'd argue that
/// <https://github.com/rust-lang/rust/issues/97156> is a problem because the compiler did not
/// respect invariance used for typestate; *even though* the relevant types are mutual subtypes
/// and (as far as I know) `&'c mut for<'a> fn(&'a (), &'a ())` can be soundly transmuted to
/// `&'c mut for<'a, 'b> fn(&'a (), &'b ())`, that does *not* mean that an *arbitrary other type*
/// with `for<'a> fn(&'a (), &'a ())` as a generic parameter used in an invariant position would
/// let that parameter be soundly changed.
///
/// Therefore, code transmuting `Invariant<T<'v1>>` to `Invariant<T<'v2>>` should be sound so
/// long as `Invariant<P>` doesn't place typestate-ish importance on `P` (that signifies the state
/// of data beyond `P` itself, which would be relevant to more than just reads and writes of
/// type `P`). Note that transmuting a type parameter would, in general, be able to change trait
/// implementations and associated types; meanwhile, a lifetime transmute can at best add or remove
/// a `'static` bound, which may affect the presence or absence of a trait implementation, but would
/// not be able to *change* between different trait implementations.
///
/// Additionally, nothing beyond `Invariant` should need to worry, because `BivariantFamily` isn't
/// an autotrait or something that would look into private fields used for typestate and mess with
/// them. As an example in the other direction, `&'a mut T` doesn't use `T` for typestate. Someone
/// who hands out a `&'a mut T<'v1>` and writes some `unsafe` code that somehow manages to escalate
/// potential existence of `&'a mut T<'v2>` into undefined behavior, perhaps with some sort of
/// `InvariantFamily<'lower, 'upper>`, would be making delicate assumptions in their `unsafe` as
/// well. Worst-case scenario, that code would be sound independently of and mutually incompatible
/// with assumptions made by `variance-family`; since such a situation is still hypothetical,
/// `BivariantFamily` and its safety comments seem sound, and no `unsafe` impls provided directly
/// by `variance-family` actually toe the line, I am satisfied with the assumptions of unsafe code
/// in `variance-family`.
pub trait BivariantFamily<'lower, 'upper>:
    CovariantFamily<'lower, 'upper> + ContravariantFamily<'lower, 'upper>
{}

impl<'lower, 'upper, T> BivariantFamily<'lower, 'upper> for T
where
    T: ?Sized + CovariantFamily<'lower, 'upper> + ContravariantFamily<'lower, 'upper>,
{}

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
/// - Note that if this lifetime family implements both `CovariantFamily` and `ContravariantFamily`,
///   then it will automatically implement [`BivariantFamily`] as well; it must be sound to
///   transmute the `'varying` lifetime of `T<'varying>` even in an invariant position
///   (provided that both `T::covariant_assertions` and `T::contravariant_assertions` do not
///   panic when called). In particular, be *very* wary of implementing both `CovariantFamily` and
///   `ContravariantFamily` for a type which uses `'varying` in some sort of typestate-ish way.
///   See [`BivariantFamily`] for more.
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
/// - Note that if this lifetime family implements both `CovariantFamily` and `ContravariantFamily`,
///   then it will automatically implement [`BivariantFamily`] as well; it must be sound to
///   transmute the `'varying` lifetime of `T<'varying>` even in an invariant position
///   (provided that both `T::covariant_assertions` and `T::contravariant_assertions` do not
///   panic when called). In particular, be *very* wary of implementing both `CovariantFamily` and
///   `ContravariantFamily` for a type which uses `'varying` in some sort of typestate-ish way.
///   See [`BivariantFamily`] for more.
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
