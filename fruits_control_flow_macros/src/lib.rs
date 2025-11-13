use proc_macro::TokenStream;

#[proc_macro]
pub fn return_if(stream: TokenStream) -> TokenStream {
    let mut result = String::new();

    result.push_str("if ");
    result.push_str(&stream.to_string());
    result.push_str(" { return; }");

    result.parse().unwrap()
}

#[proc_macro]
pub fn continue_if(stream: TokenStream) -> TokenStream {
    let mut result = String::new();

    result.push_str("if ");
    result.push_str(&stream.to_string());
    result.push_str(" { continue; }");

    result.parse().unwrap()
}

#[proc_macro]
pub fn break_if(stream: TokenStream) -> TokenStream {
    let mut result = String::new();

    result.push_str("if ");
    result.push_str(&stream.to_string());
    result.push_str(" { break; }");

    result.parse().unwrap()
}

//

#[proc_macro]
pub fn return_if_not(stream: TokenStream) -> TokenStream {
    let mut result = String::new();

    result.push_str("let ");
    result.push_str(&stream.to_string());
    result.push_str(" else { return; };");

    result.parse().unwrap()
}

#[proc_macro]
pub fn continue_if_not(stream: TokenStream) -> TokenStream {
    let mut result = String::new();

    result.push_str("let ");
    result.push_str(&stream.to_string());
    result.push_str(" else { continue; };");

    result.parse().unwrap()
}

#[proc_macro]
pub fn break_if_not(stream: TokenStream) -> TokenStream {
    let mut result = String::new();

    result.push_str("let ");
    result.push_str(&stream.to_string());
    result.push_str(" else { break; };");

    result.parse().unwrap()
}
