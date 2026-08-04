//! # fruits_serialization_macros
//!
//! Provides the `#[derive(TransSerializable)]` macro that writes the boilerplate
//! `TransSerializable` impl for a struct or enum, so types opt into the engine's
//! serialization framework without hand-writing field-by-field (de)serialization.
//!
//! This crate is the procedural-macro half of `fruits_serialization`; the macro is
//! re-exported from there, so users derive it through `fruits_serialization` rather than
//! depending on this crate directly.
//!
//! # How to use
//!
//! Annotate a type with `#[derive(TransSerializable)]`. The derive generates code that names
//! `TransSerializable`, `SerializerCtx`, `SerializedValue`, and `SerializationResult`
//! unqualified, so those names must be in scope at the derive site — `use
//! fruits_serialization::*;` brings them in. After deriving, register the type (and every type
//! it contains) on a serializer and round-trip it; that workflow lives in `fruits_serialization`.
//!
//! #### Derive on a struct
//!
//! Each named field becomes a map entry keyed by the field name:
//!
//! ```ignore
//! use fruits_serialization::*;
//!
//! #[derive(TransSerializable)]
//! struct Player {
//!     name: String,
//!     score: u32,
//! }
//! ```
//!
//! Tuple structs are supported too — their fields are keyed by index (`"0"`, `"1"`, …) — as
//! are unit structs, which serialize as an empty map.
//!
//! #### Derive on an enum
//!
//! Every variant is supported (unit, tuple, and struct variants). The serialized value records
//! which variant is active along with the full list of variant names:
//!
//! ```ignore
//! use fruits_serialization::*;
//!
//! #[derive(TransSerializable)]
//! enum Shape {
//!     Empty,
//!     Circle(f32),
//!     Rect { w: f32, h: f32 },
//! }
//! ```
//!
//! #### Derive on a generic type
//!
//! Generic parameters, including lifetimes and const generics, are carried through to the
//! generated impl, and any `where` clause on the type is preserved:
//!
//! ```ignore
//! use fruits_serialization::*;
//!
//! #[derive(TransSerializable)]
//! struct Wrapper<T> {
//!     value: T,
//! }
//! ```
//!
//! # How to maintain
//!
//! #### Code generation by string-building
//!
//! Unlike most derives, this one does not emit a token stream with `quote!`. It appends Rust
//! *source text* to a `String` and parses it back with `result.parse().unwrap()` at the end of
//! `derive_json_serializable`. The two halves of the impl are produced separately by
//! `serialize_impl` and `deserialize_impl`, which each `match` on `input.data` and write the
//! method body for the struct/enum shape at hand. When changing the generated code, remember
//! you are writing strings: every brace in a `write!` format string that should reach the
//! output must be escaped (`{{` / `}}`).
//!
//! #### Generics and the emitted header
//!
//! The impl header is assembled from `input.generics`: `impl_generics` is the verbatim generic
//! list, while `type_generics` is rebuilt by iterating the params and emitting the lifetime,
//! type, or const ident for each (trailing comma included). The header is
//! `impl{impl_generics} TransSerializable for {type_name}<{type_generics}> where Self: 'static`,
//! with the type's own `where` predicates appended after a comma when present. The
//! `Self: 'static` bound matches `TransSerializable`'s `'static` requirement in
//! `fruits_serialization`.
//!
//! #### How each shape maps to a `SerializedValue`
//!
//! Serialization always builds a *rigid* composite (`finish_as_map(true)` /
//! `finish_as_enum(true, …)`), marking the value as coming from a fixed shape. Named struct
//! fields are keyed by field name; tuple fields are keyed by their index rendered as a string;
//! unit structs produce an empty map. Enums emit a map tagged via `finish_as_enum` with the
//! active variant name and a `variants` vector of every variant name (built once, before the
//! `match`). Deserialization mirrors this: structs go through `ctx.deserialize_map(value, …)`
//! reading each field with `ctx.get_field("…")`, and enums go through
//! `ctx.deserialize_enum().variant("Name", …)….finish(value)`.
//!
//! #### Variant binding names
//!
//! When generating the `match` arm for an enum variant, struct-variant fields are bound to
//! `f_<field>` and tuple-variant fields to `arg_<index>`, keeping the generated bindings from
//! colliding with the field/index names used as map keys.
//!
//! #### Unions are rejected
//!
//! `syn::Data::Union` is unsupported; both `serialize_impl` and `deserialize_impl` panic on it,
//! surfacing as a compile error at the derive site.

use proc_macro::TokenStream;

mod impl_trans_serializable;
mod impl_serializable;

#[proc_macro_derive(TransSerializable)]
pub fn derive_trans_serializable(stream: TokenStream) -> TokenStream {
    impl_trans_serializable::derive(stream)
}

#[proc_macro_derive(Serializable)]
pub fn derive_serializable(stream: TokenStream) -> TokenStream {
    impl_serializable::derive(stream)
}
