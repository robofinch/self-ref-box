#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// The traits which are the central purpose of this crate
mod traits;
// An `UnvaryingFamily` type, greatly useful for trivial families not implemented below.
mod unvarying;
// `covariant`, `contravariant`, and `unvarying` macros that cover common cases.
mod macros;

// These are the most important implementations of `CovariantFamily` and `ContravariantFamily`, for:
// `&'a T`, `&'varying T` (as `VaryingRef<T>`), `*const T`, `&'a mut T`,
// `&'varying mut T` (as `VaryingMut<T>`),`*mut T`, `fn(..Args) -> R`
mod main_family_impls;
mod main_covariant_impls;
mod main_contravariant_impls;

// Implementations for:
// `[T]`, `[T; N]`, `(T1, ..., Tn)`, `bool`, `char`, floats, ints, uints, `str`,
// `array::IntoIter`, `cell::{Cell, Ref, RefCell, RefMut, UnsafeCell}`, `cmp::Ordering`,
// `convert::Infallible`, `ffi::CStr`, `fmt::Error`, `marker::{PhantomData, PhantomPinned}`,
// `mem::{ManuallyDrop, MaybeUninit}`, `num::NonZero*`, `option::Option`, `pin::Pin`,
// `ptr::NonNull`, `result::Result`, `slice::Iter`, `sync::atomic::*`, `time::Duration`
mod core_impls;

// Implementations for:
// `boxed::Box`, `borrow::Cow`, `alloc::collections::*::{collection_type, IntoIter, Iter}`,
// `ffi::CString`, `rc::{Rc, Weak}`, `string::String`, `sync::{Arc, Weak}`, `vec::{IntoIter, Vec}`
#[cfg(feature = "alloc")]
mod alloc_impls;

// Implementations for:
// `cell::{OnceCell, LazyCell}`, `collections::{hash_map, hash_set}::{collection, IntoIter, Iter}`,
// `ffi::{OsStr, OsString}`, `fs::File`,
// `io::{BufReader, BufWriter, Error}`, `path::{Path, PathBuf}`,
// `sync::{Condvar, Mutex, MutexGuard, OnceLock, RwLock, RwLock{Read, Write}Guard, LazyLock}`,
// `time::Instant`
#[cfg(feature = "std")]
mod std_impls;


pub use self::traits::{ContravariantFamily, CovariantFamily, LifetimeFamily};

/// Module for the `Cow<'varying, T>` family, called `VaryingCow<T>`.
pub mod borrow {}
/// Module for the `cell::Ref<'varying, T>` and `cell::RefMut<'varying, T>` families,
/// called `VaryingCellRef<T>` and `VaryingCellRefMut<T>`.
///
/// The prefix `Cell` is added to avoid a conflict with the names of the `&'varying T` and
/// `&'varying mut T` families.
pub mod cell {}
/// Module for multiple kinds of `Iter<'varying, ..>` families, called `VaryingIter<..>`.
pub mod collections {}
/// Module for the `slice::Iter<'varying, T>` family, called `VaryingIter<T>`.
pub mod slice {}
/// Module for the `MutexGuard<'varying, T>`, `RwLockReadGuard<'varying, T>`, and
/// `RwLockReadGuard<'varying, T>` families, called `Varying*Guard<T>`.
pub mod sync {}
