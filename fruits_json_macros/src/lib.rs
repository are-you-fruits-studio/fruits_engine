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
