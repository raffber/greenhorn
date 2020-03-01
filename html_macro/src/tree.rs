use syn::parse::{Parse as SynParse, ParseStream};
use syn::parse::Result as SynResult;
use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::buffer::Cursor;
use proc_macro_error::*;

use crate::matches::{Matches, MatchTwo, MatchSequence};
use proc_macro2::Delimiter;
use proc_macro2::{Literal, Span};

pub struct HtmlName(String);


struct SmallerSign;

impl Matches for SmallerSign {
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

impl Matches for Dash {
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

impl Matches for HtmlNamePart {
    type Output = String;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (part, cursor) = cursor.ident()?;
        Some((part.to_string(), cursor))
    }
}


impl Matches for HtmlName {
    type Output = String;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (part, cursor) = cursor.ident()?;
        type Rest = MatchSequence<MatchTwo<Dash, HtmlNamePart>>;
        if let Some((rest, cursor)) = Rest::matches(cursor) {
            println!("HtmlName::matches - dashed string");
            let mut rest: Vec<((), String)> = rest;
            let strings: Vec<String> = rest.drain(..).map(|x| x.1.to_ascii_lowercase()).collect();
            let ret = strings.join("-");
            if !ret.is_ascii() {
                return None;
            }
            Some((ret,cursor))
        } else {
            println!("HtmlName::matches - simple string");
            let part = part.to_string();
            if !part.is_ascii() {
                return None;
            }
            Some((part,cursor))
        }
    }
}


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


pub(crate) enum ElementAttribute {
    HtmlAttribute(HtmlAttribute),
    ClassAttribute(ClassAttribute),
}

impl Matches for ElementAttribute {
    type Output = ElementAttribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        if let Some((attr, cursor)) = HtmlAttribute::matches(cursor) {
            Some((ElementAttribute::HtmlAttribute(attr), cursor))
        } else if let Some((attr, cursor)) = ClassAttribute::matches(cursor) {
            Some((ElementAttribute::ClassAttribute(attr), cursor))
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


pub(crate) struct Element {
    tag: String,
    attributes: Vec<ElementAttribute>,
    children: Vec<Element>,
}


impl Element {
    fn parse_children<'a>(tag_name: &str, cursor: Cursor<'a>) -> (Vec<Element>, Cursor<'a>) {
        let mut children = Vec::<Element>::new();
        let start_cursor = cursor;
        loop {
            let cursor = if let Some(cursor) = ClosingTag::matches(tag_name, cursor) {
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
                let grp = Group {
                    stream: cursor.token_stream(),
                    span: grp
                };
                cursor
            } else if cursor.eof() {
                panic!("No closing tag");
            } else {
                panic!("Unexpected string");
            };
        }
        unreachable!()
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
                // this was only a start tag, parse children and end tag....
                let ret = Element::parse_children(&elem_start.tag, cursor);
                println!("Elemenet::parse - done parsing children");
                ret
            },
            _ => panic!("Expected one of `>` or `/>`")
        };

        println!("Elemenet::parse - done");
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