<div align="center" class="rustdoc-hidden">
<h1> Variance Family </h1>
</div>

[<img alt="github" src="https://img.shields.io/badge/github-variance--family-08f?logo=github" height="20">](https://github.com/robofinch/variance-family/)
[![Latest version](https://img.shields.io/crates/v/variance-family.svg)](https://crates.io/crates/variance-family)
[![Documentation](https://img.shields.io/docsrs/variance-family)](https://docs.rs/variance-family/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Overview

Using `CovariantFamily`, `ContravariantFamily`, and `UnvaryingFamily`, you can require that a
lifetime be effectively covariant, effectively contravariant, or entirely unused in a family of
types parameterized by that lifetime.

The qualifier "effectively" on variance is present as `CovariantFamily` and `ContravariantFamily`
require that a lifetime be able to be cast covariantly or contravariantly; a lifetime which is
invariant but can be soundly (and perhaps even safely) cast to a different lifetime is perfectly
acceptable.

However, the following guarantees may be assumed by unsafe code:
- If a family of types implements `CovariantFamily` (or `ContravariantFamily`), then its covariant
  (or contravariant) lifetime may be soundly manipulated in a covariant (or contravariant) way via
  `transmute` and similar means. Using the methods of `CovariantFamily` (or `ContravariantFamily`)
  is unnecessary, and merely serve as *proof* of covariance (or contravariance).
- If a family of types implements `UnvaryingFamily`, then its lifetime is entirely unused in the
  family of types; that is, the "family" is trivial and only consists of a single type. The lifetime
  may therefore be freely changed via `transmute` and similar means.

# Non-static data

The lifetime family traits do not require that the family be paramterized by all possible lifetimes
(including `'static` or arbitrarily short lifetimes), which would pose an issue for lifetime
families like `&'varying &'a str` and `&'a &'varying str`; in such cases, `'varying` being either
`'static` or arbitrarily short could be invalid.

# Custom implementations

These types are not implemented exhaustively. In particular, this crate does not mess with
`dyn Trait + 'varying` or higher-ranked types with `for<'a>` binders. Moreover, third-party structs
and enums cannot be covered by `CovariantFamily` and `ContravariantFamily` implementations here.

In any case, when the families provided here do not suffice, you can create custom lifetime
families over whatever types you wish.

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
