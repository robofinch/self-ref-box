#![expect(unsafe_code, reason = "work with raw pointers to avoid issues with provenance")]
#![warn(clippy::missing_inline_in_public_items, reason = "the functions here are all very short")]

// mod aliasable;
// mod variance;
// mod traits;
// mod impls;

use core::ptr;
use core::{marker::PhantomData, ptr::NonNull};

#[cfg(feature = "alloc")]
use core::str;
#[cfg(feature = "alloc")]
use core::mem::ManuallyDrop;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc, string::String, vec::Vec};

#[cfg(feature = "alloc")]
#[cfg(target_has_atomic = "ptr")]
use alloc::sync::Arc;


// ================================================================
//  Traits
// ================================================================

/// Require that both associated types of a [`DataSource`] outlive `'a`.
pub type Outlives<'a, D> = [&'a(<D as DataSource>::Auxiliary, <D as DataSource>::Data); 0];

/// Convert types (generally references or smart pointers) into a raw form more suitable for a
/// self-referential struct.
///
/// It is intended to be similar to [`stable_deref_trait::StableDeref`], but instead of relying on
/// calling `Deref` each time a reference is needed, the data is instead stored in a [`NonNull`]
/// (which may be safely moved without invalidating pointers or references derived from it).
///
/// Auxiliary data may be used for any information needed by the data source which is
/// unnecessary for accessing the actual data. For instance, [`Self::Auxiliary`] may be information
/// about an allocator or the capacity of a `Vec` (as only the data pointer and length of a `Vec`
/// are needed to access the data slice of a `Vec`). If unneeded, `Self::Auxiliary` can simply be
/// `()`.
///
/// Note that the semantics of this type aren't quite the same as [`Pin`], as any guarantee
/// of pinning or stability may be ended by calling [`Self::from_raw_data`] on all raw clones
/// of the data source.
///
/// # Safety
/// For any `(data, aux)` pairs returned by [`Self::into_raw_data`], moving the
/// `aux` value (of type [`Self::Auxiliary`]) must not invalidate any pointers or references
/// derived from a reference returned by calling [`Self::as_ref`] on `data`. (This is a
/// matter of pointer [provenance].)
///
/// [provenance]: core::ptr#provenance
/// [`Pin`]: core::pin::Pin
/// [`stable_deref_trait::StableDeref`]: https://docs.rs/stable_deref_trait/1/stable_deref_trait/trait.StableDeref.html
///
/// [`Self::Auxiliary`]: DataSource::Auxiliary
/// [`Self::into_raw_data`]: DataSource::into_raw_data
/// [`Self::from_raw_data`]: DataSource::from_raw_data
/// [`Self::as_ref`]: DataSource::as_ref
pub unsafe trait DataSource {
    /// The type of data which needs to be accessed in a stable way (while the raw form of the
    /// source of that data may be moved).
    type Data: ?Sized;
    /// Data relevant to the data source, but not relevant to [`DataSource::Data`].
    ///
    /// For instance, `Auxiliary` may be information about an allocator or the capacity of a `Vec`
    /// (as only the data pointer and length of a `Vec` are needed to access the slice of all its
    /// data). If unneeded, `Auxiliary` can simply be `()`.
    type Auxiliary;

    /// Convert the data source into a raw form.
    #[must_use]
    fn into_raw_data(self) -> (NonNull<Self::Data>, Self::Auxiliary);

    /// Get the data source back from its raw form.
    ///
    /// After this method is called, the raw data of the data source will no longer be accessed
    /// via `data`.
    ///
    /// # Safety
    /// `data` and `aux` must be a pair of raw and auxiliary data that was produced by
    /// [`Self::into_raw_data`] or [`Self::clone_raw`] and must not have been previously passed to
    /// [`Self::from_raw_data`] from then until now.
    ///
    /// [`Self::into_raw_data`]: DataSource::into_raw_data
    /// [`Self::from_raw_data`]: DataSource::from_raw_data
    /// [`Self::clone_raw`]: CloneableDataSource::clone_raw
    unsafe fn from_raw_data(data: NonNull<Self::Data>, aux: Self::Auxiliary) -> Self;

