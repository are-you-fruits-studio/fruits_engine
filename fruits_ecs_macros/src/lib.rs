use proc_macro::TokenStream;

#[proc_macro_derive(Component)]
pub fn derive_component(stream: TokenStream) -> TokenStream {
    let Some(struct_name) = get_struct_name(stream) else {
        panic!("The name of the struct is not found.");
    };

    format!("impl Component for {struct_name} {{ }}").parse().unwrap()
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(stream: TokenStream) -> TokenStream {
    let Some(struct_name) = get_struct_name(stream) else {
        panic!("The name of the struct is not found.");
    };

    format!("impl Resource for {struct_name} {{ }}").parse().unwrap()
}

#[proc_macro_derive(SystemResource)]
pub fn derive_system_resource(stream: TokenStream) -> TokenStream {
    let Some(struct_name) = get_struct_name(stream) else {
        panic!("The name of the struct is not found.");
    };

    format!("impl SystemResource for {struct_name} {{ }}").parse().unwrap()
}

#[proc_macro_derive(Event)]
pub fn derive_event(stream: TokenStream) -> TokenStream {
    let Some(struct_name) = get_struct_name(stream) else {
        panic!("The name of the struct is not found.");
    };

    format!("impl Event for {struct_name} {{ }}").parse().unwrap()
}

fn get_struct_name(stream: TokenStream) -> Option<String> {
    let mut iter = stream.into_iter();

    for tree in &mut iter {
        if let proc_macro::TokenTree::Ident(ident) = tree {
            if ident.to_string() == "struct" {
                break;
            }
            if ident.to_string() == "enum" {
                break;
            }
        }
    }

    let name_tree = iter.next()?;

    let proc_macro::TokenTree::Ident(name_ident) = name_tree else {
        return None;
    };

    Some(name_ident.to_string())
}
