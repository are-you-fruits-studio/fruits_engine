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

    impl_serialize.push_str("fn serialize(&self, ctx: &SerializerCtx) -> SerializationResult<serde_json::Value> { ");

    impl_serialize.push_str(r#"let mut err = None;"#);
    impl_serialize.push_str(r#"SerializationResult { result: "#);

    match &input.data {
        syn::Data::Struct(data_struct) => impl_serialize.push_str(&serialize_fields_impl(&data_struct.fields, "&self.")),
        syn::Data::Enum(data_enum) => {
            impl_serialize.push_str("match self {");

            for variant in &data_enum.variants {
                let variant_name = variant.ident.to_string();

                let deconstruction = match &variant.fields {
                    syn::Fields::Named(fields_named) => format!(
                        " {{ {} }}",
                        fields_named
                            .named
                            .iter()
                            .map(|f| f.ident.as_ref().unwrap().to_token_stream().to_string())
                            .map(|f| format!("{f}: f_{f}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    syn::Fields::Unnamed(fields_unnamed) => format!(
                        "( {} )",
                        (0..fields_unnamed.unnamed.len())
                            .into_iter()
                            .map(|f| format!("f_{f}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    syn::Fields::Unit => String::from(""),
                };

                write!(
                    impl_serialize,
                    r#"Self::{variant_name}{deconstruction} => serde_json::Value::Object([(String::from("{variant_name}"), "#
                )
                .unwrap();

                impl_serialize.push_str(&serialize_fields_impl(&variant.fields, "f_"));

                impl_serialize.push_str(")].into_iter().collect()),");
            }

            impl_serialize.push_str("}");
        }
        syn::Data::Union(_) => panic!("Union types are not supported."),
    }

    impl_serialize.push_str(", err, }");

    impl_serialize.push_str(" } ");

    impl_serialize
}

fn serialize_fields_impl(fields: &syn::Fields, field_prefix: &str) -> String {
    let mut impl_serialize = String::new();

    match fields {
        syn::Fields::Named(fields_named) => {
            impl_serialize.push_str("serde_json::Value::Object([");

            for field in &fields_named.named {
                let field_name = field.ident.as_ref().unwrap().to_token_stream().to_string();

                write!(
                    impl_serialize,
                    r#"(String::from("{field_name}"), err.consume_non_fatal(ctx.serialize({field_prefix}{field_name}))),"#
                )
                .unwrap();
            }

            impl_serialize.push_str("].into_iter().collect())");
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            impl_serialize.push_str("serde_json::Value::Array(vec![");

            for (field_idx, _) in fields_unnamed.unnamed.iter().enumerate() {
                write!(impl_serialize, r#"err.consume_non_fatal(ctx.serialize({field_prefix}{field_idx})),"#).unwrap();
            }

            impl_serialize.push_str("])");
        }
        syn::Fields::Unit => {
            impl_serialize.push_str("serde_json::Value::Object([].into_iter().collect())");
        }
    }

    impl_serialize
}

fn deserialize_impl(input: &DeriveInput) -> String {
    let mut impl_deserialize = String::new();

    impl_deserialize.push_str("fn deserialize(ctx: &SerializerCtx, value: &serde_json::Value) -> Result<SerializationResult<Self>, SerializationError> { ");

    match &input.data {
        syn::Data::Struct(data_struct) => {
            impl_deserialize.push_str(&deserialize_fields_impl("Self", &data_struct.fields));
        }
        syn::Data::Enum(data_enum) => {
            for variant in &data_enum.variants {
                let variant_name = variant.ident.to_string();

                write!(impl_deserialize, r#"let variant_{variant_name} = |ctx: &SerializerCtx, value: &serde_json::Value| -> Result<SerializationResult<Self>, SerializationError> {{"#).unwrap();

                impl_deserialize.push_str(&deserialize_fields_impl(&format!("Self::{variant_name}"), &variant.fields));
                
                impl_deserialize.push_str("};");
            }
            
            impl_deserialize.push_str(r#"
            let mut err = Option::<SerializationError>::None;

            let result = 'parsing: {
                let serde_json::Value::Object(object) = value else {
                    break 'parsing Err(SerializationError::InvalidInput);
                };
                if object.len() != 1 {
                    break 'parsing Err(SerializationError::InvalidInput);
                }
                let (variant, data) = object.iter().next().unwrap();
                match variant.as_str() {
            "#);
            for variant in &data_enum.variants {
                let variant_name = variant.ident.to_string();

                write!(impl_deserialize, r#""{variant_name}" => variant_{variant_name}(ctx, data),"#).unwrap();
            }
            impl_deserialize.push_str(r#"
                    _ => Err(SerializationError::InvalidInput),
                }
            };

            let err = match result {
                Ok(x) => return Ok(x),
                Err(new_err) => err.unwrap_or(new_err),
            };
            "#);

            
            for variant in &data_enum.variants {
                let variant_name = variant.ident.to_string();

                write!(impl_deserialize, r#"if let Ok(SerializationResult {{ result, err: _ }}) = variant_{variant_name}(ctx, &serde_json::Value::Null) {{"#).unwrap();
                write!(impl_deserialize, r#"    return Ok(SerializationResult {{ result, err: Some(err) }});"#).unwrap();
                write!(impl_deserialize, r#"}}"#).unwrap();
            }

            impl_deserialize.push_str("Err(err)");
        }
        syn::Data::Union(_) => panic!("Union types are not supported."),
    }

    impl_deserialize.push_str(" } ");

    impl_deserialize
}

fn deserialize_fields_impl(type_name: &str, fields: &syn::Fields) -> String {
    let mut impl_deserialize = String::new();

    impl_deserialize.push_str("let mut err = Option::<SerializationError>::None;");

    match &fields {
        syn::Fields::Named(fields_named) => {
            impl_deserialize.push_str("let map = match value { serde_json::Value::Object(map) => map, _ => &serde_json::Map::new(), };");
            write!(impl_deserialize, r#"Ok(SerializationResult {{ result: {type_name} {{"#).unwrap();

            for field in &fields_named.named {
                let field_name = field.ident.as_ref().unwrap().to_token_stream().to_string();

                write!(
                    impl_deserialize,
                    r#"{field_name}: err.consume_non_fatal(ctx.deserialize(map.get("{field_name}").unwrap_or_else(|| &serde_json::Value::Null))?),"#
                )
                .unwrap();
            }

            impl_deserialize.push_str("}, err, })");
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            impl_deserialize.push_str("let array = match value { serde_json::Value::Array(array) => array, _ => &Vec::new(), };");
            write!(impl_deserialize, r#"Ok(SerializationResult {{ result: {type_name}("#).unwrap();

            for (field_idx, _) in fields_unnamed.unnamed.iter().enumerate() {
                write!(
                    impl_deserialize,
                    r#"err.consume_non_fatal(ctx.deserialize(array.get({field_idx}).unwrap_or_else(|| &serde_json::Value::Null))?),"#
                )
                .unwrap();
            }

            impl_deserialize.push_str("), err, })");
        }
        syn::Fields::Unit => {
            write!(impl_deserialize, r#"Ok(SerializationResult {{ result: {type_name}, err, }})"#).unwrap();
        },
    }

    impl_deserialize
}