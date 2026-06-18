//! # fruits_json_macros
//!
//! Backs `fruits_json` with the `#[derive(JsonSerializable)]` macro, so engine
//! types can be turned into JSON and back without hand-writing the conversion.
//!
//! # How to use
//!
//! The derive is re-exported from `fruits_json` and is reached from there, not
//! from this crate directly — users write `#[derive(JsonSerializable)]` and the
//! macro generates the `to_json`, `from_json`, and `fill_partially_from_json`
//! methods of the `JsonSerializable` trait. The examples below need `JsonValue`,
//! `JsonObject`, and the `JsonSerializable` trait in scope (all from
//! `fruits_json`), so they are shown as `ignore`; see the runnable versions on
//! the `fruits_json` crate page.
//!
//! #### Derive conversion for a struct
//!
//! Apply the derive to a struct of named or tuple fields to encode it as a JSON
//! object keyed by field name.
//!
//! ```ignore
//! use fruits_json::{JsonSerializable, JsonValue};
//!
//! #[derive(JsonSerializable)]
//! struct Player {
//!     name: String,
//!     score: u32,
//! }
//!
//! let player = Player { name: String::from("Mei"), score: 42 };
//! let json = player.to_json();
//! let restored = Player::from_json(&json).unwrap();
//! ```
//!
//! #### Derive conversion for a fieldless enum
//!
//! Apply the derive to an enum whose variants carry no data to encode each value
//! as the variant's name string.
//!
//! ```ignore
//! use fruits_json::JsonSerializable;
//!
//! #[derive(JsonSerializable)]
//! enum Facing {
//!     North,
//!     South,
//! }
//!
//! // `Facing::North` round-trips through the JSON string "North".
//! let json = Facing::North.to_json();
//! let restored = Facing::from_json(&json).unwrap();
//! ```
//!
//! #### Merge incoming JSON into an existing value
//!
//! `fill_partially_from_json` overlays only the fields present in the JSON onto a
//! value already in hand, leaving the rest untouched — useful for applying a
//! partial config or patch.
//!
//! ```ignore
//! use fruits_json::{JsonSerializable, JsonValue};
//!
//! #[derive(JsonSerializable)]
//! struct Settings {
//!     volume: u32,
//!     brightness: u32,
//! }
//!
//! let mut settings = Settings { volume: 50, brightness: 50 };
//! let patch = JsonValue::parse(&mut r#"{"volume":80}"#.chars()).unwrap();
//! settings.fill_partially_from_json(&patch); // volume becomes 80, brightness stays 50
//! ```
//!
//! # How to maintain
//!
//! The derive does not build output with `quote!`. It writes the implementation
//! as Rust *source text* into [`String`] buffers with `write!`, then calls
//! `.parse()` to turn that text back into the returned `TokenStream`. The input
//! is parsed once into a [`syn::DeriveInput`]; everything after that is string
//! assembly. When changing the generated code, read the format strings as the
//! literal Rust they emit (the `{{`/`}}` are escaped braces in the final source).
//!
//! Generics are reproduced textually: the full `impl<...>` clause comes from
//! `input.generics`, and a separate `type_generics` list rebuilds the
//! `Type<...>` arguments from the type and const parameters. A `<{type_generics}>`
//! suffix is always emitted after the type name even when the list is empty,
//! which renders as a harmless empty `<>` for non-generic types. Lifetime
//! parameters are rejected with a panic — the trait requires `Self: Sized` and
//! the macro assumes `'static` types, so non-static types are unsupported.
//!
//! Structs are encoded as a `JsonObject`: `to_json` pushes one field per
//! struct field under its name (tuple fields use their numeric index as the
//! name), `from_json` requires a `JsonValue::Object` and rebuilds every field by
//! name (a missing or unconvertible field makes the whole conversion return
//! `None`), and `fill_partially_from_json` recurses into each field only when the
//! object actually carries it.
//!
//! Fieldless enums are encoded as the variant name string: `to_json` produces a
//! `JsonValue::String`, `from_json` matches that string back to a variant and
//! returns `None` for anything unknown, and `fill_partially_from_json` simply
//! reparses and overwrites the whole value when parsing succeeds. Enums whose
//! variants carry fields are not yet supported and panic; the commented-out block
//! in the variant loop sketches the intended field handling (see the `// todo`s).
//! Unions also panic.
//!
//! All three methods are generated for every accepted shape, so the trait is
//! always implemented in full and the partial-merge path is consistent with the
//! full conversion.

use proc_macro::TokenStream;
use quote::ToTokens;
use std::fmt::Write;
use syn::DeriveInput;

