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
use quote::ToTokens;
use std::fmt::Write;
use syn::DeriveInput;

#[proc_macro_derive(TransSerializable)]
pub fn derive_json_serializable(stream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(stream as DeriveInput);

    let mut result = String::new();

    let impl_generics = input.generics.to_token_stream().to_string();
    let mut type_generics = String::new();

    for param in &input.generics.params {
        match param {
            syn::GenericParam::Lifetime(lifetime_param) => {
                write!(type_generics, "{},", lifetime_param.lifetime.to_string()).unwrap()
            }
            syn::GenericParam::Type(type_param) => write!(type_generics, "{},", type_param.ident.to_string()).unwrap(),
            syn::GenericParam::Const(const_param) => write!(type_generics, "{},", const_param.ident.to_string()).unwrap(),
        }
    }

    let type_name = input.ident.to_string();

    write!(
        result,
        r#"impl{impl_generics} TransSerializable for {type_name}<{type_generics}> where Self: 'static "#
    )
    .unwrap();

    let mut where_clause_text = String::new();

    if let Some(where_clause) = &input.generics.where_clause {
        where_clause_text.push_str(", ");
        where_clause_text.push_str(&where_clause.predicates.to_token_stream().to_string());
        result.push_str(&where_clause_text);
    }

    result.push_str(" { ");

    result.push_str(&serialize_impl(&input));
    result.push_str(&deserialize_impl(&input));
    result.push_str(" } ");

    result.parse().unwrap()
}

fn serialize_impl(input: &DeriveInput) -> String {
    let mut impl_serialize = String::new();

    impl_serialize.push_str("fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<SerializedValue> { ");

    match &input.data {
        syn::Data::Struct(data_struct) => {
            match &data_struct.fields {
                syn::Fields::Named(fields_named) => {
                    impl_serialize.push_str("ctx.serialize_map()");

                    for field in &fields_named.named {
                        let field = field.ident.as_ref().unwrap().to_token_stream().to_string();

                        write!(
                            impl_serialize,
                            r#".with_field("{field}", &self.{field})"#
                        )
                        .unwrap();
                    }
                    
                    impl_serialize.push_str(".finish_as_map(true)");
                },
                syn::Fields::Unnamed(fields_unnamed) => {
                    impl_serialize.push_str("ctx.serialize_map()");

                    for field in 0..fields_unnamed.unnamed.len() {
                        write!(
                            impl_serialize,
                            r#".with_field("{field}", &self.{field})"#
                        )
                        .unwrap();
                    }
                    
                    impl_serialize.push_str(".finish_as_map(true)");
                },
                syn::Fields::Unit => {
                    impl_serialize.push_str("ctx.serialize_map().finish_as_map(true)");
                },
            }
        },
        syn::Data::Enum(data_enum) => {
            let mut variants_line = String::new();

            variants_line.push_str("let variants = [");

            for (i, variant) in data_enum.variants.iter().enumerate() {
                if i > 0 {
                    variants_line.push_str(", ");
                }

                variants_line.push('"');
                variants_line.push_str(&variant.ident.to_token_stream().to_string());
                variants_line.push('"');
            }

            variants_line.push_str("].into_iter().map(FfiString::from).collect();");

            //

            impl_serialize.push_str(&variants_line);

            impl_serialize.push_str("match self {");

            for variant in &data_enum.variants {
                let variant_name = variant.ident.to_token_stream().to_string();

                match &variant.fields {
                    syn::Fields::Named(fields_named) => {
                        let fields_deconstruct = fields_named.named.iter()
                            .map(|f| f.ident.to_token_stream().to_string())
                            .map(|f| format!("{f}: f_{f}"))
                            .collect::<Vec<_>>()
                            .join(", ");

                        write!(
                            impl_serialize,
                            r#"Self::{variant_name} {{ {fields_deconstruct} }} => ctx.serialize_map()"#
                        )
                        .unwrap();

                        for field in &fields_named.named {
                            let field = field.ident.to_token_stream().to_string();

                            write!(
                                impl_serialize,
                                r#".with_field("{field}", f_{field})"#
                            )
                            .unwrap();
                        }
                        
                        write!(
                            impl_serialize,
                            r#".finish_as_enum(true, "{variant_name}", variants),"#
                        )
                        .unwrap();
                    },
                    syn::Fields::Unnamed(fields_unnamed) => {
                        let fields_deconstruct = (0..fields_unnamed.unnamed.len())
                            .map(|f| format!("arg_{f}"))
                            .collect::<Vec<_>>()
                            .join(", ");

                        write!(
                            impl_serialize,
                            r#"Self::{variant_name}({fields_deconstruct}) => ctx.serialize_map()"#
                        )
                        .unwrap();

                        for field in 0..fields_unnamed.unnamed.len() {
                            write!(
                                impl_serialize,
                                r#".with_field("{field}", arg_{field})"#
                            )
                            .unwrap();
                        }
                        
                        write!(
                            impl_serialize,
                            r#".finish_as_enum(true, "{variant_name}", variants),"#
                        )
                        .unwrap();
                    },
                    syn::Fields::Unit => {
                        write!(
                            impl_serialize,
                            r#"Self::{variant_name} => ctx.serialize_map().finish_as_enum(true, "{variant_name}", variants),"#
                        )
                        .unwrap();
                    },
                }
            }

            impl_serialize.push_str("}");
        },
        syn::Data::Union(_) => panic!("Union types are not supported."),
    }
    
    impl_serialize.push_str(" } ");

    impl_serialize
}

