use syn::parse::{Parse as SynParse, ParseStream};
use syn::parse::Result as SynResult;
use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::buffer::Cursor;
use proc_macro_error::*;

use crate::matches::{Match, MatchTwo, MatchSequence};

pub struct HtmlName(String);


struct SmallerSign;

impl Match for SmallerSign {
    type Output = ();

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (punct, cursor) = cursor.punct()?;
        if punct.as_char() == '<' {
            Some(((), cursor))
        } else {
            None
        }
    }
}

struct Dash;

impl Match for Dash {
    type Output = ();

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (punct, cursor) = cursor.punct()?;
        if punct.as_char() == '-' {
            Some(((), cursor))
        } else {
            None
        }
    }
}

struct HtmlNamePart;

impl Match for HtmlNamePart {
    type Output = String;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (part, cursor) = cursor.ident()?;
        // TODO: check for valid html-identifier
        Some((part.to_string(), cursor))
    }
}


impl Match for HtmlName {
    type Output = String;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (part, cursor) = cursor.ident()?;
        type Rest = MatchSequence<MatchTwo<HtmlNamePart, Dash>>;
        let (rest, cursor) = Rest::matches(cursor)?;
        let mut rest: Vec<(String, ())> = rest;
        let strings: Vec<String> = rest.drain(..).map(|x| x.0).collect();
        let ret = strings.join("-");
        Some((ret,cursor))
    }
}


pub(crate) struct Element {
    tag: String,
}

impl Match for Element {
    type Output = Element;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (_, cursor) = SmallerSign::matches(cursor)?;
        let (name, cusor) = HtmlName::matches(cursor)?;
        let ret = Element {
            tag: name
        };
        Some((ret, cusor))
    }
}

impl SynParse for Element {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let cursor = input.cursor();
        unimplemented!()
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        unimplemented!()
    }
}