#[proc_macro_derive(JsonSerializable)]
pub fn derive_json_serializable(stream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(stream as DeriveInput);

    let mut result = String::new();

    let impl_generics = input.generics.to_token_stream().to_string();
    let mut type_generics = String::new();

    for param in input.generics.params {
        match param {
            syn::GenericParam::Lifetime(_) => panic!("Non-static types are not supported"),
            syn::GenericParam::Type(type_param) => write!(type_generics, "{},", type_param.ident.to_string()).unwrap(),
            syn::GenericParam::Const(const_param) => write!(type_generics, "{},", const_param.ident.to_string()).unwrap(),
        }
    }

    let type_name = input.ident.to_string();

    write!(
        result,
        r#"impl{impl_generics} JsonSerializable for {type_name}<{type_generics}> {{ "#
    )
    .unwrap();

    let mut to_json_impl = String::new();
    let mut from_json_impl = String::new();
    let mut fill_partially_from_json_impl = String::new();

    write!(to_json_impl, r#"fn to_json(&self) -> JsonValue {{ "#).unwrap();

    write!(from_json_impl, r#"fn from_json(json: &JsonValue) -> Option<Self> {{ "#).unwrap();

    write!(
        fill_partially_from_json_impl,
        r#"fn fill_partially_from_json(&mut self, json: &JsonValue) {{ "#
    )
    .unwrap();

    match input.data {
        syn::Data::Struct(data_struct) => {
            write!(to_json_impl, r#"let mut json = JsonObject::new();"#).unwrap();

            write!(
                from_json_impl,
                r#"let JsonValue::Object(json) = json else {{ return None; }};"#
            )
            .unwrap();
            write!(from_json_impl, r#"Some(Self {{"#).unwrap();

            write!(
                fill_partially_from_json_impl,
                r#"let JsonValue::Object(json) = json else {{ return; }};"#
            )
            .unwrap();

            for (i, field) in data_struct.fields.iter().enumerate() {
                let field_name = field
                    .ident
                    .as_ref()
                    .map(|i| i.to_token_stream().to_string())
                    .unwrap_or_else(|| i.to_string());

                write!(
                    to_json_impl,
                    r#"json.push_field("{field_name}", JsonSerializable::to_json(&self.{field_name})).ok().unwrap();"#
                )
                .unwrap();

                write!(
                    from_json_impl,
                    r#"{field_name}: JsonSerializable::from_json(json.get_value("{field_name}")?)?,"#
                )
                .unwrap();

                write!(fill_partially_from_json_impl, r#"if let Some(field_{field_name}) = json.get_value("{field_name}") {{ self.{field_name}.fill_partially_from_json(field_{field_name}); }}"#).unwrap();
            }

            to_json_impl.push_str("JsonValue::Object(json)");

            from_json_impl.push_str("})");
        }
        syn::Data::Enum(data_enum) => {
            if data_enum.variants.iter().any(|v| v.fields.len() > 0) {
                // todo
                panic!("Enums with fields are not supported.");
            }

            write!(to_json_impl, r#"JsonValue::String(String::from(match self {{"#).unwrap();

            write!(from_json_impl, r#"let JsonValue::String(v) = json else {{ return None; }};"#).unwrap();
            write!(from_json_impl, r#"Some(match v.as_str() {{"#).unwrap();

            write!(
                fill_partially_from_json_impl,
                r#"if let Some(parsed) = Self::from_json(json) {{ *self = parsed; }}"#
            )
            .unwrap();

            for variant in data_enum.variants {
                let variant_name = variant.ident.to_string();

                write!(to_json_impl, r#"Self::{variant_name} => "{variant_name}","#).unwrap();

                write!(from_json_impl, r#""{variant_name}" => Self::{variant_name},"#).unwrap();

                // todo
                // match variant.fields {
                //     syn::Fields::Named(fields_named) => {
                //         for field in fields_named.named {
                //             let field_type = field.ty.to_token_stream().to_string();

                //             write!(to_json_impl, r#"ReflTyId::of::<{field_type}>(),"#).unwrap();
                //         }

                //         write!(to_json_impl, r#"])"#).unwrap();
                //     },
                //     syn::Fields::Unnamed(fields_unnamed) => {
                //         write!(to_json_impl, r#"ReflTyStruct::Tuple(vec!["#).unwrap();

                //         for field in fields_unnamed.unnamed {
                //             let field_name = field.ident.to_token_stream().to_string();
                //             let field_type = field.ty.to_token_stream().to_string();

                //             write!(to_json_impl, r#"(String::from("{field_name}"), ReflTyId::of::<{field_type}>()),"#).unwrap();
                //         }

                //         write!(to_json_impl, r#"])"#).unwrap();
                //     },
                //     syn::Fields::Unit => write!(to_json_impl, r#"ReflTyStruct::Unit"#).unwrap(),
                // }

                // write!(to_json_impl, r#" ),"#).unwrap();
            }

            to_json_impl.push_str("}))");

            from_json_impl.push_str("_ => return None,");
            from_json_impl.push_str("})");
        }
        syn::Data::Union(_) => panic!("Union types are not supported."),
    }

    to_json_impl.push_str(" } ");

    from_json_impl.push_str(" } ");

    fill_partially_from_json_impl.push_str(" } ");

    result.push_str(&to_json_impl);
    result.push_str(&from_json_impl);
    result.push_str(&fill_partially_from_json_impl);
    result.push_str(" } ");

    result.parse().unwrap()
}
