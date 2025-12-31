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

Using `CovariantFamily`, `ContravariantFamily`, and `BivariantFamily`, you can require that a
family of types parameterized by a `'varying` lifetime be effectively covariant over `'varying`,
be effectively contravariant over `'varying`, or both (that is, allow for covariant casts
and contravariant casts).

Any variance other than invariance is supported, since, as discussed, forcing invariance
does not require a fancy trait. Moreover, bounding a generic parameter by "the compiler must
indicate that this parameter is invariant" is akin to a `!Send` bound and is unlikely to provide
any useful information. Covariance and contravariance give you additive abilities, similar to
`Send` and `Sync`; the *presence* of a `Send` implementation implies that it's definitely sound to
send a value of that type to a different thread, but the *absence* of a `Send` implementation does
not guarantee that sending a value of the type to a different thread is necessarily unsound.

Likewise, the lack of an implementation of `CovariantFamily` or `ContravariantFamily` (and even
the compiler indicating that a type family is invariant over `'varying`) is insufficient to imply
that `'varying` cannot soundly be covariantly or contravariantly changed in a `T<'varying>` lifetime
family.

### Definition of variance
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

### Guarantees for `unsafe` code

The following guarantees may be assumed by unsafe code:
- If a family of types implements `CovariantFamily` (or `ContravariantFamily`), then its `'varying`
  lifetime may be soundly manipulated in a covariant (or contravariant) way via
  `transmute` and similar means *so long as `covariant_assertions()` (or
  `contravariant_assertions()`) is called on that family and does not panic*. The latter constraint
  allows for post-monomorphization errors. Using the methods of `CovariantFamily`
  (or `ContravariantFamily`) is unnecessary for `unsafe` code (though useful for safe code), and
  merely serve as *proof* of covariance (or contravariance).
- If a family of types implements `BivariantFamily`, then in addition to abilities provided by its
  `CovariantFamily` and `ContravariantFamily` supertraits, the `'varying` lifetime of the family
  may be soundly changed via `transmute` and similar means (even when the parameterized type
  is in an invariant position, like `*mut T<'varying>`), so long as both `covariant_assertions()`
  and `contravariant_assertions()` are called on that family and do not panic.

### Guarantees for safe code

Using any safe means to change the `'varying` lifetime (including methods of `CovariantFamily`,
methods of `ContravariantFamily`, and the compiler's provided coercions) does not require calling
`covariant_assertions()` or `contravariant_assertions()`. (Such a requirement would unsoundly
place a safety requirement on safe code.) The assertion methods are only required for `unsafe`
code.

### Bivariance and Invariance

Note that "bivariance" (at least in a type parameter) allows casts that are either covariant or
contravariant, and "invariance" allows, at best, casts that are both covariant and contravariant.
Usually, lifetime families that allow for bivariant casts of `'varying` do not actually care about
the `'varying` lifetime at all (and likely do not use it whatsoever, as in the case of the `u32`
lifetime family), while invariant casts are generally trivial and do not usually change anything.

However, some pairs of types in Rust are considered distinct despite being mutual subtypes of
each other, allowing for covariant and contravariant casts between the types in either direction;
if invariance over a generic parameter still allows for that parameter to be cast in a way
which is both covariant and contravariant, then such an invariant cast could actually change the
type of the generic parameter. This lead to unsoundness in
<https://github.com/rust-lang/rust/issues/97156>, where an invariant parameter is used for
a typestate-like purpose. Therefore, invariant parameters in Rust cannot generally be changed to a
non-equal type, even if the cast of the parameter would be both covariant and contravariant.

It seems that so long as `Invariant<Bivariant<P>>` does not attach typestate-like significance to
the precise type (or `TypeId`) of an invariant parameter `Bivariant<P>`, then changing `P` to `Q`
in `Invariant<Bivariant<P>>` is sound as long as casting `Bivariant<P>` to `Bivariant<Q>` is both a
covariant cast and a contravariant cast. The latter condition holds when `Bivariant<P>` is
bivariant over its parameter `P`. This implies that `Invariant<Bivariant<P>>` is bivariant over
`P` *provided that* `Invariant<_>` is not doing something interesting with typestate.

Also, note that bivariance of a generic type parameter allows coercion of that parameter into any
type, which could be considered to be *transitive* covariance + contravariance (assuming the
existence of some uninhabited bottom type, similar to `!`, which can be cast into any other type).
At least in the case of lifetimes, either `'a: 'b` or `'b: 'a` (or both), so so covariance and
contravariance should cover every case.

# Non-static data

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

Note that the usage of implied bounds in this crate should not come close to triggering the
unsoundness shown in <https://github.com/rust-lang/rust/issues/25860>; this crate doesn't
implement any of its traits for higher-ranked function pointers (or higher-ranked `dyn` trait
objects).
TODO: should `LifetimeFamily` be made non-dyn-compatible, in case `dyn LifetimeFamily<..>`
could cause problems?

# Custom implementations

These types are not implemented exhaustively. In particular, this crate does not implement its
traits for `dyn Trait + 'varying` or higher-ranked types with `for<'a>` binders. Moreover,
third-party structs and enums cannot be covered by `CovariantFamily` and `ContravariantFamily`
implementations here.

In any case, when the families provided here do not suffice, you can create custom lifetime
families over whatever types you wish.

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