    /// Get immutable/shared access to the data, given shared access over its raw form.
    ///
    /// # Safety
    /// `data` must have been produced by [`Self::into_raw_data`] or [`Self::clone_raw`] and no
    /// have been previously passed to [`Self::from_raw_data`] from then until now. The associated
    /// auxiliary data must not have been dropped.
    ///
    /// For the duration of the borrow (when the returned reference exists) -- whether that be
    /// `'a` or an unsafely extended lifetime -- no method of [`DataSource`], [`MutableDataSource`],
    /// or [`CloneableDataSource`] may be called on the given clone of the raw data, except for
    /// [`DataSource::as_ref`]. That is, if multiple clones of the raw data were produced via
    /// [`CloneableDataSource`], then shared access is required of only one of the raw clones
    /// (namely, the one provided as `data`) for the duration of the borrow, and other raw clones
    /// may still be accessed in any way.
    ///
    /// The raw `data` pointer and associated auxiliary data may still be moved during
    /// the duration of the borrow, but the auxiliary data must not be dropped.
    ///
    /// The duration of the borrow (whether or not it is unsafely extended outside of this method)
    /// must not outlive either `Self::Data` or `Self::Auxiliary`. Except when unsafe lifetime
    /// extension is performed outside this method, this condition is enforced via the
    /// `_: Outlives<'a, Self>` parameter.
    ///
    /// [`Self::into_raw_data`]: DataSource::into_raw_data
    /// [`Self::from_raw_data`]: DataSource::from_raw_data
    /// [`Self::clone_raw`]: CloneableDataSource::clone_raw
    #[must_use]
    unsafe fn as_ref<'a: 'a>(data: NonNull<Self::Data>, _: Outlives<'a, Self>) -> &'a Self::Data;
}

/// Convert types (generally references or smart pointers) into a raw form more suitable for a
/// self-referential struct.
///
/// It is intended to be similar to [`stable_deref_trait::StableDeref`], but for `DerefMut`
/// instead of `Deref`. Additionally, instead of relying on calling `DerefMut` each time a
/// reference is needed, the data is instead stored in a movable [`NonNull`].
///
/// Note that it is extremely unlikely that both [`MutableDataSource`] and [`CloneableDataSource`]
/// could be soundly implemented for a nontrivial data source while preserving the intended
/// semantics.
///
/// # Safety
/// For any `(data, aux)` pairs returned by [`Self::into_raw_data`], moving the
/// `aux` value (of type [`Self::Auxiliary`]) must not invalidate any pointers or references
/// derived from a reference returned by calling [`Self::as_mut`] on `data`. (This is a
/// matter of pointer [provenance].)
///
/// [provenance]: core::ptr#provenance
/// [`stable_deref_trait::StableDeref`]: https://docs.rs/stable_deref_trait/1/stable_deref_trait/trait.StableDeref.html
///
/// [`Self::Auxiliary`]: DataSource::Auxiliary
/// [`Self::into_raw_data`]: DataSource::into_raw_data
/// [`Self::from_raw_data`]: DataSource::from_raw_data
/// [`Self::as_mut`]: MutableDataSource::as_mut
pub unsafe trait MutableDataSource: DataSource {
    /// Get mutable/exclusive access to the data, given exclusive access over its raw form.
    ///
    /// # Safety
    /// `data` must have been produced by [`Self::into_raw_data`] or [`Self::clone_raw`] and no
    /// have been previously passed to [`Self::from_raw_data`] from then until now. The associated
    /// auxiliary data must not have been dropped.
    ///
    /// For the duration of the borrow (when the returned reference exists) -- whether that be
    /// `'a` or an unsafely extended lifetime -- no method of [`DataSource`], [`MutableDataSource`],
    /// or [`CloneableDataSource`] may be called on the given clone of the raw data. That is, if
    /// multiple clones of the raw data were produced via [`CloneableDataSource`], then exclusive
    /// access is required of only one of the raw clones (namely, one equal to `data`) for the
    /// duration of the borrow, and other raw clones may still be accessed in any way.
    ///
    /// The raw `data` pointer and associated auxiliary data may still be moved during
    /// the duration of the borrow, but the auxiliary data must not be dropped.
    ///
    /// The duration of the borrow (whether or not it is unsafely extended outside of this method)
    /// must not outlive either `Self::Data` or `Self::Auxiliary`. Except when unsafe lifetime
    /// extension is performed outside this method, this condition is enforced via the
    /// `_: Outlives<'a, Self>` parameter.
    ///
    /// [`Self::into_raw_data`]: DataSource::into_raw_data
    /// [`Self::from_raw_data`]: DataSource::from_raw_data
    /// [`Self::clone_raw`]: CloneableDataSource::clone_raw
    #[must_use]
    unsafe fn as_mut<'a: 'a>(
        data: NonNull<Self::Data>,
        _:    Outlives<'a, Self>,
    ) -> &'a mut Self::Data;
}

/// Clone the raw form of data sources while maintaining stable access to the
/// [`DataSource::Data`] value.
///
/// This trait is intended to be somewhat similar to [`stable_deref_trait::CloneStableDeref`].
///
/// Note that it is extremely unlikely that both [`MutableDataSource`] and [`CloneableDataSource`]
/// could be soundly implemented for a nontrivial data source while preserving the intended
/// semantics.
///
/// # Safety
/// For any `(data, aux)` pairs returned by [`Self::into_raw_data`] or [`Self::clone_raw`], moving
/// the `aux` value (of type [`Self::Auxiliary`]) must not invalidate any pointers or references
/// derived from a reference returned by calling [`Self::as_ref`] or (if applicable)
/// [`Self::as_mut`] on `data`. (This is a matter of pointer [provenance].)
///
/// [provenance]: core::ptr#provenance
/// [`stable_deref_trait::CloneStableDeref`]: https://docs.rs/stable_deref_trait/1/stable_deref_trait/trait.CloneStableDeref.html
///
/// [`Self::Auxiliary`]: DataSource::Auxiliary
/// [`Self::into_raw_data`]: DataSource::into_raw_data
/// [`Self::from_raw_data`]: DataSource::from_raw_data
/// [`Self::as_ref`]: DataSource::as_ref
/// [`Self::as_mut`]: MutableDataSource::as_mut
/// [`Self::clone_raw`]: CloneableDataSource::clone_raw
pub unsafe trait CloneableDataSource: DataSource {
    /// Clone the raw form of the data source. The cloned data source should still refer to the
    /// same `Data`.
    ///
    /// This method may, for example, be implemented by incrementing a reference count.
    ///
    /// Note that while this method is called on `data`, no borrow from [`Self::as_ref`] or
    /// [`Self::as_mut`] is active on `data` (as enforced by other methods' safety
    /// conditions), though raw clones of `data` may still be accessed in any way while this
    /// function is called.
    ///
    /// # Safety
    /// `data` and `aux` must be a pair of raw and auxiliary data that was produced by
    /// [`Self::into_raw_data`] or [`Self::clone_raw`] and must not have been previously passed to
    /// [`Self::from_raw_data`] from then until now.
    ///
    /// [`Self::into_raw_data`]: DataSource::into_raw_data
    /// [`Self::from_raw_data`]: DataSource::from_raw_data
    /// [`Self::as_ref`]: DataSource::as_ref
    /// [`Self::as_mut`]: MutableDataSource::as_mut
    /// [`Self::clone_raw`]: CloneableDataSource::clone_raw
    #[must_use]
    unsafe fn clone_raw(
        data: NonNull<Self::Data>,
        aux: &Self::Auxiliary,
    ) -> (NonNull<Self::Data>, Self::Auxiliary);
}

// ================================================================
//  Trivial impl
// ================================================================

// SAFETY: Moving a `()` value cannot possibly invalidate a `&()` or `&mut ()` reference.
unsafe impl DataSource for () {
    type Data = ();
    type Auxiliary = ();

    #[inline]
    fn into_raw_data(self) -> (NonNull<Self::Data>, Self::Auxiliary) {
        (NonNull::dangling(), ())
    }

    #[inline]
    unsafe fn from_raw_data(_data: NonNull<Self::Data>, _aux: Self::Auxiliary) -> Self {}

    #[inline]
    unsafe fn as_ref<'a: 'a>(data: NonNull<Self::Data>, _: Outlives<'a, Self>) -> &'a Self::Data {
        // Just in case the address of the reference somehow matters, use `data`.
        let data: *const () = data.as_ptr().cast_const();

        // SAFETY: `data` is:
        // - trivially properly aligned for `()` with alignment 1
        // - non-null (it came from a `NonNull`)
        // - trivially dereferenceable for 0 bytes
        // - trivially points to a valid value of the ZST `()` as every raw pointer does
        // - trivially satisfies aliasing, as it refers to no memory
        //   that could be accessed or mutated
        unsafe { &*data }
    }
}

// SAFETY: Moving a `()` value cannot possibly invalidate a `&()` or `&mut ()` reference.
unsafe impl MutableDataSource for () {
    #[inline]
    unsafe fn as_mut<'a: 'a>(
        data: NonNull<Self::Data>,
        _:    Outlives<'a, Self>,
    ) -> &'a mut Self::Data {
        // Just in case the address of the reference somehow matters, use `data`.
        let data: *mut () = data.as_ptr();

