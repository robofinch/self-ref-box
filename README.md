# Self-Bind

[![Latest version](https://img.shields.io/crates/v/self-bind.svg)](https://crates.io/crates/self-bind)
[![Documentation](https://img.shields.io/docsrs/self-bind)](https://docs.rs/self-bind/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Variance Family

[![Latest version](https://img.shields.io/crates/v/variance-family.svg)](https://crates.io/crates/variance-family)
[![Documentation](https://img.shields.io/docsrs/variance-family)](https://docs.rs/variance-family/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Aliasable View
[![Latest version](https://img.shields.io/crates/v/aliasable-view.svg)](https://crates.io/cratesaliasable-view)
[![Documentation](https://img.shields.io/docsrs/aliasable-view)](https://docs.rs/aliasable-view/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Overview

The goal is to make a "power-yoke"-like type; that is, `yoke::Yoke`, but maximally flexible
(and more sound by virtue of not treating `&mut T` and `Box<T>` as aliasable).