fn deserialize_impl(input: &DeriveInput) -> String {
    let mut impl_deserialize = String::new();

    impl_deserialize.push_str("fn deserialize(ctx: &SerializerCtx, value: &SerializedValue) -> SerializationResult<Option<Self>> { ");

    match &input.data {
        syn::Data::Struct(data_struct) => {

            match &data_struct.fields {
                syn::Fields::Named(fields_named) => {
                    impl_deserialize.push_str("ctx.deserialize_map(value, |ctx| { Some(Self {");

                    for field in &fields_named.named {
                        let field = field.ident.as_ref().unwrap().to_token_stream().to_string();

                        write!(
                            impl_deserialize,
                            r#"{field}: ctx.get_field("{field}")?,"#
                        )
                        .unwrap();
                    }
                    
                    impl_deserialize.push_str("})})");
                },
                syn::Fields::Unnamed(fields_unnamed) => {
                    impl_deserialize.push_str("ctx.deserialize_map(value, |ctx| { Some(Self(");

                    for field in 0..fields_unnamed.unnamed.len() {
                        write!(
                            impl_deserialize,
                            r#"ctx.get_field("{field}")?,"#
                        )
                        .unwrap();
                    }
                    
                    impl_deserialize.push_str("))})");
                },
                syn::Fields::Unit => {
                    impl_deserialize.push_str("ctx.deserialize_map(value, |ctx| { Some(Self)})");
                },
            }
        },
        syn::Data::Enum(data_enum) => {
            impl_deserialize.push_str("ctx.deserialize_enum()");

            for variant in &data_enum.variants {
                let variant_name = variant.ident.to_string();

                write!(
                    impl_deserialize,
                    r#".variant("{variant_name}", |ctx, value| {{"#
                )
                .unwrap();
            
                match &variant.fields {
                    syn::Fields::Named(fields_named) => {
                        impl_deserialize.push_str("");

                        write!(
                            impl_deserialize,
                            r#"ctx.deserialize_map(value, |ctx| {{ Some(Self::{variant_name} {{"#
                        )
                        .unwrap();

                        for field in &fields_named.named {
                            let field = field.ident.as_ref().unwrap().to_token_stream().to_string();

                            write!(
                                impl_deserialize,
                                r#"{field}: ctx.get_field("{field}")?,"#
                            )
                            .unwrap();
                        }

                        impl_deserialize.push_str("})})");
                    },
                    syn::Fields::Unnamed(fields_unnamed) => {
                        impl_deserialize.push_str("");

                        write!(
                            impl_deserialize,
                            r#"ctx.deserialize_map(value, |ctx| {{ Some(Self::{variant_name} ("#
                        )
                        .unwrap();

                        for field in 0..fields_unnamed.unnamed.len() {
                            write!(
                                impl_deserialize,
                                r#"ctx.get_field("{field}")?,"#
                            )
                            .unwrap();
                        }
                    
                        impl_deserialize.push_str("))})");
                    },
                    syn::Fields::Unit => {
                        write!(
                            impl_deserialize,
                            r#"ctx.deserialize_map(value, |ctx| {{ Some(Self::{variant_name}) }})"#
                        )
                        .unwrap();
                    },
                }

                impl_deserialize.push_str("})");
            }
            
            impl_deserialize.push_str(".finish(value)");
        }
        syn::Data::Union(_) => panic!("Union types are not supported."),
    }

    impl_deserialize.push_str(" } ");

    impl_deserialize
}
