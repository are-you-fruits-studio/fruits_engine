//! # fruits_ffi
//!
//! FFI-safe replacements for common standard-library types — vectors, strings, hash
//! maps, boxes, options, slices, and type-erased values — so engine data can be
//! passed across a stable binary boundary between the engine and the modules it
//! links against. Where `std` types have an unspecified, compiler-chosen layout,
//! the `Ffi*` counterparts here have a fixed one.
//!
//! # How to use
//!
//! These types are used directly wherever a value must keep a stable layout. Each
//! mirrors the part of its `std` counterpart the engine needs, and converts to and
//! from the `std` type at the edges.
//!
//! #### Holding a growable list
//!
//! [`FfiVec<T>`] is the stable-layout counterpart of [`Vec<T>`]. It pushes, pops,
//! indexes, iterates, and converts to and from a `Vec`.
//!
//! ```
//! use fruits_ffi::FfiVec;
//!
//! let mut v: FfiVec<i32> = FfiVec::new();
//! v.push(1);
//! v.push(2);
//! v.push(3);
//!
//! assert_eq!(v.len(), 3);
//! assert_eq!(v.as_slice(), &[1, 2, 3]);
//! assert_eq!(v.pop(), Some(3));
//!
//! // Convert to and from a std Vec at the edges.
//! let from_std: FfiVec<i32> = vec![10, 20].into();
//! assert_eq!(from_std.clone_to_vec(), vec![10, 20]);
//! ```
//!
//! #### Holding text
//!
//! [`FfiString`] is the stable-layout counterpart of [`String`]. It derefs to
//! [`str`], so the usual string methods are available through it.
//!
//! ```
//! use fruits_ffi::FfiString;
//!
//! let mut s = FfiString::from("hello");
//! s.push_str(", world");
//!
//! assert_eq!(s.as_str(), "hello, world");
//! assert!(s.starts_with("hello"));
//! ```
//!
//! #### Carrying an optional value
//!
//! [`FfiOption<T>`] is the stable-layout counterpart of [`Option<T>`], and converts
//! to and from it. It also implements [`Serialize`](serde::Serialize) /
//! [`Deserialize`](serde::Deserialize).
//!
//! ```
//! use fruits_ffi::FfiOption;
//!
//! let some: FfiOption<i32> = Some(5).into();
//! assert!(some.is_some());
//! assert_eq!(some.into_option(), Some(5));
//!
//! let none = FfiOption::<i32>::None;
//! assert_eq!(none.into_option(), None);
//! ```
//!
//! #### Owning a single heap value
//!
//! [`FfiBox<T>`] is the stable-layout counterpart of [`Box<T>`]. It derefs to the
//! value and can hand the value back out.
//!
//! ```
//! use fruits_ffi::FfiBox;
//!
//! let boxed = FfiBox::new(42);
//! assert_eq!(*boxed, 42);
//! assert_eq!(boxed.into_inner(), 42);
//! ```
//!
//! #### Mapping keys to values
//!
//! [`FfiHashMap<K, V>`] is the stable-layout counterpart of
//! [`HashMap<K, V>`](std::collections::HashMap), with
//! insert / remove / get / get_mut.
//!
//! ```
//! use fruits_ffi::FfiHashMap;
//!
//! let mut map: FfiHashMap<i32, String> = FfiHashMap::new();
//! map.insert(1, "one".to_string());
//!
//! assert_eq!(map.get(&1).map(String::as_str), Some("one"));
//! assert_eq!(map.remove(&1).as_deref(), Some("one"));
//! assert!(map.get(&1).is_none());
//! ```
//!
//! #### Looking up a string-keyed map without allocating a key
//!
//! When the key type is [`FfiString`], a map can be queried with a plain `&str`, so
//! a lookup needs no owned key.
//!
//! ```
//! use fruits_ffi::{FfiHashMap, FfiString};
//!
//! let mut map: FfiHashMap<FfiString, i32> = FfiHashMap::new();
//! map.insert(FfiString::from("health"), 100);
//!
//! assert_eq!(map.get_by_str("health"), Some(&100));
//! assert_eq!(map.get_by_str("missing"), None);
//! ```
//!
//! #### Passing an optional reference as a nullable pointer
//!
//! [`ref_into_nullable_ptr`] / [`mut_into_nullable_ptr`] turn an `Option<&T>` into a
//! raw pointer that is null for `None`, and [`ref_from_nullable_ptr`] /
//! [`mut_from_nullable_ptr`] recover the `Option` on the other side.
//!
//! ```
//! use fruits_ffi::{ref_into_nullable_ptr, ref_from_nullable_ptr};
//!
//! let value = 10;
//! let ptr = ref_into_nullable_ptr(Some(&value));
//! assert_eq!(unsafe { ref_from_nullable_ptr(ptr) }, Some(&10));
//!
//! let null = ref_into_nullable_ptr::<i32>(None);
//! assert!(null.is_null());
//! ```
//!
//! # How to maintain
//!
//! #### The problem these types solve
//!
//! The layout of `std` collections (`Vec`, `String`, `HashMap`, `Box`, `Option`) is
//! deliberately unspecified, and generic code is monomorphized per crate. Neither
//! property survives a binary boundary: two separately compiled sides cannot agree
//! on a `Vec`'s field order, nor share the monomorphized `Drop`/`Hash`/lookup code
//! for a generic type. Every type here exists to remove one of those obstacles.
//!
//! #### Two families of types
//!
//! - **Plain `#[repr(C)]` data** with a fully fixed, self-contained layout:
//!   [`FfiOption`], [`FfiSliceRef`] / [`FfiSliceMut`], and the string-slice views
//!   [`FfiStrSliceRef`] / [`FfiStrSliceMut`]. These own no heap behavior of their
//!   own — a slice view is a pointer plus a [`u64`] length, an option is a tagged
//!   union — so a fixed layout is all they need.
//! - **Owning types that carry their behavior as embedded `extern "C"` function
//!   pointers.** [`FfiAny`], [`FfiDroppable`], [`FfiHashMap`], and [`FfiAllocator`]
//!   store the operations they need (drop, deallocate, insert, look up) as function
//!   pointers alongside the data, because the consuming side cannot monomorphize
//!   those operations itself. The data travels with a small table of pointers that
//!   knows how to act on it.
//!
//! #### Function-pointer tables
//!
//! [`FfiAny`] is a type-erased owned value: a raw pointer plus a `&'static`
//! [`FfiAnyMetadata`] holding the value's size, align, type name, `drop_in_place`,
//! and `dealloc`. The metadata is built in a `const` block per `T`, so each type
//! gets one promoted-`'static` table. [`FfiBox<T>`] is `#[repr(transparent)]` over
//! `FfiAny` and adds the type back as a `PhantomData`; [`into_inner`](FfiBox::into_inner)
//! reads the value out and then deallocates the storage *without* running its
//! destructor, so the value is moved rather than dropped.
//!
//! [`FfiDroppable`] boxes a value behind a single opaque pointer whose allocation
//! begins with a metadata header (an `extern "C"` drop function and a pointer to the
//! value); dropping the `FfiDroppable` calls through that header. [`FfiTypedDroppable<T>`]
//! is the typed wrapper that derefs to the value and forwards the standard traits.
//! [`FfiStaticRef<T>`] is a `#[repr(transparent)]`, [`Copy`] pointer to a `'static`
//! value, used to share these tables cheaply.
//!
//! [`FfiHashMap<K, V>`] composes both ideas: the actual data is a real `std`
//! [`HashMap`](std::collections::HashMap) held behind an [`FfiDroppable`], and every operation is reached
//! through an [`FfiStaticRef`] to a per-`(K, V)` vtable of `extern "C"` functions
//! that down-cast the opaque pointer and call the matching `HashMap` method. The
//! `FfiString`-keyed `get_by_str` / `remove_by_str` entries take an
//! [`FfiStrSliceRef`] so a borrowed `&str` can probe the map without building an
//! owned key.
//!
//! #### Allocation across the boundary
//!
//! [`FfiAllocator`] is a `#[repr(C)]`, [`Copy`] pair of `extern "C"` alloc/dealloc
//! function pointers; [`from_global`](FfiAllocator::from_global) routes to the global
//! allocator and [`from_system`](FfiAllocator::from_system) to [`System`](std::alloc::System).
//! [`FfiVec`] stores its `FfiAllocator` inline, so a buffer is always freed through
//! the same allocator that produced it even if the side dropping it differs from the
//! side that grew it. [`FfiTypedMemory<T>`] / [`FfiOpaqueMemory`] are single-value
//! allocations that likewise carry their own `dealloc` pointer.
//!
//! #### FfiVec internals
//!
//! All vector layouts share a private `FfiRawVec` (pointer, capacity, length,
//! allocator). [`FfiVec<T>`] is `#[repr(transparent)]` over it plus `PhantomData`,
//! while [`FfiOpaqueVec`] is the type-erased twin that records element size, element
//! align, and an optional `drop_fn` instead of a generic parameter; because
//! `FfiRawVec` is its first `#[repr(C)]` field, the two transmute into one another.
//! Capacity doubles on growth (starting at 4). Zero-sized types store no buffer and
//! pin capacity to `ZERO_SIZED_CAP`, tracking only the length. The `Send`/`Sync`
//! impls are written by hand and conditioned on `T`.
//!
//! #### Invariants and caveats
//!
//! - Lengths and sizes are [`u64`] (not [`usize`]) so the layout is identical on
//!   32- and 64-bit targets; they are cast to `usize` at use sites.
//! - [`FfiString`] keeps its bytes as an `FfiVec<u8>` that is assumed to always be
//!   valid UTF-8 — `as_str` uses an unchecked conversion, so any new mutating path
//!   must preserve that invariant.
//! - Many constructors and accessors are `unsafe` because pointer validity and the
//!   returned lifetimes cannot be checked here; callers uphold them.
//! - The crate is still incomplete: `// todo` markers flag missing trait impls,
//!   unoptimized paths, and drop behavior that is not yet sound under exotic panics.
//!
//! Developer note: the source carries `// todo` markers (e.g. a reallocating
//! `into_vec` and panic-safe drops in [`FfiVec`]) that record intended but
//! unfinished work; leave them in place.

mod alloc;
mod boxed;
mod convert;
mod drop;
mod hash_map;
mod index_map;
mod index_set;
mod option;
mod slice;
mod string;
mod vec;
mod any;
mod closure;

pub use alloc::*;
pub use boxed::*;
pub use convert::*;
pub use drop::*;
pub use hash_map::*;
pub use index_map::*;
pub use index_set::*;
pub use option::*;
pub use slice::*;
pub use string::*;
pub use vec::*;
pub use any::*;
pub use closure::*;
