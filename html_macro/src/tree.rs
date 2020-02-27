use syn::parse::{Parse as SynParse, ParseStream};
use syn::parse::Result as SynResult;
use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::buffer::Cursor;
use proc_macro_error::*;
use syn::Expr;

use crate::matches::{Match, MatchTwo, MatchSequence};
use proc_macro2::Delimiter;
use proc_macro2::{Literal, Span};

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

pub(crate) struct HtmlAttribute {
    key: String,
    value: AttributeValue,
}


pub struct Group {
    // cursor: Cursor<'a>,
    span: Span,
}

pub enum AttributeValue {
    Literal(Literal),
    HtmlName(String),
    Group(Group),
}

impl Match for HtmlAttribute {
    type Output = HtmlAttribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (name, cursor) = HtmlName::matches(cursor)?;
        let (punct, cursor) = cursor.punct()?;
        if punct.as_char() != '=' {
            return None;
        }
        let value = if let Some((literal, cursor)) = cursor.literal() {
            Some(AttributeValue::Literal(literal))
        } else if let Some((value, cursor)) = HtmlName::matches( cursor) {
            Some(AttributeValue::HtmlName(value))
        } else if let Some((grp_cursor, grp, cursor)) = cursor.group(Delimiter::Bracket) {
            Some(AttributeValue::Group(Group {
                span: grp,
            }))
        } else {
            None
        }?;
        let ret = HtmlAttribute {
            key: name,
            value
        };
        Some((ret, cursor))
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
    tag: String,
    attributes: Vec<ElementAttribute>,
    children: Vec<Element>,
}


impl Element {

    fn parse_children(tag_name: &str) -> (Vec<Element>, Cursor) {
        todo!()
    }

}

impl SynParse for Element {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let cursor = input.cursor();

        // match opening tag of the form <some-name
        let (elem_start, cursor) = ElementStart::matches(cursor).expect("Expected an opening tag");

        // parse all element attributes
        let (attribtues, cursor) =
            MatchSequence::<ElementAttribute>::matches(cursor)
                .unwrap_or_else(|| (Vec::new(), cursor));

        // now expect a ">" or a "/>",
        // in case there was only a ">", we continue parsing children
        let (punct, cursor) = cursor.punct().expect("Expected one of `>` or `/>`");
        let punct = punct.as_char();
        let (children, cursor) = match punct {
            '/' => {
                // element is already done, expect a `>` and return no children
                if let Some((punct, cursor)) = cursor.punct() {
                    let punct = punct.as_char();
                    if punct != '>' {
                        panic!("Expected > after /");
                    }
                }
                (Vec::new(), cursor)
            },
            '>' => {
                // this was only a start tag, parse children....
                Element::parse_children(&elem_start.tag)
            },
            _ => panic!("Expected one of `>` or `/>`")
        };

        Ok(Element {
            tag: elem_start.tag,
            attributes: attribtues,
            children
        })
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        unimplemented!()
    }
}