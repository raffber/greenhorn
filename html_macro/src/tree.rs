use syn::parse::{Parse, ParseStream};
use syn::parse::Result as SynResult;
use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::buffer::Cursor;
use proc_macro_error::*;


trait Match {
    fn matches(cursor: Cursor) -> bool;
}

pub struct Element {

}

impl Match for Element {
    fn matches(cursor: Cursor) -> bool {
        unimplemented!()
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> SynResult<Self> {
        unimplemented!()
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        unimplemented!()
    }
}