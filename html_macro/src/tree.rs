use syn::parse::{Parse, ParseStream};
use syn::parse::Result as SynResult;
use quote::ToTokens;
use proc_macro2::TokenStream;

pub struct Tree {

}

impl Parse for Tree {
    fn parse(input: ParseStream) -> SynResult<Self> {
        unimplemented!()
    }
}

impl ToTokens for Tree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        unimplemented!()
    }
}