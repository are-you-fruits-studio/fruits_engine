use std::fmt::Debug;

use proc_macro::{Literal, TokenStream, TokenTree};

#[proc_macro]
pub fn rgba_u8_array(stream: TokenStream) -> TokenStream {
    to_token_stream(fruits_math::parse_color_rgba_u8(&to_string_literal(stream)).unwrap_or_else(|| panic_invalid_format()))
}

#[proc_macro]
pub fn rgb_u8_array(stream: TokenStream) -> TokenStream {
    to_token_stream(fruits_math::parse_color_rgb_u8(&to_string_literal(stream)).unwrap_or_else(|| panic_invalid_format()))
}

#[proc_macro]
pub fn rgba_f32_array(stream: TokenStream) -> TokenStream {
    to_token_stream(fruits_math::parse_color_rgba_f32(&to_string_literal(stream)).unwrap_or_else(|| panic_invalid_format()))
}

#[proc_macro]
pub fn rgb_f32_array(stream: TokenStream) -> TokenStream {
    to_token_stream(fruits_math::parse_color_rgb_f32(&to_string_literal(stream)).unwrap_or_else(|| panic_invalid_format()))
}

fn to_string_literal(stream: TokenStream) -> String {
    let mut iter = stream.into_iter();

    let literal = iter.next().unwrap_or_else(|| panic_invalid_format());

    let TokenTree::Literal(literal) = literal else {
        panic_invalid_format();
    };

    to_exact_string_literal(literal)
}

fn to_token_stream<T: Debug, const N: usize>(v: [T; N]) -> TokenStream {
    let mut result = String::new();

    result.push_str("[");
    result.push_str(&v.into_iter().map(|x| format!("{:?}", x)).collect::<Vec<_>>().join(", "));
    result.push_str("]");

    result.parse().unwrap_or_else(|_| panic_invalid_format())
}

fn to_exact_string_literal(lit: Literal) -> String {
    let lit = lit.to_string();

    let mut chars = lit.chars();

    let Some('"') = chars.next() else {
        panic_invalid_format();
    };

    let mut result = String::new();

    let mut chars = chars.peekable();

    if chars.peek().is_none() {
        panic_invalid_format();
    }

    loop {
        let c = chars.next().unwrap();

        if chars.peek().is_some() {
            result.push(c);
            continue;
        }

        if c != '"' {
            panic_invalid_format();
        }

        break;
    }

    result
}

fn panic_invalid_format() -> ! {
    panic!("macro expects a single string literal of format \"#ffffffff\" or \"#ffffff\"");
}
