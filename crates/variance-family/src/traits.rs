
#![expect(unsafe_code, reason = "allow unsafe code to rely on these marker trait impls")]


/// A family of types which are parameterized by a `'varying` lifetime.
///
/// In order to support non-`'static` references interacting with `'varying` in complicated ways,
/// lower and upper bounds are placed on the possible lifetimes that `'varying` may be.
///
/// If an implementation does not need a `'lower` bound, the implementor should use an `'arbitrary`
/// lifetime in place of `'lower` and implement `LifetimeFamily<'arbitrary, 'upper>` for any
/// `'arbitrary` lifetime such that `'upper: 'arbitrary`.
///
/// If an implementation does not need an `'upper` bound, the implementor should use `'static`.
pub trait LifetimeFamily<'lower, 'upper> {
    type WithLifetime<'varying>: ?Sized where 'upper: 'varying, 'varying: 'lower;
}

/// A "lifetime family" of types parameterized by a `'varying` lifetime such that performing
/// covariant casts on the `'varying` lifetime is sound.
///
/// # Safety
/// For any implementation of this type, it must be sound to cast the lifetime of
/// `Self::WithLifetime<'varying>` to any shorter lifetime which is at least as long as `'lower`.
///
/// The cast may be performed via [`transmute`] or similar means, not necessarily via the
/// [`CovariantFamily::shorten`] and [`CovariantFamily::shorten_ref`] methods. Those methods are
/// included in order to help confirm that an implementation of this trait is sound. The methods
/// may include compile-time assertions (which may result in post-monomorphization errors),
/// though runtime assertions are pointless and possibly indicate an unsound implementation of this
/// type.
///
/// ## Examples
///
/// If `'varying` is genuinely covariant in the lifetime family, this trait can be soundly
/// implemented. For instance, `&'a &'varying str`, `&'varying &'a str`, and
/// `fn(&'a fn(&'varying str))` can soundly implement this trait with appropriate `'lower` and
/// `'upper` bounds.
///
/// If `'varying` is entirely unused in the lifetime family, meaning that the "family" consists of
/// a single type, this trait can be soundly implemented. Examples include `u8`, `[u8]`, and
/// `&'a [u8]`.
///
/// Additionally, `'varying` might not be covariant in the family, but it may still be sound to
/// implement this trait. A type might, for instance, gate any parts of its interface that would
/// normally rely on contravariance or invariance behind `unsafe` functions with safety comments
/// properly ensuring that a type can be treated as covariant. The below is a more trivial example
/// where the type does not actually rely on contravariance or invariance whatsoever.
///
/// ```
/// # use variance_family::{CovariantFamily, LifetimeFamily};
/// # use core::marker::PhantomData;
/// /// Warning: even though `'a` is invariant, covariant casts on `'a` are provided. Users should
/// /// not rely on this type's invariance in `'a`.
/// struct CouldBeCovariant<'a>(&'a str, PhantomData<fn(&'a ()) -> &'a ()>);
/// struct CouldBeCovariantFamily;
///
/// impl<'arbitrary> LifetimeFamily<'arbitrary, 'static> for CouldBeCovariantFamily {
///     // `'static: 'varying` does not need to be explicitly stated.
///     type WithLifetime<'varying> = CouldBeCovariant<'varying> where 'varying: 'arbitrary;
/// }
///
/// /// # Safety
/// /// `CouldBeCovariant<'varying>` can be treated as covariant in `'varying`; the invariance of
/// /// `'varying` is utterly unimportant for safety. Semantically, it varies the same as
/// /// `&'varying str`.
/// unsafe impl<'arbitrary> CovariantFamily<'arbitrary, 'static> for CouldBeCovariantFamily {
///     fn shorten<'l, 's>(long: Self::WithLifetime<'l>) -> Self::WithLifetime<'s>
///     where
///         'l: 's,
///         's: 'arbitrary,
///         Self::WithLifetime<'l>: Sized,
///     {
///         CouldBeCovariant(long.0, PhantomData)
///     }
///
///     fn shorten_ref<'l, 's>(long: &'s Self::WithLifetime<'l>) -> &'s Self::WithLifetime<'s>
///     where
///         'l: 's,
///         's: 'arbitrary,
///     {
///         let long: &'s CouldBeCovariant<'l> = long;
///         // SAFETY: this shortens the lifetime of the pointee. Shortening `&'l str` to
///         // `&'s str` is sound, since that's covariant; meanwhile, `PhantomData` is a ZST that
///         // attaches no semantic meaning to its type parameter. Additionally, `CouldBeCovariant`
///         // doesn't *actually* use the invariance of its lifetime for anything important.
///         // Moreover, to avoid the hypothetical situation where someone may use `CouldBeCovariant`
///         // to cause invariance of `'l` and rely on that invariance for correct semantics (or
///         // maybe even soundness), `CouldBeCovariant` documents that its lifetime parameter is
///         // treated as covariant.
///         let transmuted: &'s CouldBeCovariant<'s> = unsafe { core::mem::transmute(long) };
///         transmuted
///     }
/// }
/// ```
///
/// [`transmute`]: core::mem::transmute
pub unsafe trait CovariantFamily<'lower, 'upper>: LifetimeFamily<'lower, 'upper> {
    /// Soundly shorten the `'varying` lifetime of an owned `Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// Aside from compile-time assertions that may cause post-monomorphization errors, the
    /// function's body **MUST** be equivalent to `{ ::core::mem::transmute(long) }`.
    ///
    /// In particular, it is always sound to implement this function with the body `{ long }`,
    /// relying on implicit covariant coercion.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    fn shorten<'l, 's>(long: Self::WithLifetime<'l>) -> Self::WithLifetime<'s>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower,
        Self::WithLifetime<'l>: Sized;