        // SAFETY: `data` is:
        // - trivially properly aligned for `()` with alignment 1
        // - non-null (it came from a `NonNull`)
        // - trivially dereferenceable for 0 bytes
        // - trivially points to a valid value of the ZST `()` as every raw pointer does
        // - trivially satisfies aliasing, as it refers to no memory
        //   that could be accessed or mutated
        unsafe { &mut *data }
    }
}

// SAFETY: Moving a `()` value cannot possibly invalidate a `&()` or `&mut ()` reference.
unsafe impl CloneableDataSource for () {
    #[inline]
    unsafe fn clone_raw(
        data: NonNull<Self::Data>,
        _aux: &Self::Auxiliary,
    ) -> (NonNull<Self::Data>, Self::Auxiliary) {
        (data, ())
    }
}

// ================================================================
//  Mutable impls
// ================================================================

#[cfg(feature = "alloc")]
// SAFETY: Moving a `()` value cannot possibly invalidate a `&T` or `&mut T` reference.
unsafe impl<T: ?Sized> DataSource for Box<T> {
    type Data = T;
    type Auxiliary = ();

    #[inline]
    fn into_raw_data(self) -> (NonNull<Self::Data>, Self::Auxiliary) {
        let raw = Self::into_raw(self);
        // SAFETY: `Box::into_raw` guarantees that the returned pointer is non-null.
        let raw = unsafe { NonNull::new_unchecked(raw) };
        (raw, ())
    }

    #[inline]
    unsafe fn from_raw_data(data: NonNull<Self::Data>, _aux: Self::Auxiliary) -> Self {
        // SAFETY: the memory referred to by `data` was indeed allocated in accordance with the
        // memory layout used by `Box`, since `data` is guaranteed by the caller to have come
        // from `Self::into_raw_data`, meaning that `data` was returned by `Box::into_raw`.
        //
        // Additionally, we do not risk a double-free, as the caller promises that `data`
        // was not previously passed to `Self::from_raw_data` after having been produced by
        // `Self::into_raw_data`.
        //
        // Note also that this safety comment (correctly) assumes that `Box<T>` never implements
        // `CloneableDataSource`, meaning that the only possibly source of `data` (as guaranteed by
        // the caller) is `Self::into_raw_data`.
        unsafe {
            Self::from_raw(data.as_ptr())
        }
    }

    #[inline]
    unsafe fn as_ref<'a: 'a>(data: NonNull<Self::Data>, _: Outlives<'a, Self>) -> &'a Self::Data {
        // SAFETY: `data` is
        // - properly aligned for `Self::Data` (which is `T`), as the pointer came from
        //   `Box::<T>::into_raw`, which promises that the returned pointer is properly-aligned,
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for `size_of::<T>()` bytes, as a pointer with valid read
        //   provenance to the contents of a `Box<T>` must be dereferenceable for `size_of::<T>()`
        //   bytes, and
        // - points to a valid value of type `T`, as the contents of `Box<T>` are initially
        //   a valid value of type `T`, and exposing (possibly mutable) references to a valid value
        //   of type `T` should not result in an invalid value (for type `T`) being observably
        //   written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for shared references, since the
        //   caller promises that, while the returned reference exists, no data source method
        //   is called on `data` other than `DataSource::as_ref`. This means that only shared
        //   access to the pointee of `data` is exposed by data source methods while the returned
        //   reference exists (and since the raw data source has ownership over the `Box`, no other
        //   means of accessing the pointee are sound), and thus that the pointee is not mutated
        //   while the returned reference exists (except in `UnsafeCell`).
        //
        // Note also that this safety comment (correctly) assumes that `Box<T>` never implements
        // `CloneableDataSource`, meaning that the only possibly source of `data` (as guaranteed by
        // the caller) is `Self::into_raw_data`.
        unsafe { &*data.as_ptr().cast_const() }
    }
}

#[cfg(feature = "alloc")]
// SAFETY: Moving a `()` value cannot possibly invalidate a `&T` or `&mut T` reference.
unsafe impl<T: ?Sized> MutableDataSource for Box<T> {
    #[inline]
    unsafe fn as_mut<'a: 'a>(
        data: NonNull<Self::Data>,
        _:    Outlives<'a, Self>,
    ) -> &'a mut Self::Data {
        // SAFETY: `data` is
        // - properly aligned for `Self::Data` (which is `T`), as the pointer came from
        //   `Box::<T>::into_raw`, which promises that the returned pointer is properly-aligned,
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for `size_of::<T>()` bytes, as a pointer with valid read/write
        //   provenance to the contents of a `Box<T>` must be dereferenceable for `size_of::<T>()`
        //   bytes, and
        // - points to a valid value of type `T`, as the contents of `Box<T>` are initially
        //   a valid value of type `T`, and exposing (possibly mutable) references to a valid value
        //   of type `T` should not result in an invalid value (for type `T`) being observably
        //   written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for exclusive references, since
        //   the caller promises that, while the returned reference exists, no data source method
        //   is called on `data`. (Since the raw data source has ownership over the `Box`, no other
        //   means of accessing the pointee are sound.) This means that the returned mutable
        //   reference to the pointee of `data` (and pointers or references derived from it) is the
        //   only way to (soundly) access the pointee through data source methods while the
        //   returned reference exists.
        //
        // Note also that this safety comment (correctly) assumes that `Box<T>` never implements
        // `CloneableDataSource`, meaning that the only possibly source of `data` (as guaranteed by
        // the caller) is `Self::into_raw_data`.
        unsafe { &mut *data.as_ptr() }
    }
}

#[cfg(feature = "alloc")]
// SAFETY: Moving a `usize` value cannot possibly invalidate a `&[T]` or `&mut [T]` reference.
unsafe impl<T> DataSource for Vec<T> {
    type Data = [T];
    type Auxiliary = usize;

    #[inline]
    fn into_raw_data(self) -> (NonNull<Self::Data>, Self::Auxiliary) {
        let mut source = ManuallyDrop::new(self);
        let capacity = source.capacity();
        let len = source.len();

        // Notice that `.as_mut_ptr()` is the last method called on `source`, in order to ensure
        // that we don't call any method that would invalidate the provenance of the pointer
        // returned by `.as_mut_ptr()`.
        let raw = ptr::slice_from_raw_parts_mut(source.as_mut_ptr(), len);
        // SAFETY: `raw` came from the data pointer of a `Vec`, which is guaranteed to be
        // non-null. (The docs of `Vec::as_mut_ptr` honestly aren't *that* explicit about it,
        // but the main docs of `Vec` are unambiguous.)
        let raw = unsafe { NonNull::new_unchecked(raw) };

        (raw, capacity)
    }

