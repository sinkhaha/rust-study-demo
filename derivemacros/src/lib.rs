use proc_macro::TokenStream;

#[proc_macro_derive(RawBuilder)]
pub fn derive_raw_builder(input: TokenStream) -> TokenStream {
    println!("{:#?}", input);
    TokenStream::default()
}
