use syn::parse::{Parse as SynParse, ParseStream};
use syn::parse::Result as SynResult;
use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::buffer::Cursor;
use proc_macro_error::*;
use syn::Expr;

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


pub(crate) struct ElementStart {
    tag: String,
}

impl Match for ElementStart {
    type Output = ElementStart;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (_, cursor) = SmallerSign::matches(cursor)?;
        let (name, cusor) = HtmlName::matches(cursor)?;
        let ret = ElementStart {
            tag: name
        };
        Some((ret, cusor))
    }
}

pub(crate) enum AttributeValue {
    String(String),
    Expr(Expr)
}

pub(crate) struct HtmlAttribute {
    key: String,
    value: AttributeValue,
}

impl Match for HtmlAttribute {
    type Output = HtmlAttribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        todo!()
    }
}

pub(crate) struct ClassAttribute {
    value: String,
}

impl Match for ClassAttribute {
    type Output = ClassAttribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (punct, cursor) = cursor.punct()?;
        if punct.as_char() != '.' {
            return None;
        }
        let (name, cursor) = HtmlName::matches(cursor)?;
        Some((ClassAttribute {
            value: name.to_string()
        }, cursor))
    }
}


pub(crate) enum ElementAttribute {
    HtmlAttribute(HtmlAttribute),
    ClassAttribute(ClassAttribute),
}

impl Match for ElementAttribute {
    type Output = ElementAttribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        todo!()
    }
}


pub(crate) struct Element {

}

impl SynParse for Element {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let cursor = input.cursor();
        if let Some((elem_start, cursor)) = ElementStart::matches(cursor) {
            type AttributesMatch = MatchSequence<ElementAttribute>;
            if let Some((attributes, cursor)) = AttributesMatch::matches(cursor) {
                let attributes: Vec<ElementAttribute> = attributes;
                todo!()
            }
            todo!()
            // match tag close, either > or />
        } else {
            panic!("HTML block must start with an element")
        }
        todo!()
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        unimplemented!()
    }
}