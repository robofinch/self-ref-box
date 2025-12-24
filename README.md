# Self-Ref Box

[<img alt="github" src="https://img.shields.io/badge/github-self--ref--box-08f?logo=github" height="20">](https://github.com/robofinch/self-ref-box/)
[![Latest version](https://img.shields.io/crates/v/self-ref-box.svg)](https://crates.io/crates/self-ref-box)
[![Documentation](https://img.shields.io/docsrs/self-ref-box)](https://docs.rs/self-ref-box/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Variance Family

[<img alt="github" src="https://img.shields.io/badge/github-variance--family-08f?logo=github" height="20">](https://github.com/robofinch/variance-family/)
[![Latest version](https://img.shields.io/crates/v/variance-family.svg)](https://crates.io/crates/variance-family)
[![Documentation](https://img.shields.io/docsrs/variance-family)](https://docs.rs/variance-family/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Overview

The central `SelfRefBox` type is based on the following idea (setting aside finer details related to
lifetimes, traits, and soundness):

```rust
struct SelfRefBox<T, S, E> {
    self_ref_slot: SelfRefSlot<S, E>,
    backing_data:  AliasableBox<T>,
}

enum SelfRefSlot<S, E> {
    None,
    /// Lifetime-erased self-reference produced from a `&T`
    Shared(S),
    /// Lifetime-erased self-reference produced from a `&mut T`
    Exclusive(E),
}
```

The type is designed for the self-reference to be frequently changed out and/or mutated, with
access to the backing data provided to the greatest extent that is sound.

See the [`self-ref-box` `README`](crates/self-ref-box/README.md) for more details.

The [`variance-family` crate](crates/variance-family/README.md) was created to support this type,
and may be more broadly useful for requiring that a lifetime is covariant, contravariant, or
entirely unused in a family of types parameterized by a lifetime.
