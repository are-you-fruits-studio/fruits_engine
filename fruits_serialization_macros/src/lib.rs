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

    for param in input.generics.params {
        match param {
            syn::GenericParam::Lifetime(lifetime_param) => write!(type_generics, "{},", lifetime_param.lifetime.to_string()).unwrap(),
            syn::GenericParam::Type(type_param) => write!(type_generics, "{},", type_param.ident.to_string()).unwrap(),
            syn::GenericParam::Const(const_param) => write!(type_generics, "{},", const_param.ident.to_string()).unwrap(),
        }
    }

    let type_name = input.ident.to_string();

    write!(result, r#"impl{impl_generics} TransSerializable for {type_name}<{type_generics}> where Self: 'static "#).unwrap();

    let mut where_clause_text = String::new();

    if let Some(where_clause) = input.generics.where_clause {
        where_clause_text.push_str(", ");
        where_clause_text.push_str(&where_clause.predicates.to_token_stream().to_string());
        result.push_str(&where_clause_text);
    }

    write!(result, r#" {{ "#).unwrap();

    let mut impl_serialize = String::new();
    let mut impl_deserialize = String::new();

    write!(impl_serialize, r#"fn serialize(&self, ctx: &SerializerCtx) -> Result<serde_json::Value, SerializationError> {{ "#).unwrap();

    write!(impl_deserialize, r#"fn deserialize(ctx: &SerializerCtx, value: &serde_json::Value) -> Result<Self, DeserializationError> {{ "#).unwrap();

    match input.data {
        syn::Data::Struct(data_struct) => {
            let is_tuple = matches!(&data_struct.fields, syn::Fields::Unnamed{ .. });

            match is_tuple {
                true => write!(impl_serialize, r#"Ok(serde_json::Value::Array(vec!["#).unwrap(),
                false => write!(impl_serialize, r#"Ok(serde_json::Value::Object(["#).unwrap(),
            };

            match is_tuple {
                true => write!(impl_deserialize, r#"let serde_json::Value::Array(array) = value else {{ return Err(DeserializationError::InvalidInput); }};"#).unwrap(),
                false => write!(impl_deserialize, r#"let serde_json::Value::Object(map) = value else {{ return Err(DeserializationError::InvalidInput); }};"#).unwrap(),
            }
            
            match is_tuple {
                true => write!(impl_deserialize, r#"Ok(Self("#).unwrap(),
                false => write!(impl_deserialize, r#"Ok(Self {{"#).unwrap(),
            }

            match &data_struct.fields {
                syn::Fields::Named(fields_named) => {
                    for field in &fields_named.named {
                        let field_name = field.ident.as_ref().unwrap().to_token_stream().to_string();

                        write!(
                            impl_serialize,
                            r#"(String::from("{field_name}"), ctx.serialize(&self.{field_name})?),"#
                        )
                        .unwrap();

                        write!(
                            impl_deserialize,
                            r#"{field_name}: ctx.deserialize(map.get("{field_name}").ok_or_else(|| DeserializationError::InvalidInput)?)?,"#
                        )
                        .unwrap();
                    }
                },
                syn::Fields::Unnamed(fields_unnamed) => {
                    for (field_idx, _) in fields_unnamed.unnamed.iter().enumerate() {
                        write!(
                            impl_serialize,
                            r#"ctx.serialize(&self.{field_idx})?,"#
                        )
                        .unwrap();

                        write!(
                            impl_deserialize,
                            r#"ctx.deserialize(array.get({field_idx}).ok_or_else(|| DeserializationError::InvalidInput)?)?,"#
                        )
                        .unwrap();
                    }
                },
                syn::Fields::Unit => (),
            }

            match is_tuple {
                true => write!(impl_serialize, r#"]))"#).unwrap(),
                false => write!(impl_serialize, r#"].into_iter().collect()))"#).unwrap(),
            }

            match is_tuple {
                true => write!(impl_deserialize, r#"))"#).unwrap(),
                false => write!(impl_deserialize, r#"}})"#).unwrap(),
            }
        }
        syn::Data::Enum(data_enum) => {
            write!(impl_serialize, r#"Ok(match self {{"#).unwrap();

            write!(impl_deserialize, r#"let serde_json::Value::Object(object) = value else {{ return Err(DeserializationError::InvalidInput); }};"#).unwrap();
            write!(impl_deserialize, r#"if object.len() != 1 {{ return Err(DeserializationError::InvalidInput); }}"#).unwrap();
            write!(impl_deserialize, r#"let (variant, data) = object.iter().next().unwrap();"#).unwrap();
            write!(impl_deserialize, r#"match (variant.as_str(), data) {{"#).unwrap();

            for variant in data_enum.variants {
                let variant_name = variant.ident.to_string();

                let deconstruction = match &variant.fields {
                    syn::Fields::Named(fields_named) => format!(" {{ {} }}", fields_named.named.iter().map(|f| f.ident.as_ref().unwrap().to_token_stream().to_string()).collect::<Vec<_>>().join(", ")),
                    syn::Fields::Unnamed(fields_unnamed) => format!("( {} )", (0..fields_unnamed.unnamed.len()).into_iter().map(|f| format!("f{f}")).collect::<Vec<_>>().join(", ")),
                    syn::Fields::Unit => String::from(""),
                };

                write!(impl_serialize, r#"Self::{variant_name}{deconstruction} => serde_json::Value::Object([(String::from("{variant_name}"), "#).unwrap();

                match &variant.fields {
                    syn::Fields::Named(fields_named) => {
                        write!(impl_serialize, r#"serde_json::Value::Object(["#).unwrap();
                        write!(impl_deserialize, r#"("{variant_name}", serde_json::Value::Object(object)) => Ok(Self::{variant_name} {{"#).unwrap();
                        
                        for field in &fields_named.named {
                            let field_name = field.ident.as_ref().unwrap().to_token_stream().to_string();
                            
                            write!(impl_serialize, r#"(("{field_name}", ctx.serialize({field_name})?),"#).unwrap();
                            write!(impl_deserialize, r#"{field_name}: ctx.deserialize(object.get("{field_name}").ok_or_else(|| DeserializationError::InvalidInput)?)?,"#).unwrap();
                        }
                        
                        write!(impl_serialize, r#"].into_iter().collect()),"#).unwrap();
                        write!(impl_deserialize, r#"}}),"#).unwrap();
                    },
                    syn::Fields::Unnamed(fields_unnamed) => {
                        write!(impl_serialize, r#"serde_json::Value::Array(vec!["#).unwrap();
                        write!(impl_deserialize, r#"("{variant_name}", serde_json::Value::Array(array)) => Ok(Self::{variant_name}("#).unwrap();

                        for (field_idx, _) in fields_unnamed.unnamed.iter().enumerate() {
                            write!(impl_serialize, r#"ctx.serialize(f{field_idx})?,"#).unwrap();
                            write!(impl_deserialize, r#"ctx.deserialize(array.get({field_idx}).ok_or_else(|| DeserializationError::InvalidInput)?)?,"#).unwrap();
                        }

                        write!(impl_serialize, r#"]),"#).unwrap();
                        write!(impl_deserialize, r#")),"#).unwrap();
                    },
                    syn::Fields::Unit => {
                        write!(impl_serialize, r#"serde_json::Value::Object([].into_iter().collect()),"#).unwrap();
                        write!(impl_deserialize, r#"("{variant_name}", _) => Ok(Self::{variant_name}),"#).unwrap();
                    },
                }

                write!(impl_serialize, r#")].into_iter().collect())"#).unwrap();
            }

            impl_serialize.push_str("})");

            impl_deserialize.push_str("_ => Err(DeserializationError::InvalidInput),");
            impl_deserialize.push_str("}");
        }
        syn::Data::Union(_) => panic!("Union types are not supported."),
    }

    impl_serialize.push_str(" } ");

    impl_deserialize.push_str(" } ");

    result.push_str(&impl_serialize);
    result.push_str(&impl_deserialize);
    result.push_str(" } ");

    result.parse().unwrap()
}