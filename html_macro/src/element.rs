use syn::parse::{Parse, ParseStream};
use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::buffer::Cursor;
use proc_macro_error::*;
use syn::{Error, Result};

use crate::matches::{Matches, MatchSequence};
use proc_macro2::Delimiter;
use proc_macro2::Span;
use quote::quote;
use crate::attributes::Attribute;

use crate::primitives::{HtmlName, SmallerSign, Slash, BiggerSign};

pub(crate) struct ElementStart {
    tag: String,
}

impl Matches for ElementStart {
    type Output = ElementStart;

    fn matches(cursor: Cursor) -> Result<(Self::Output, Cursor)> {
        let (_, cursor) = SmallerSign::matches(cursor)?;
        let (name, cursor) = HtmlName::matches(cursor)?;
        let ret = ElementStart {
            tag: name
        };
        Ok((ret, cursor))
    }
}

pub(crate) struct ClosingTag;

impl ClosingTag {
    pub(crate) fn matches<'a>(tag_name: &str, cursor: Cursor<'a>) -> Result<Cursor<'a>> {
        // allow </> and </tag_name>
        let (_, cursor) = SmallerSign::matches(cursor)?;
        let (_, cursor) = Slash::matches(cursor)?;
        // optionally match for the tag name
        let cursor = if let Ok((name, cursor)) = HtmlName::matches(cursor) {
            if &name != tag_name {
                let msg = format!("Tag name does not match expected value. Got `{}` but expected `{}`", name, tag_name);
                return Err(Error::new(cursor.span(), msg));
            }
            cursor
        } else {
            cursor
        };

        BiggerSign::matches(cursor).map(|x| x.1)
    }
}


pub(crate) enum Element {
    Html(HtmlElement),
    Expr(ElementExpression),
    Text(String),
}

pub(crate) struct HtmlElement {
    tag: String,
    attributes: Vec<Attribute>,
    children: Vec<Element>,
    namespace: Option<String>,
}

pub(crate) struct ElementExpression {
    tokens: TokenStream,
    span: Span,
}


impl Element {
    pub(crate) fn setup_namespace(&mut self, ns: &str) {
        match self {
            Element::Html(ref mut elem) => {
                elem.namespace = Some(ns.to_string());
                for child in &mut elem.children {
                    child.setup_namespace(ns);
                }
            },
            _ => {}
        }

    }

    fn parse_children<'a>(tag_name: &str, cursor: Cursor<'a>) -> Result<(Vec<Element>, Cursor<'a>)> {
        let mut children = Vec::<Element>::new();
        let mut cursor = cursor;
        loop {
            let start_cursor = cursor;
            cursor = if let Ok(cursor) = ClosingTag::matches(tag_name, cursor) {
                return Ok((children, cursor));
            } else if let Ok((_, _)) = SmallerSign::matches(cursor) {
                if let Ok((elem, cursor)) =  Element::matches(start_cursor) {
                    children.push(elem);
                    cursor
                } else {
                    return Err(Error::new(cursor.span(), "Cannot match child element."));
                }
            } else if let Some((grp_cursor, grp, cursor)) = cursor.group(Delimiter::Brace) {
                let expr = ElementExpression {
                    tokens: grp_cursor.token_stream(),
                    span: grp
                };
                children.push(Element::Expr(expr));
                cursor
            } else if let Some((ident, cursor)) = cursor.ident() {
                let txt = ident.to_string();
                children.push(Element::Text(txt));
                cursor
            } else if cursor.eof() {
                return Err(Error::new(cursor.span(), "No closing tag"));
            } else {
                return Err(Error::new(cursor.span(), "Unexpected string"));
            };
        }
    }
}

impl Matches for Element {
    type Output = Self;

    fn matches(cursor: Cursor) -> Result<(Self::Output, Cursor)> {
        println!("Elemenet::parse - start");

        // match opening tag of the form <some-name
        let (elem_start, cursor) = ElementStart::matches(cursor)?;

        println!("Elemenet::parse - element started");

        // parse all element attributes
        let (attribtues, cursor) = MatchSequence::<Attribute>::matches(cursor)?;

        println!("Elemenet::parse - attributes matched");

        // now expect a ">" or a "/>",
        // in case there was only a ">", we continue parsing children
        let (punct, cursor) = if let Some((punct, cursor)) = cursor.punct() {
            (punct, cursor)
        } else {
            return Err(Error::new(cursor.span(), "Expected one of `>` or `/>`"))
        };
        let punct = punct.as_char();
        let (children, cursor) = match punct {
            '/' => {
                // element is already done, expect a `>` and return no children
                let cursor = if let Some((punct, cursor)) = cursor.punct() {
                    let punct = punct.as_char();
                    if punct != '>' {
                        return Err(Error::new(cursor.span(), "Expected a `>`"));
                    }
                    cursor
                } else {
                    cursor
                };
                (Vec::new(), cursor)
            },
            '>' => {
                println!("Elemenet::parse - start parsing children");
                // this was only a start tag, parse children and end tag....
                Element::parse_children(&elem_start.tag, cursor)?
            },
            _ => {
                return Err(Error::new(cursor.span(), "Expected one of `>` or `/>`"))
            }
        };

        println!("Elemenet::parse - done");

        let ret = Element::Html(HtmlElement {
            tag: elem_start.tag,
            attributes: attribtues,
            children,
            namespace: None
        });
        Ok((ret, cursor))
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|cursor| {
            let cursor = *cursor;
            Self::matches(cursor)
        })
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Element::Html(html) => {
                html.to_tokens(tokens);
            },
            Element::Expr(expr) => {
                tokens.extend(expr.tokens.clone());
            },
            Element::Text(txt) => {
                tokens.extend(quote! {
                     NodeBuilder::new().text(#txt)
                })
            }
        }
    }
}

impl ToTokens for HtmlElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tag_name = &self.tag;
        let mut ret = if let Some(ns) = &self.namespace {
            quote! {
                NodeBuilder::new_with_ns(#ns).elem(#tag_name)
            }
        } else {
            quote! {
                NodeBuilder::new().elem(#tag_name)
            }
        };
        for attr in &self.attributes {
            match attr {
                Attribute::Html(attr) => {
                    let name = &attr.key;
                    let value = &attr.value;
                    ret.extend(quote! {
                        .attr(#name, #value)
                    })
                },
                Attribute::Class(attr) => {
                    ret.extend(quote! {
                        .class(#attr)
                    })
                },
                Attribute::Id(attr) => {
                    ret.extend(quote! {
                        .id(#attr)
                    })
                },
                Attribute::Listener(attr) => {
                    ret.extend(attr.to_token_stream());
                },
                Attribute::Js(evt) => {
                    ret.extend(evt.to_token_stream());
                }
            }
        }
        for child in &self.children {
            ret.extend(quote! {
                .add(#child)
            });
        }
        tokens.extend(ret);
    }
}

