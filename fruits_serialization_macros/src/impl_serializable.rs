use proc_macro::TokenStream;
use quote::ToTokens;
use std::fmt::Write;
use syn::DeriveInput;

pub fn derive(stream: TokenStream) -> TokenStream {
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
        r#"impl{impl_generics} Serializable for {type_name}<{type_generics}> where Self: 'static "#
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

    impl_serialize.push_str("fn serialize(&self, mut ctx: PureSerializerCtx) -> SerializedValue { ");

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

    impl_deserialize.push_str("fn deserialize(mut ctx: PureSerializerCtx, value: &SerializedValue) -> Option<Self> { ");

    match &input.data {
        syn::Data::Struct(data_struct) => {

            match &data_struct.fields {
                syn::Fields::Named(fields_named) => {
                    impl_deserialize.push_str("ctx.deserialize_map(value, |mut ctx| { Some(Self {");

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
                    impl_deserialize.push_str("ctx.deserialize_map(value, |mut ctx| { Some(Self(");

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
                    impl_deserialize.push_str("ctx.deserialize_map(value, |mut ctx| { Some(Self)})");
                },
            }
        },
        syn::Data::Enum(data_enum) => {
            impl_deserialize.push_str("ctx.deserialize_enum()");

            for variant in &data_enum.variants {
                let variant_name = variant.ident.to_string();

                write!(
                    impl_deserialize,
                    r#".variant("{variant_name}", |mut ctx, value| {{"#
                )
                .unwrap();
           
                match &variant.fields {
                    syn::Fields::Named(fields_named) => {
                        impl_deserialize.push_str("");

                        write!(
                            impl_deserialize,
                            r#"ctx.deserialize_map(value, |mut ctx| {{ Some(Self::{variant_name} {{"#
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
                            r#"ctx.deserialize_map(value, |mut ctx| {{ Some(Self::{variant_name} ("#
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
                            r#"ctx.deserialize_map(value, |mut ctx| {{ Some(Self::{variant_name}) }})"#
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