    #[inline]
    unsafe fn from_raw_data(data: NonNull<Self::Data>, aux: Self::Auxiliary) -> Self {
        let slice_ptr: *mut [T] = data.as_ptr();
        let data_ptr: *mut T = slice_ptr.cast();
        let data_len = slice_ptr.len();

        // SAFETY: the caller guarantees that `(data, aux)` came from `Self::into_raw_data`,
        // so `data` came from `ptr::slice_from_raw_parts_mut` applied to the returned values
        // of `Vec::as_mut_ptr` and `Vec::len` (respectively), while `aux` came from the returned
        // value of `Vec::capacity`. Note also that the source `Vec` was not dropped, since it
        // was wrapped in `ManuallyDrop` (and not manually dropped). Moreover, we don't expose
        // any way to (safely) write values invalid for `T` into the slice, nor do we say anything
        // that would imply to authors of `unsafe` that writing such invalid values would be
        // sound.
        //
        // Thus, this call puts back together raw parts that came from a `Vec<T>` and is therefore
        // sound. Going through the checklist in detail:
        //
        // - the allocation, non-null, and alignment requirements are met for `data_ptr`;
        // - `T` has the same alignment as what `data_ptr` was allocated with (if applicable);
        // - the size of `T` times the capacity, if nonzero, was the same size as what `data_ptr`
        //   was allocated with;
        // - `data_len <= aux`;
        // - (finally, a less trivial one:) the first `data_len` values are properly initialized
        //   for type `T`;
        // - `capacity` is the capacity that the pointer was allocated with (if applicable)
        //   (uhhh this condition seems wrong, and overlaps with a previous bullet point, but
        //   either way this condition holds here);
        //   (there's an issue opened on the rust-lang/rust repo about this condition, so my
        //   instinct isn't wrong)
        // - The allocated size in bytes does not exceed `isize::MAX`.
        //
        // Note also that this safety comment (correctly) assumes that `Vec<T>` never implements
        // `CloneableDataSource`, meaning that the only possibly source of `data` (as guaranteed by
        // the caller) is `Self::into_raw_data`.
        unsafe { Self::from_raw_parts(data_ptr, data_len, aux) }
    }

    #[inline]
    unsafe fn as_ref<'a: 'a>(data: NonNull<Self::Data>, _: Outlives<'a, Self>) -> &'a Self::Data {
        // SAFETY: `data` is
        // - properly aligned for `Self::Data` (which is `[T]`), as the pointer came from
        //   `Vec::<T>::as_mut_ptr`, which promises that the returned pointer is valid for reads
        //   of at least zero bytes (which requires that it be properly-aligned),
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for the length of the whole `[T]` slice, as a pointer with valid
        //   read provenance for the contents of a `Vec<T>` must be dereferenceable (and the
        //   pointer is a `Vec<T>`'s data pointer, and the length metadata of `data` is the length
        //   of that `Vec`),
        // - points to a valid value of type `[T]`, as the pointee was initially a valid value
        //   of type `[T]` when it was created from the parts of a `Vec<T>`, and exposing
        //   (possibly mutable) references to a valid value of type `[T]` should not result in an
        //   invalid value (for type `[T]`) being observably written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for shared references, since the
        //   caller promises that, while the returned reference exists, no data source method
        //   is called on `data` other than `DataSource::as_ref`. (Since the raw data source has
        //   ownership over the `Vec`, no other means of accessing the pointee are sound.) This
        //   means that only shared access to the pointee of `data` is exposed by data source
        //   methods while the returned reference exists, and thus that the pointee is not mutated
        //   while the returned reference exists (except in `UnsafeCell`).
        //
        // Note also that this safety comment (correctly) assumes that `Vec<T>` never implements
        // `CloneableDataSource`, meaning that the only possibly source of `data` (as guaranteed by
        // the caller) is `Self::into_raw_data`.
        unsafe { &*data.as_ptr().cast_const() }
    }
}

#[cfg(feature = "alloc")]
// SAFETY: Moving a `usize` value cannot possibly invalidate a `&[T]` or `&mut [T]` reference.
unsafe impl<T> MutableDataSource for Vec<T> {
    #[inline]
    unsafe fn as_mut<'a: 'a>(
        data: NonNull<Self::Data>,
        _:    Outlives<'a, Self>,
    ) -> &'a mut Self::Data {
        // SAFETY: `data` is
        // - properly aligned for `Self::Data` (which is `[T]`), as the pointer came from
        //   `Vec::<T>::as_mut_ptr`, which promises that the returned pointer is valid for reads
        //   of at least zero bytes (which requires that it be properly-aligned),
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for the length of the whole `[T]` slice, as a pointer with valid
        //   read/write provenance for the contents of a `Vec<T>` must be dereferenceable (and the
        //   pointer is a `Vec<T>`'s data pointer, and the length metadata of `data` is the length
        //   of that `Vec`),
        // - points to a valid value of type `[T]`, as the pointee was initially a valid value
        //   of type `[T]` when it was created from the parts of a `Vec<T>`, and exposing
        //   (possibly mutable) references to a valid value of type `[T]` should not result in an
        //   invalid value (for type `[T]`) being observably written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for exclusive references, since
        //   the caller promises that, while the returned reference exists, no data source method
        //   is called on `data`. (Since the raw data source has ownership over the `Vec`, no other
        //   means of accessing the pointee are sound.) This means that the returned mutable
        //   reference to the pointee of `data` (and pointers or references derived from it) is the
        //   only way to (soundly) access the pointee through data source methods while the
        //   returned reference exists.
        //
        // Note also that this safety comment (correctly) assumes that `Vec<T>` never implements
        // `CloneableDataSource`, meaning that the only possibly source of `data` (as guaranteed by
        // the caller) is `Self::into_raw_data`.
        unsafe { &mut *data.as_ptr() }
    }
}

#[cfg(feature = "alloc")]
// SAFETY: Moving a `usize` value cannot possibly invalidate a `&str` or `&mut str` reference.
unsafe impl DataSource for String {
    type Data = str;
    type Auxiliary = usize;

    #[inline]
    fn into_raw_data(self) -> (NonNull<Self::Data>, Self::Auxiliary) {
        let mut source = ManuallyDrop::new(self);
        let capacity = source.capacity();
        // Notice that `.as_mut_str()` is the last method called on `source`, in order to ensure
        // that we don't call any method that would invalidate the provenance of the reference
        // returned by `.as_mut_str()` (which is immediately coerced into a pointer).
        let raw: *mut str = source.as_mut_str();
        // SAFETY: `raw` came from a `&mut str`, so it is guaranteed to be non-null.
        let raw = unsafe { NonNull::new_unchecked(raw) };

        (raw, capacity)
    }

