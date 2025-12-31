/// Create a ZST which is invariant over one or more generic parameters.
///
/// Attributes (such as doc comments) may be placed on the struct. The struct may have any
/// visibility (e.g. `pub` or the default private visibility) or name. No bounds on the generic
/// parameters are supported other than optional `: ?Sized` bounds. Defaults for the generic
/// parameters are not supported.
///
/// The created ZST wraps [`PhantomData`](::core::marker::PhantomData) and implements a
/// variety of traits.
///
/// # Example
/// ```
/// use variance_family::invariant_zst;
///
/// invariant_zst!(
///     /// `Foo` is invariant over `T` and `U`.
///     ///
///     /// It unconditionally implements `Clone`, `Copy`, `Debug`, `Default`, `Eq`, `Hash`,
///     /// `Ord`, `PartialEq`, `PartialOrd`, `Send`, `Sync`, `Unpin`, etc.
///     ///
///     /// (Well, "unconditionally" as in "when the struct is well-formed", which requires
///     /// `U: Sized` in this case.)
///     // Show that a random attribute works
///     #[repr(align(64))]
///     pub(crate) struct Foo<T: ?Sized, U>;
/// );
///
/// impl<T: ?Sized, U> Foo<T, U> {
///     pub(crate) const fn new() -> Self {
///         Self(::core::marker::PhantomData)
///     }
/// }
/// ```
#[macro_export]
macro_rules! invariant_zst {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident<$($T:ident $(: ?$sized:ident)?),+>;
    ) => {
        // `fn` is a keyword, so there's no need for a `::core::primitive::` prefix or similar.
        $(#[$meta])*
        $vis struct $name<$($T $(: ?$sized)?),+>(
            ::core::marker::PhantomData<fn(($(*mut $T,)+))>,
        );

        impl<$($T $(: ?$sized)?),+> ::core::clone::Clone for $name<$($T),+> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<$($T $(: ?$sized)?),+> ::core::marker::Copy for $name<$($T),+> {}

        impl<$($T $(: ?$sized)?),+> ::core::fmt::Debug for $name<$($T),+> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::write!(
                    f,
                    "{}<{}>",
                    ::core::stringify!($name),
                    ::core::any::type_name::<T>(),
                )
            }
        }

        impl<$($T $(: ?$sized)?),+> ::core::default::Default for $name<$($T),+> {
            fn default() -> Self {
                Self(::core::marker::PhantomData)
            }
        }

        impl<$($T $(: ?$sized)?),+> ::core::cmp::Eq for $name<$($T),+> {}

        impl<$($T $(: ?$sized)?),+> ::core::hash::Hash for $name<$($T),+> {
            fn hash<H: ::core::hash::Hasher>(&self, _state: &mut H) {}
        }

        impl<$($T $(: ?$sized)?),+> ::core::cmp::Ord for $name<$($T),+> {
            fn cmp(&self, _other: &Self) -> ::core::cmp::Ordering {
                ::core::cmp::Ordering::Equal
            }
        }

        impl<$($T $(: ?$sized)?),+> ::core::cmp::PartialEq for $name<$($T),+> {
            fn eq(&self, _other: &Self) -> ::core::primitive::bool {
                true
            }
        }

        impl<$($T $(: ?$sized)?),+> ::core::cmp::PartialOrd for $name<$($T),+> {
            fn partial_cmp(&self, other: &Self) -> ::core::option::Option<::core::cmp::Ordering> {
                ::core::option::Option::Some(<Self as ::core::cmp::Ord>::cmp(self, other))
            }
        }
    };
}
