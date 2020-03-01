use syn::parse::{Parse as SynParse, ParseStream};
use syn::parse::Result as SynResult;
use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::buffer::Cursor;
use proc_macro_error::*;

use crate::matches::{Matches, MatchSequence};
use proc_macro2::Delimiter;
use proc_macro2::{Literal, Span};

use crate::primitives::{HtmlName, SmallerSign, Hash};

pub(crate) struct ElementStart {
    tag: String,
}

impl Matches for ElementStart {
    type Output = ElementStart;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (_, cursor) = SmallerSign::matches(cursor)?;
        let (name, cursor) = HtmlName::matches(cursor)?;
        let ret = ElementStart {
            tag: name
        };
        Some((ret, cursor))
    }
}

pub(crate) struct HtmlAttribute {
    key: String,
    value: AttributeValue,
}


pub struct Group {
    stream: TokenStream,
    span: Span,
}

pub enum AttributeValue {
    Literal(Literal),
    HtmlName(String),
    Group(Group),
}

impl Matches for HtmlAttribute {
    type Output = HtmlAttribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        println!("HtmlAttribute::matches - start");
        let (name, cursor) = HtmlName::matches(cursor)?;
        let (punct, cursor) = cursor.punct()?;
        if punct.as_char() != '=' {
            return None;
        }
        let (value, cursor) = if let Some((literal, cursor)) = cursor.literal() {
            println!("HtmlAttribute::matches - literal");
            Some( (AttributeValue::Literal(literal), cursor) )
        } else if let Some((value, cursor)) = HtmlName::matches( cursor) {
            println!("HtmlAttribute::matches - html-name");
            Some( (AttributeValue::HtmlName(value), cursor) )
        } else if let Some((grp_cursor, grp, cursor)) = cursor.group(Delimiter::Brace) {
            println!("HtmlAttribute::matches - group");
            Some( (AttributeValue::Group(Group {
                stream: grp_cursor.token_stream(),
                span: grp,
            }), cursor) )
        } else {
            None
        }?;
        let ret = HtmlAttribute {
            key: name,
            value
        };
        println!("HtmlAttribute::matches - done");
        Some((ret, cursor))
    }
}

pub(crate) struct ClassAttribute {
    value: String,
}

impl Matches for ClassAttribute {
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

pub(crate) struct IdAttribute {
    value: String,
}


impl Matches for IdAttribute {
    type Output = IdAttribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (_, cursor) = Hash::matches(cursor)?;
        let (name, cursor) = HtmlName::matches(cursor)?;
        Some((IdAttribute {
            value: name.to_string()
        }, cursor))
    }
}

pub(crate) enum ElementAttribute {
    Html(HtmlAttribute),
    Class(ClassAttribute),
    Id(IdAttribute)
}

impl Matches for ElementAttribute {
    type Output = ElementAttribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        if let Some((attr, cursor)) = HtmlAttribute::matches(cursor) {
            Some((ElementAttribute::Html(attr), cursor))
        } else if let Some((attr, cursor)) = ClassAttribute::matches(cursor) {
            Some((ElementAttribute::Class(attr), cursor))
        } else if let Some((attr, cursor)) = IdAttribute::matches(cursor) {
            Some((ElementAttribute::Id(attr), cursor))
        } else {
            None
        }
    }
}


pub(crate) struct ClosingTag {

}

impl ClosingTag {
    pub(crate) fn matches<'a>(tag_name: &str, cursor: Cursor<'a>) -> Option<Cursor<'a>> {
        // allow </> and </tag_name>

        // match starting < which is in common
        let (punct, cursor) = cursor.punct()?;
        if punct.as_char() != '<' {
            return None;
        }

        println!("ClosingTag::matches - <");

        let (punct, cursor) = cursor.punct()?;
        if punct.as_char() != '/' {
            return None;
        }

        println!("ClosingTag::matches - /");

        let cursor = if let Some((name, cursor)) = HtmlName::matches(cursor) {
            println!("ClosingTag::matches - name matched");
            if &name != tag_name {
                return None;
            }
            cursor
        } else {
            cursor
        };

        println!("ClosingTag::matches - name");

        let (punct, cursor) = cursor.punct()?;

        println!("ClosingTag::matches - done");

        if punct.as_char() != '>' {
            return None;
        }
        return Some(cursor);
    }
}


pub(crate) enum Element {
    Html(HtmlElement),
    Expr(ElementExpression),
}

pub(crate) struct HtmlElement {
    tag: String,
    attributes: Vec<ElementAttribute>,
    children: Vec<Element>,
}

pub(crate) struct ElementExpression {
    tokens: TokenStream,
    span: Span,
}


impl Element {
    fn parse_children<'a>(tag_name: &str, cursor: Cursor<'a>) -> (Vec<Element>, Cursor<'a>) {
        let mut children = Vec::<Element>::new();
        let start_cursor = cursor;
        let mut cursor = cursor;
        loop {
            cursor = if let Some(cursor) = ClosingTag::matches(tag_name, cursor) {
                return (children, cursor);
            } else if let Some((punct, cursor)) = cursor.punct() {
                if punct.as_char() == '<' {
                    let ts = start_cursor.token_stream();
                    let elem: Element = syn::parse2(ts).unwrap();
                    children.push(elem);
                } else {
                    panic!("Expected tag or expression");
                }
                cursor
            } else if let Some((grp_cursor, grp, cursor)) = cursor.group(Delimiter::Bracket) {
                let expr = ElementExpression {
                    tokens: grp_cursor.token_stream(),
                    span: grp
                };
                children.push(Element::Expr(expr));
                cursor
            } else if cursor.eof() {
                panic!("No closing tag");
            } else {
                panic!("Unexpected string");
            };
        }
    }
}

impl SynParse for Element {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let cursor = input.cursor();

        println!("Elemenet::parse - start");

        // match opening tag of the form <some-name
        let (elem_start, cursor) = ElementStart::matches(cursor).expect("Expected an opening tag");

        println!("Elemenet::parse - element started");

        // parse all element attributes
        let (attribtues, cursor) =
            MatchSequence::<ElementAttribute>::matches(cursor)
                .unwrap_or_else(|| (Vec::new(), cursor));

        println!("Elemenet::parse - attributes matched");

        // now expect a ">" or a "/>",
        // in case there was only a ">", we continue parsing children
        let (punct, cursor) = cursor.punct().expect("Expected one of `>` or `/>`");
        let punct = punct.as_char();
        let (children, _cursor) = match punct {
            '/' => {
                // element is already done, expect a `>` and return no children
                let cursor = if let Some((punct, cursor)) = cursor.punct() {
                    let punct = punct.as_char();
                    if punct != '>' {
                        panic!("Expected > after /");
                    }
                    cursor
                } else {
                    cursor
                };
                (Vec::new(), cursor)
            },
            '>' => {
                // this was only a start tag, parse children and end tag....
                let ret = Element::parse_children(&elem_start.tag, cursor);
                println!("Elemenet::parse - done parsing children");
                ret
            },
            _ => panic!("Expected one of `>` or `/>`")
        };

        println!("Elemenet::parse - done");
        Ok(Element::Html(HtmlElement {
            tag: elem_start.tag,
            attributes: attribtues,
            children
        }))
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        unimplemented!()
    }
}
