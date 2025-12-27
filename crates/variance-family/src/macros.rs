#[macro_export]
macro_rules! invariant_zst {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident<$($T:ident $(: ?$sized:ident)?),+>;
    ) => {
        // `fn` is a keyword, so there's no need for a `::core::primitive::` prefix or similar.
        $(#[$meta])*
        $vis struct $name<$($T $(: ?$sized)?),+>(
            ::core::marker::PhantomData<$(fn($T) -> $T),+>,
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
