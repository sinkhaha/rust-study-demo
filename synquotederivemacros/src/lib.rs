mod builder;
mod builder_with_attr;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};


// 创建Builder宏
#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    println!("{:#?}", input);
    // TokenStream::default()
    builder::BuilderContext::from(input).render().into()
}

// 创建BuilderWithAttr宏
#[proc_macro_derive(BuilderWithAttr, attributes(builder))]
pub fn derive_builder_with_attr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    builder_with_attr::BuilderContext::from(input)
        .render()
        .into()
}