    #[inline]
    unsafe fn from_raw_data(data: NonNull<Self::Data>, aux: Self::Auxiliary) -> Self {
        let data: *mut str = data.as_ptr();
        #[expect(clippy::as_conversions, reason = "pointer::cast does not work on unsized types")]
        let raw: *mut [u8] = data as *mut [u8];
        let data_ptr: *mut u8 = raw.cast();
        let data_len = raw.len();

        // SAFETY: We must meet all guarantees of `Vec::from_raw_parts`, as well as guarantee that
        // the data is valid UTF-8. The caller guarantees that `(data, aux)` came from
        // `Self::into_raw_data`, so `data` came from `String::as_mut_str`, while `aux` came from
        // the returned value of `String::capacity`. `data_ptr` points to the start of the `String`
        // allocation and `data_len` is the length of the `String`, and therefore `data_ptr`,
        // `data_len`, and `aux` are the data pointer, length, and capacity of the `Vec<u8>` used
        // by the `String`. Note also that the source `String` was not dropped, since it was wrapped
        // in `ManuallyDrop` (and not manually dropped). Moreover, we don't expose any way to
        // (safely) write values invalid for `str` into the string slice, nor do we say anything
        // that would imply to authors of `unsafe` that writing such invalid values would be sound.
        //
        // Thus, this call puts back together raw parts that came from a `String` and is therefore
        // sound. Going through the checklist in detail:
        //
        // - the allocation, non-null, and alignment requirements are met for `data_ptr`;
        // - `u8` has the same alignment as what `data_ptr` was allocated with (if applicable);
        // - the size of `u8` times the capacity, if nonzero, was the same size as what `data_ptr`
        //   was allocated with;
        // - `data_len <= aux`;
        // - (finally, a less trivial one:) the first `data_len` values are properly initialized
        //   for type `u8` (since the first `data_len` bytes must form a properly-initialized
        //   `str`, whose requirements are a strict superset of those of `[u8]`);
        // - `capacity` is the capacity that the pointer was allocated with (if applicable);
        // - The allocated size in bytes does not exceed `isize::MAX`.
        //
        // - the passed bytes are valid UTF-8.
        //
        // Note also that this safety comment (correctly) assumes that `String` never implements
        // `CloneableDataSource`, meaning that the only possibly source of `data` (as guaranteed by
        // the caller) is `Self::into_raw_data`.
        unsafe { Self::from_raw_parts(data_ptr, data_len, aux) }
    }

    #[inline]
    unsafe fn as_ref<'a: 'a>(data: NonNull<Self::Data>, _: Outlives<'a, Self>) -> &'a Self::Data {
        // SAFETY: `data` is
        // - trivially properly aligned, since `str` has alignment 1 (same as `[u8`),
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for the whole `str` slice, as a pointer with valid read
        //   provenance for the contents of a `String` must be dereferenceable (and `data` came from
        //   `String::as_mut_str`),
        // - points to a valid value of type `str`, as the pointee was initially a valid value
        //   of type `str` when the `data` pointer was obtained, and exposing (possibly mutable)
        //   references to a valid value of type `str` should not result in an invalid value (for
        //   type `str`) being observably written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for shared references, since the
        //   caller promises that, while the returned reference exists, no data source method
        //   is called on `data` other than `DataSource::as_ref`. (Since the raw data source has
        //   ownership over the `String`, no other means of accessing the pointee are sound.)
        //   This means that only shared access to the pointee of `data` is exposed by data source
        //   methods while the returned reference exists, and thus that the pointee is not mutated
        //   while the returned reference exists (except in `UnsafeCell`).
        //
        // Note also that this safety comment (correctly) assumes that `String` never implements
        // `CloneableDataSource`, meaning that the only possibly source of `data` (as guaranteed by
        // the caller) is `Self::into_raw_data`.
        unsafe { &*data.as_ptr().cast_const() }
    }
}

#[cfg(feature = "alloc")]
// SAFETY: Moving a `usize` value cannot possibly invalidate a `&str` or `&mut str` reference.
unsafe impl MutableDataSource for String {
    #[inline]
    unsafe fn as_mut<'a: 'a>(
        data: NonNull<Self::Data>,
        _:    Outlives<'a, Self>,
    ) -> &'a mut Self::Data {
        // SAFETY: `data` is
        // - trivially properly aligned, since `str` has alignment 1 (same as `[u8`),
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for the whole `str` slice, as a pointer with valid read/write
        //   provenance for the contents of a `String` must be dereferenceable (and `data` came from
        //   `String::as_mut_str`),
        // - points to a valid value of type `str`, as the pointee was initially a valid value
        //   of type `str` when the `data` pointer was obtained, and exposing (possibly mutable)
        //   references to a valid value of type `str` should not result in an invalid value (for
        //   type `str`) being observably written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for exclusive references, since
        //   the caller promises that, while the returned reference exists, no data source method
        //   is called on `data`. (Since the raw data source has ownership over the `String`, no
        //   other means of accessing the pointee are sound.) This means that the returned mutable
        //   reference to the pointee of `data` (and pointers or references derived from it) is the
        //   only way to (soundly) access the pointee through data source methods while the
        //   returned reference exists.
        //
        // Note also that this safety comment (correctly) assumes that `String` never implements
        // `CloneableDataSource`, meaning that the only possibly source of `data` (as guaranteed by
        // the caller) is `Self::into_raw_data`.
        unsafe { &mut *data.as_ptr() }
    }
}

// SAFETY: Moving a `PhantomData` value cannot possibly invalidate a `&T` or `&mut T` reference.
unsafe impl<'d, T: ?Sized> DataSource for &'d mut T {
    type Data = T;
    // Ensure that the `'d` lifetime is included in the bounds for `as_ref` and `as_mut`, and make
    // sure the borrow checker recognizes the pointee of the reference given to `into_raw_data` as
    // borrowed, even when it's in a raw form.
    type Auxiliary = PhantomData<fn() -> &'d mut T>;

    #[inline]
    fn into_raw_data(self) -> (NonNull<Self::Data>, Self::Auxiliary) {
        let raw = ptr::from_mut(self);
        // SAFETY: references are non-null, so `raw` is non-null.
        let raw = unsafe { NonNull::new_unchecked(raw) };
        (raw, PhantomData)
    }

