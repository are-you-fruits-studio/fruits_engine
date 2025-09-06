use proc_macro::TokenStream;
use quote::ToTokens;
use std::fmt::Write;
use syn::DeriveInput;


#[proc_macro_derive(ReflectTy)]
pub fn derive_reflect_ty(stream: TokenStream) -> TokenStream {
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

    write!(result, r#"impl{impl_generics} ReflectTy for {type_name}<{type_generics}> {{ "#).unwrap();
    write!(result, r#"fn refl_ty() -> ReflTy {{ "#).unwrap();

    match input.data {
        syn::Data::Struct(data_struct) => {
            result.push_str("ReflTy::Struct(ReflTyStruct::Named(vec![");
            
            for field in data_struct.fields {
                let field_name = field.ident.to_token_stream().to_string();
                let field_type = field.ty.to_token_stream().to_string();

                write!(result, r#"(String::from("{field_name}"), ReflTyId::of::<{field_type}>()),"#).unwrap();
            }

            result.push_str(" ]))");
        },
        syn::Data::Enum(data_enum) => {
            result.push_str("ReflTy::Enum(ReflTyEnum { variants: vec![ ");

            for variant in data_enum.variants {
                let variant_name = variant.ident.to_string();

                write!(result, r#"(String::from("{variant_name}"), "#).unwrap();

                match variant.fields {
                    syn::Fields::Named(fields_named) => {
                        write!(result, r#"ReflTyStruct::Named(vec!["#).unwrap();

                        for field in fields_named.named {
                            let field_type = field.ty.to_token_stream().to_string();

                            write!(result, r#"ReflTyId::of::<{field_type}>(),"#).unwrap();
                        }
                        
                        write!(result, r#"])"#).unwrap();
                    },
                    syn::Fields::Unnamed(fields_unnamed) => {
                        write!(result, r#"ReflTyStruct::Tuple(vec!["#).unwrap();

                        for field in fields_unnamed.unnamed {
                            let field_name = field.ident.to_token_stream().to_string();
                            let field_type = field.ty.to_token_stream().to_string();

                            write!(result, r#"(String::from("{field_name}"), ReflTyId::of::<{field_type}>()),"#).unwrap();
                        }

                        write!(result, r#"])"#).unwrap();
                    },
                    syn::Fields::Unit => write!(result, r#"ReflTyStruct::Unit"#).unwrap(),
                }
                
                write!(result, r#" ),"#).unwrap();
            }
        },
        syn::Data::Union(data_union) => panic!("Union types are not supported."),
    }

    result.push_str(" } ");
    result.push_str(" } ");

    result.parse().unwrap()
}