    /// Soundly shorten the `'varying` lifetime of `&'varying Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// Aside from compile-time assertions that may cause post-monomorphization errors, the
    /// function's body **MUST** be equivalent to `{ ::core::mem::transmute(long) }`.
    ///
    /// In particular, it is always sound to implement this function with the body `{ long }`,
    /// relying on implicit covariant coercion.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    fn shorten_ref<'l, 's>(long: &'s Self::WithLifetime<'l>) -> &'s Self::WithLifetime<'s>
    where
        'upper: 'l,
        'l: 's,
        's: 'lower;
}

/// A "lifetime family" of types parameterized by a `'varying` lifetime such that performing
/// contravariant casts on the `'varying` lifetime is sound.
///
/// # Safety
/// For any implementation of this type, it must be sound to cast the lifetime of
/// `Self::WithLifetime<'varying>` to any longer lifetime which is at most as long as `'upper`.
///
/// The cast may be performed via [`transmute`] or similar means, not necessarily via the
/// [`ContravariantFamily::lengthen`] and [`ContravariantFamily::lengthen_ref`] methods. Those
/// methods are included in order to help confirm that an implementation of this trait is sound.
/// The methods may include compile-time assertions (which may result in post-monomorphization
/// errors), though runtime assertions are pointless and possibly indicate an unsound
/// implementation of this type.
///
/// ## Examples
///
/// If `'varying` is genuinely contravariant in the lifetime family, this trait can be soundly
/// implemented. For instance, `fn(&'a &'varying str)` and `fn(&'varying &'a str)` can soundly
/// implement this trait, with appropriate `'lower` and `'upper` bounds,
///
/// If `'varying` is entirely unused in the lifetime family, meaning that the "family" consists of
/// a single type, this trait can be soundly implemented. Examples include `u8`, `[u8]`, and
/// `&'a [u8]`.
///
/// Additionally, `'varying` might not be contravariant in the family, but it may still be sound to
/// implement this trait. A type might, for instance, gate any parts of its interface that would
/// normally rely on covariance or invariance behind `unsafe` functions with safety comments
/// properly ensuring that a type can be treated as contravariant.
pub unsafe trait ContravariantFamily<'lower, 'upper>: LifetimeFamily<'lower, 'upper> {
    /// Soundly lengthen the `'varying` lifetime of an owned `Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// Aside from compile-time assertions that may cause post-monomorphization errors, the
    /// function's body **MUST** be equivalent to `{ ::core::mem::transmute(long) }`.
    ///
    /// In particular, it is always sound to implement this function with the body `{ long }`,
    /// relying on implicit contravariant coercion.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    fn lengthen<'l, 's>(short: Self::WithLifetime<'s>) -> Self::WithLifetime<'l>
    where
        'upper: 'l,
        'l:     's,
        's:     'lower;

    /// Soundly lengthen the `'varying` lifetime of `&'varying Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// Aside from compile-time assertions that may cause post-monomorphization errors, the
    /// function's body **MUST** be equivalent to `{ ::core::mem::transmute(long) }`.
    ///
    /// In particular, it is always sound to implement this function with the body `{ long }`,
    /// relying on implicit contravariant coercion.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    fn lengthen_ref<'l, 's>(short: &'s Self::WithLifetime<'s>) -> &'l Self::WithLifetime<'l>
    where
        'upper: 'l,
        'l:     's,
        's:     'lower;
}