    #[inline]
    unsafe fn from_raw_data(data: NonNull<Self::Data>, _aux: Self::Auxiliary) -> Self {
        // SAFETY: the caller promises that `data` came from `Self::into_raw_data` or
        // `Self::clone_raw`, and since `Self` doesn't implement `CloneableDataSource`, it follows
        // that the `data` pointer comes from a `&mut T` (and its address was not mutated, and
        // its provenance shouldn't have been invalidated, as the `PhantomData` ensures that the
        // borrow checker knew that the pointee of the source reference was exclusively borrowed
        // from the `into_raw_data` call up to now, and the aliasing rules for data source methods
        // should also ensure that its provenance was not invalidated). Therefore, `data` is:
        // - properly aligned for `Self::Data` (which is `T`), as the pointer came from a `&mut T`,
        //   which must be properly-aligned,
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for `size_of::<T>()` bytes, as `data` has provenance valid for
        //   reading and writing the pointee of the source `&mut T`, and
        // - points to a valid value of type `T`, as the pointee of the `&mut T` was initially
        //   a valid value of type `T`, and exposing (possibly mutable) references to a valid value
        //   of type `T` should not result in an invalid value (for type `T`) being observably
        //   written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for exclusive references, since
        //   until the data source has `Self::from_raw_data` called on it, we only ever expose
        //   references of lifetime at most `'d` to the pointee, and `from_raw_data` is not allowed
        //   to be called on the raw data source while one of the references from `as_ref` or
        //   `as_mut` still exists (due to those functions' safety conditions). The initially-given
        //   exclusive reference had lifetime `'d`, and here we create an exclusive reference of
        //   type `'d`. The given exclusive reference is considered borrowed while the raw data
        //   source version of it exists, thanks to the `PhantomData` in `Self::Auxiliary`, and as
        //   the `'d` lifetime in that `PhantomData` is now passed here, for at least the duration
        //   of `'d`, the pointee should not be accessed except through pointers or references
        //   derived from the returned `&mut T`.
        unsafe { &mut *data.as_ptr() }
    }

    #[inline]
    unsafe fn as_ref<'a: 'a>(data: NonNull<Self::Data>, _: Outlives<'a, Self>) -> &'a Self::Data {
        // SAFETY: the caller promises that `data` came from `Self::into_raw_data` or
        // `Self::clone_raw`, and since `Self` doesn't implement `CloneableDataSource`, it follows
        // that the `data` pointer comes from a `&mut T` (and its address was not mutated, and
        // its provenance shouldn't have been invalidated, as the `PhantomData` ensures that the
        // borrow checker knew that the pointee of the source reference was exclusively borrowed
        // from the `into_raw_data` call up to now, and the aliasing rules for data source methods
        // should also ensure that its provenance was not invalidated). Therefore, `data` is:
        // - properly aligned for `Self::Data` (which is `T`), as the pointer came from a `&mut T`,
        //   which must be properly-aligned,
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for `size_of::<T>()` bytes, as `data` has provenance valid for
        //   reading the pointee of the source `&mut T`, and
        // - points to a valid value of type `T`, as the pointee of the `&mut T` was initially
        //   a valid value of type `T`, and exposing (possibly mutable) references to a valid value
        //   of type `T` should not result in an invalid value (for type `T`) being observably
        //   written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for shared references, since the
        //   caller promises that, while the returned reference exists, no data source method
        //   is called on `data` other than `DataSource::as_ref`. Note also that the
        //   initially-given exclusive reference had lifetime `'d`, and the returned reference is
        //   only allowed to exist for at most `'d`. The `PhantomData` in `Self::Auxiliary` (along
        //   with the requirement to not drop that auxiliary data while the returned reference
        //   exists) ensures that the the pointee of the initially-given reference is considered to
        //   be under an exclusive borrow *at least* while the returned reference exists. This
        //   means that no (sound) access to the pointee of `data` is exposed through external
        //   means( as the pointee is known by the borrow checker to be under an exclusive borrow),
        //   leaving only the shared access to the pointee of `data` exposed by data source methods.
        //   Therefore, while the returned reference exists, the pointee is not mutated
        //   (except in `UnsafeCell`).
        unsafe { &*data.as_ptr().cast_const() }
    }
}

// SAFETY: Moving a `PhantomData` value cannot possibly invalidate a `&T` or `&mut T` reference.
unsafe impl<T: ?Sized> MutableDataSource for &mut T {
    #[inline]
    unsafe fn as_mut<'a: 'a>(
        data: NonNull<Self::Data>,
        _:    Outlives<'a, Self>,
    ) -> &'a mut Self::Data {
        // SAFETY: the caller promises that `data` came from `Self::into_raw_data` or
        // `Self::clone_raw`, and since `Self` doesn't implement `CloneableDataSource`, it follows
        // that the `data` pointer comes from a `&mut T` (and its address was not mutated, and
        // its provenance shouldn't have been invalidated, as the `PhantomData` ensures that the
        // borrow checker knew that the pointee of the source reference was exclusively borrowed
        // from the `into_raw_data` call up to now, and the aliasing rules for data source methods
        // should also ensure that its provenance was not invalidated). Therefore, `data` is:
        // - properly aligned for `Self::Data` (which is `T`), as the pointer came from a `&mut T`,
        //   which must be properly-aligned,
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for `size_of::<T>()` bytes, as `data` has provenance valid for
        //   reading and writing the pointee of the source `&mut T`, and
        // - points to a valid value of type `T`, as the pointee of the `&mut T` was initially
        //   a valid value of type `T`, and exposing (possibly mutable) references to a valid value
        //   of type `T` should not result in an invalid value (for type `T`) being observably
        //   written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for exclusive references, since
        //   the caller promises that, while the returned reference exists, no data source method
        //   is called on `data`. Note also that the initially-given exclusive reference had
        //   lifetime `'d`, and the returned reference is only allowed to exist for at most `'d`.
        //   The `PhantomData` in `Self::Auxiliary` (along with the requirement to not drop that
        //   auxiliary data while the returned reference exists) ensures that the the pointee of
        //   the initially-given reference is considered to be under an exclusive borrow *at least*
        //   while the returned reference exists. This means that no (sound) access to the pointee
        //   of `data` is exposed through external means (as the pointee is known by the borrow
        //   checker to be under an exclusive borrow) or by data source methods.
        //   Therefore, while the returned reference exists, the pointee is only accessed through
        //   pointers and references derived from the returned reference.
        unsafe { &mut *data.as_ptr() }
    }
}

// ================================================================
//  Cloneable impls
// ================================================================

#[cfg(feature = "alloc")]
// SAFETY: Moving a `()` value cannot possibly invalidate a `&T` reference.
unsafe impl<T: ?Sized> DataSource for Rc<T> {
    type Data = T;
    type Auxiliary = ();

    #[inline]
    fn into_raw_data(self) -> (NonNull<Self::Data>, Self::Auxiliary) {
        let raw = Self::into_raw(self).cast_mut();
        // SAFETY: the pointer associated with an `Rc` is never null. ("`Rc` always allocates"
        // and "`Rc::into_raw` is marked with `#[rustc_never_returns_null_ptr]`" seem like
        // thorough justifications.)
        let raw = unsafe { NonNull::new_unchecked(raw) };
        (raw, ())
    }

