mod raw_builder;

use proc_macro::TokenStream;
use raw_builder::BuilderContext;

// 添加一个名字为RawBuilder的派生宏
#[proc_macro_derive(RawBuilder)]
pub fn derive_raw_builder(input: TokenStream) -> TokenStream {
    println!("input的值是 {:#?}", input);

    BuilderContext::render(input).unwrap().parse().unwrap()
}
