mod tree;

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use syn::parse_macro_input;
use quote::quote;
use tree::Tree;

#[proc_macro_hack]
pub fn html(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as Tree);
    TokenStream::from(quote! {#root})
}