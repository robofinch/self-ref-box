<div align="center" class="rustdoc-hidden">
<h1> Variance Family </h1>
</div>

[<img alt="github" src="https://img.shields.io/badge/github-variance--family-08f?logo=github" height="20">](https://github.com/robofinch/variance-family/)
[![Latest version](https://img.shields.io/crates/v/variance-family.svg)](https://crates.io/crates/variance-family)
[![Documentation](https://img.shields.io/docsrs/variance-family)](https://docs.rs/variance-family/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Motivation
Sometimes, you need to require that a generic type is covariant or contravariant over one of its
parameters.

If you need to require invariance, you can make a wrapper type that uses `PhantomData` to force
invariance over a generic parameter in your own structs or enums. There is no similar easy solution
for covariance or contravariance.

This crate enables you to place requirements on a generic type's variance over a `'varying`
lifetime. (Type parameters cannot easily be supported in a useful way without `for<T>` binders.)

# Overview

Using `CovariantFamily`, `ContravariantFamily`, and `UnvaryingFamily`, you can require that a
family of types parameterized by a `'varying` lifetime be effectively covariant over `'varying`,
be effectively contravariant over `'varying`, or leave `'varying` entirely unused.

Requiring invariance is not supported, since, as mentioned, forcing invariance does not require a
fancy trait.

### Definition of Variance
The qualifier "effectively" on variance is added in above descriptions of `CovariantFamily` and
`ContravariantFamily` to emphasize that this crate deals with the *soundness* of changing a
`'varying` lifetime, not with the variance assigned by the compiler. One may assume that
"covariant" or "contravariant" (in the context of Rust) refers to the variance assigned by the
compiler; however, some of the documentation throughout this crate uses a slightly broader notion.
Whenever "covariant" (or "contravariant") is genuinely intended to mean "the variance assigned by
the compiler is covariance (or contravariance)", that will be explicitly stated.

Essentially, `CovariantFamily` and `ContravariantFamily` require that the *implementor* of the
trait can prove that `'varying` can be soundly cast in a covariant or contravariant way, not that
the *compiler* can prove covariance or contravariance over `'varying`. A type parameterized by
`'varying` might be considered invariant over `'varying` by the compiler but carefully ensure that
the type's interface remains sound when `'varying` is changed covariantly; such a type can soundly
implement `CovariantFamily`.

### Guarantees for `unsafe` Code

If a family of types implements `CovariantFamily` (or `ContravariantFamily`), then its `'varying`
lifetime may be soundly manipulated in a covariant (or contravariant) way via
`transmute` and similar means *so long as `covariant_assertions()` (or
`contravariant_assertions()`) is called on that family and does not panic*. The latter constraint
allows for post-monomorphization errors.

### Guarantees for Safe Code

Using any safe means to change the `'varying` lifetime (including methods of `CovariantFamily`,
methods of `ContravariantFamily`, and the compiler's provided coercions) does not require calling
`covariant_assertions()` or `contravariant_assertions()`. (Such a requirement would unsoundly
place a safety requirement on safe code.) The assertion methods are only required for `unsafe`
code.

### `UnvaryingFamily`

`UnvaryingFamily` ensures with the type system that implementors do not use the `'varying` lifetime
whatsoever. Therefore, if you bound a generic `T<'varying>` lifetime family by `UnvaryingFamily`,
it is extremely likely that the compiler will let you freely coerce `T<'v1>` into `T<'v2>`
regardless of what the two lifetimes are. If implicit coercion does not work, `transmute` and
similar means can soundly transmute `T<'v1>` into `T<'v2>` even in an invariant position, such
as `*mut T<'varying>`.

# Non-`'static` Data

The lifetime family traits do not require that the family be parameterized by all possible lifetimes
(including `'static` or arbitrarily short lifetimes), which would pose an issue for lifetime
families like `&'varying &'a str` and `&'a &'varying str`; in such cases, `'varying` being either
`'static` or arbitrarily short could be invalid.

Each lifetime family comes with `'lower` and `'upper` bounds on how `'varying` is allowed to vary.
Those bounds are enforced through implied bounds, causing
```rust
for<'varying> Varying<'varying, 'lower, 'upper, T>
```
to behave like
```rust
for<'varying where 'upper: 'varying, 'varying: 'lower> Varying<'varying, 'lower, 'upper, T>
```

Note that you may sometimes wish to require *no* upper bound or lower bound. `'static` is the
maximally loose upper bound, but there is no special lifetime shorter than all other lifetimes;
instead, `for<'lower> CovariantFamily<'lower, 'upper>` effectively removes the lower bound.
(This, too, automagically acts like it had a `for<'lower where 'upper: 'lower>` binder.)

## Caution for Implied Bounds

Currently, in some situations involving higher-ranked function pointers, the compiler can neglect
to enforce implied bounds, resulting in soundness. This known bug is tracked at
<https://github.com/rust-lang/rust/issues/25860>. Higher-ranked `dyn` trait objects for impossible
traits can also be created, as mentioned in <https://github.com/rust-lang/rust/issues/84533>.
This crate does not implement any of its traits for higher-ranked types, and none of its
traits are `dyn`-compatible; therefore, this crate itself should not come close to triggering
the unsoundness. Nevertheless, for the sake of caution, it is worth keeping this potential risk in
mind when working with higher-ranked types alongside this crate.

# Custom implementations

These types are not implemented exhaustively. In particular, third-party structs and enums
cannot be covered by `CovariantFamily` and `ContravariantFamily` implementations here, and
exhaustively implementing traits for types in `std` is not a goal of this crate. Instead,
when the lifetime families provided here are not sufficient, utilities are provided for soundly
creating custom lifetime families over whatever types you wish (including types not defined in
the same crate as the custom lifetime family).

TODO: (create and) discuss helper macros and composing lifetime families together

# License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE][])
* MIT license ([LICENSE-MIT][])

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.

[LICENSE-APACHE]: LICENSE-APACHE
[LICENSE-MIT]: LICENSE-MIT