    #[inline]
    unsafe fn from_raw_data(data: NonNull<Self::Data>, _aux: Self::Auxiliary) -> Self {
        // SAFETY: the caller promises that `data` came from `Self::into_raw_data` or
        // `Self::clone_raw`. Since `Self::clone_raw` returns its `data` argument (which
        // must have come from `into_raw_data` or `clone_raw`), it follows that the `data` pointer
        // ultimately comes from `Rc::<T>::into_raw` in `Self::into_raw_data`.
        // We also must ensure that the value of `T` within the `Rc` is only dropped once. The
        // safety condition on data source functions does not allow a given raw clone of a data
        // source to have `Self::from_raw_data` called on it more than once; the only way to get
        // more raw clones of a data source is with `Self::clone_raw`, which increments the
        // strong reference count of the `Rc`. Therefore, the total number of times that
        // `from_raw_data` may be called (ultimately leading to decrements of the `Rc` reference
        // count) is equal to the number of times `Self::clone_raw` was called plus the one time
        // `Self::into_raw_data` was called (...considering other calls of `into_raw_data` on `Rc`
        // clones to result in "different" raw data sources...) so we do not use more reference
        // counts than we are supposed to.
        unsafe {
            Self::from_raw(data.as_ptr().cast_const())
        }
    }

    #[inline]
    unsafe fn as_ref<'a: 'a>(data: NonNull<Self::Data>, _: Outlives<'a, Self>) -> &'a Self::Data {
        // SAFETY: `data` is
        // - properly aligned for `Self::Data` (which is `T`), as the pointer came from
        //   `Rc::<T>::into_raw`, and the pointer to `T` used by an `Rc` must be properly-aligned,
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for `size_of::<T>()` bytes, as a pointer with valid read
        //   provenance to the contents of a `Rc<T>` must be dereferenceable
        //   for `size_of::<T>()` bytes, and
        // - points to a valid value of type `T`, as the contents of `Rc<T>` are initially
        //   a valid value of type `T`, and exposing references to a valid value
        //   of type `T` should not result in an invalid value (for type `T`) being observably
        //   written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for shared references, since the
        //   caller promises that, while the returned reference exists, no data source method
        //   is called on `data` other than `DataSource::as_ref`. This means that only shared
        //   access to the pointee of `data` is exposed by data source methods while the returned
        //   reference exists, and since the raw data source has ownership over a strong count of
        //   the `Rc`, external means cannot soundly mutate the pointee of `data` while the returned
        //   reference exists (except in `UnsafeCell`).
        unsafe { &*data.as_ptr().cast_const() }
    }
}

#[cfg(feature = "alloc")]
// SAFETY: Moving a `()` value cannot possibly invalidate a `&T` reference.
unsafe impl<T: ?Sized> CloneableDataSource for Rc<T> {
    #[inline]
    unsafe fn clone_raw(
        data: NonNull<Self::Data>,
        _aux: &Self::Auxiliary,
    ) -> (NonNull<Self::Data>, Self::Auxiliary) {
        // SAFETY: the caller promises that `data` came from `Self::into_raw_data` or
        // `Self::clone_raw`. Since `Self::clone_raw` returns its `data` argument (which
        // must have come from `into_raw_data` or `clone_raw`), it follows that the `data` pointer
        // ultimately comes from `Rc::<T>::into_raw` in `Self::into_raw_data`.
        // The caller promises that the given `(data, aux)` pair had not already been passed
        // to `Self::from_raw_data` (which could decrement the `Rc` reference count). The strong
        // count of the `Rc` must therefore be at least 1 during this function (see
        // `Self::from_raw_data`; calls to that method are not allowed to excessively decrement
        // the reference count, and we may assume that the `Rc` passed to `into_raw_data` was
        // valid and had a strong reference count of at least 1).
        unsafe {
            Self::increment_strong_count(data.as_ptr().cast_const());
        };
        (data, ())
    }
}

#[cfg(feature = "alloc")]
#[cfg(target_has_atomic = "ptr")]
// SAFETY: Moving a `()` value cannot possibly invalidate a `&T` reference.
unsafe impl<T: ?Sized> DataSource for Arc<T> {
    type Data = T;
    type Auxiliary = ();

    #[inline]
    fn into_raw_data(self) -> (NonNull<Self::Data>, Self::Auxiliary) {
        let raw = Self::into_raw(self).cast_mut();
        // SAFETY: the pointer associated with an `Arc` is never null. ("`Arc` always allocates"
        // and "`Arc::into_raw` is marked with `#[rustc_never_returns_null_ptr]`" seem like
        // thorough justifications.)
        let raw = unsafe { NonNull::new_unchecked(raw) };
        (raw, ())
    }

    #[inline]
    unsafe fn from_raw_data(data: NonNull<Self::Data>, _aux: Self::Auxiliary) -> Self {
        // SAFETY: the caller promises that `data` came from `Self::into_raw_data` or
        // `Self::clone_raw`. Since `Self::clone_raw` returns its `data` argument (which
        // must have come from `into_raw_data` or `clone_raw`), it follows that the `data` pointer
        // ultimately comes from `Arc::<T>::into_raw` in `Self::into_raw_data`.
        // We also must ensure that the value of `T` within the `Arc` is only dropped once. The
        // safety condition on data source functions does not allow a given raw clone of a data
        // source to have `Self::from_raw_data` called on it more than once; the only way to get
        // more raw clones of a data source is with `Self::clone_raw`, which increments the
        // strong reference count of the `Arc`. Therefore, the total number of times that
        // `from_raw_data` may be called (ultimately leading to decrements of the `Arc` reference
        // count) is equal to the number of times `Self::clone_raw` was called plus the one time
        // `Self::into_raw_data` was called (...considering other calls of `into_raw_data` on `Arc`
        // clones to result in "different" raw data sources...) so we do not use more reference
        // counts than we are supposed to.
        unsafe {
            Self::from_raw(data.as_ptr().cast_const())
        }
    }

    #[inline]
    unsafe fn as_ref<'a: 'a>(data: NonNull<Self::Data>, _: Outlives<'a, Self>) -> &'a Self::Data {
        // SAFETY: `data` is
        // - properly aligned for `Self::Data` (which is `T`), as the pointer came from
        //   `Arc::<T>::into_raw`, and the pointer to `T` used by an `Arc` must be properly-aligned,
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for `size_of::<T>()` bytes, as a pointer with valid read
        //   provenance to the contents of a `Arc<T>` must be dereferenceable
        //   for `size_of::<T>()` bytes, and
        // - points to a valid value of type `T`, as the contents of `Arc<T>` are initially
        //   a valid value of type `T`, and exposing references to a valid value
        //   of type `T` should not result in an invalid value (for type `T`) being observably
        //   written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for shared references, since the
        //   caller promises that, while the returned reference exists, no data source method
        //   is called on `data` other than `DataSource::as_ref`. This means that only shared
        //   access to the pointee of `data` is exposed by data source methods while the returned
        //   reference exists, and since the raw data source has ownership over a strong count of
        //   the `Arc`, external means cannot soundly mutate the pointee of `data` while the
        //   returned reference exists (except in `UnsafeCell`).
        unsafe { &*data.as_ptr().cast_const() }
    }
}

#[cfg(feature = "alloc")]
#[cfg(target_has_atomic = "ptr")]
// SAFETY: Moving a `()` value cannot possibly invalidate a `&T` reference.
unsafe impl<T: ?Sized> CloneableDataSource for Arc<T> {
    #[inline]
    unsafe fn clone_raw(
        data: NonNull<Self::Data>,
        _aux: &Self::Auxiliary,
    ) -> (NonNull<Self::Data>, Self::Auxiliary) {
        // SAFETY: the caller promises that `data` came from `Self::into_raw_data` or
        // `Self::clone_raw`. Since `Self::clone_raw` returns its `data` argument (which
        // must have come from `into_raw_data` or `clone_raw`), it follows that the `data` pointer
        // ultimately comes from `Arc::<T>::into_raw` in `Self::into_raw_data`.
        // The caller promises that the given `(data, aux)` pair had not already been passed
        // to `Self::from_raw_data` (which could decrement the `Arc` reference count). The strong
        // count of the `Arc` must therefore be at least 1 during this function (see
        // `Self::from_raw_data`; calls to that method are not allowed to excessively decrement
        // the reference count, and we may assume that the `Arc` passed to `into_raw_data` was
        // valid and had a strong reference count of at least 1).
        unsafe {
            Self::increment_strong_count(data.as_ptr().cast_const());
        };
        (data, ())
    }
}

// SAFETY: Moving a `PhantomData` value cannot possibly invalidate a `&T` reference.
unsafe impl<'d, T: ?Sized> DataSource for &'d T {
    type Data = T;
    // Ensure that the `'d` lifetime is included in the bounds for `as_ref`, and make
    // sure the borrow checker recognizes the pointee of the reference given to `into_raw_data` as
    // borrowed, even when it's in a raw form.
    type Auxiliary = PhantomData<&'d ()>;

    #[inline]
    fn into_raw_data(self) -> (NonNull<Self::Data>, Self::Auxiliary) {
        let raw = ptr::from_ref(self).cast_mut();
        // SAFETY: references are non-null, so `raw` is non-null.
        let raw = unsafe { NonNull::new_unchecked(raw) };
        (raw, PhantomData)
    }

    #[inline]
    unsafe fn from_raw_data(data: NonNull<Self::Data>, _aux: Self::Auxiliary) -> Self {
        // SAFETY: the caller promises that `data` came from `Self::into_raw_data` or
        // `Self::clone_raw`. Since `Self::clone_raw` just returns its `data` argument (which
        // must have come from `into_raw_data` or `clone_raw`), it follows that the `data` pointer
        // derives from a `&T` (and its address was not mutated, and and its provenance shouldn't
        // have been invalidated, as the `PhantomData` ensures that the borrow checker knew that
        // the pointee of the source reference was under a shared borrow from the `into_raw_data`
        // call up to now, and the aliasing rules for data source methods should also ensure that
        // its provenance was not invalidated). Therefore, `data` is:
        // - properly aligned for `Self::Data` (which is `T`), as the pointer came from a `&T`,
        //   which must be properly-aligned,
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for `size_of::<T>()` bytes, as `data` has provenance valid for
        //   reading the pointee of the source `&T`, and
        // - points to a valid value of type `T`, as the pointee of the `&T` was initially
        //   a valid value of type `T`, and exposing shared references to a valid value
        //   of type `T` should not result in an invalid value (for type `T`) being observably
        //   written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for shared references, since
        //   until every raw clone of the data source has `Self::from_raw_data` called on it,
        //   we only ever expose shared references (of lifetime at most `'d`) to the pointee
        //   The initially-given shared reference had lifetime `'d`, and here we create a shared
        //   reference of type `'d`. The given shared reference is considered borrowed while
        //   any raw data source version of it exists, thanks to the `PhantomData` in
        //   `Self::Auxiliary`, and as the `'d` lifetime in that `PhantomData` is now passed here,
        //   for at least the duration of `'d`, the pointee should not be mutated
        //   (except in `UnsafeCell`).
        unsafe { &*data.as_ptr().cast_const() }
    }

    #[inline]
    unsafe fn as_ref<'a: 'a>(data: NonNull<Self::Data>, _: Outlives<'a, Self>) -> &'a Self::Data {
        // SAFETY: the caller promises that `data` came from `Self::into_raw_data` or
        // `Self::clone_raw`. Since `Self::clone_raw` just returns its `data` argument (which
        // must have come from `into_raw_data` or `clone_raw`), it follows that the `data` pointer
        // derives from a `&T` (and its address was not mutated, and and its provenance shouldn't
        // have been invalidated, as the `PhantomData` ensures that the borrow checker knew that
        // the pointee of the source reference was under a shared borrow from the `into_raw_data`
        // call up to now, and the aliasing rules for data source methods should also ensure that
        // its provenance was not invalidated). Therefore, `data` is:
        // - properly aligned for `Self::Data` (which is `T`), as the pointer came from a `&T`,
        //   which must be properly-aligned,
        // - non-null, since `data` came from a `NonNull`,
        // - dereferenceable for `size_of::<T>()` bytes, as `data` has provenance valid for
        //   reading the pointee of the source `&T`, and
        // - points to a valid value of type `T`, as the pointee of the `&T` was initially
        //   a valid value of type `T`, and exposing shared references to a valid value
        //   of type `T` should not result in an invalid value (for type `T`) being observably
        //   written to the pointee.
        // - This dereference also satisfies Rust's aliasing rules for shared references, since the
        //   caller promises that, while the returned reference exists, no data source method
        //   is called on `data` other than `DataSource::as_ref`. Note also that the
        //   initially-given shared reference had lifetime `'d`, and the returned reference is only
        //   allowed to exist for at most `'d`. The `PhantomData` in `Self::Auxiliary` (along with
        //   the requirement to not drop that auxiliary data while the returned reference exists)
        //   ensures that the the pointee of the initially-given reference is considered to be
        //   under a shared borrow *at least* while the returned reference exists. This means that
        //   only shared access to the pointee of `data` is exposed by data source methods OR
        //   sound external means (as the pointee is known by the borrow checker to be under a
        //   shared borrow). Therefore, while the returned reference exists, the pointee is not
        //   mutated (except in `UnsafeCell`).
        unsafe { &*data.as_ptr().cast_const() }
    }
}

// SAFETY: Moving a `PhantomData` value cannot possibly invalidate a `&T` reference.
unsafe impl<T: ?Sized> CloneableDataSource for &T {
    #[inline]
    unsafe fn clone_raw(
        data: NonNull<Self::Data>,
        _aux: &Self::Auxiliary,
    ) -> (NonNull<Self::Data>, Self::Auxiliary) {
        (data, PhantomData)
    }
}
