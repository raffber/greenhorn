use syn::parse::{Parse, ParseStream};
use syn::parse::Result as SynResult;
use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::buffer::Cursor;
use proc_macro_error::*;

use crate::matches::{Matches, MatchSequence, ParseAdapter};
use proc_macro2::Delimiter;
use proc_macro2::{Literal, Span};
use quote::quote;

use crate::primitives::{HtmlName, SmallerSign, Hash, AtSign, DollarSign, Equal, Dot};

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


impl Matches for AttributeValue {
    type Output = Self;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        if let Some((literal, cursor)) = cursor.literal() {
            println!("AttributeValue::matches - literal");
            Some( (AttributeValue::Literal(literal), cursor) )
        } else if let Some((value, cursor)) = HtmlName::matches( cursor) {
            println!("AttributeValue::matches - html-name");
            Some( (AttributeValue::HtmlName(value), cursor) )
        } else if let Some((grp_cursor, grp, cursor)) = cursor.group(Delimiter::Brace) {
            println!("AttributeValue::matches - group");
            Some( (AttributeValue::Group(Group {
                stream: grp_cursor.token_stream(),
                span: grp,
            }), cursor) )
        } else {
            None
        }
    }
}

impl ToTokens for AttributeValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            AttributeValue::Literal(lit) => {
                lit.to_tokens(tokens);
            },
            AttributeValue::HtmlName(name) => {
                let lit = Literal::string(&name);
                lit.to_tokens(tokens);
            },
            AttributeValue::Group(grp) => {
                let stream = grp.stream.clone();
                tokens.extend(stream);
            },
        }
    }
}

impl Matches for HtmlAttribute {
    type Output = HtmlAttribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        println!("HtmlAttribute::matches - start");
        let (name, cursor) = HtmlName::matches(cursor)?;
        let (_, cursor) = Equal::matches(cursor)?;
        let (value, cursor) = AttributeValue::matches(cursor)?;
        let ret = HtmlAttribute {
            key: name,
            value
        };
        println!("HtmlAttribute::matches - done");
        Some((ret, cursor))
    }
}

pub(crate) struct ClassAttribute {
    value: AttributeValue,
}

impl ToTokens for ClassAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = &self.value;
        tokens.extend(quote!{ #value });
    }
}

impl Matches for ClassAttribute {
    type Output = ClassAttribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (_, cursor) = Dot::matches(cursor)?;
        let (name, cursor) = AttributeValue::matches(cursor)?;
        Some((ClassAttribute {
            value: name
        }, cursor))
    }
}

pub(crate) struct IdAttribute {
    value: AttributeValue,
}

impl ToTokens for IdAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = &self.value;
        tokens.extend(quote!{ #value });
    }
}

impl Matches for IdAttribute {
    type Output = IdAttribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (_, cursor) = Hash::matches(cursor)?;
        let (name, cursor) = AttributeValue::matches(cursor)?;
        Some((IdAttribute {
            value: name
        }, cursor))
    }
}

pub(crate) struct ListenerAttribute {
    name: String,
    value: TokenStream,
}


impl Matches for ListenerAttribute  {
    type Output = ListenerAttribute ;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (_, cursor) = AtSign::matches(cursor)?;
        let (name, cursor) = HtmlName::matches(cursor)?;
        let (_, cursor) = Equal::matches(cursor)?;
        let (grp_cursor, _grp, cursor) = cursor.group(Delimiter::Bracket)?;
        Some((ListenerAttribute  { 
            name,
            value: grp_cursor.token_stream(),
        }, cursor))
    }
}

impl ToTokens for ListenerAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name: &str = &self.name;
        let ts = &self.value; 
        let ret = quote! {
            .on(#name, #ts)
        };
        tokens.extend(ret);
    } 
}


pub(crate) struct JsEvent {
    name: String,
    value: String,
}

impl Matches for JsEvent {
    type Output = JsEvent;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (_, cursor) = DollarSign::matches(cursor)?;
        let (name, cursor) = HtmlName::matches(cursor)?;
        let (_, cursor) = Equal::matches(cursor)?;
        let (lit, cursor) = cursor.literal()?;
        let value = lit.to_string();
        for b in value.bytes() {
            if b != b'"' {
                panic!("Invalid non-string literal");
            }
            break;
        }
        let value = value[1..value.len()-1].to_string();

        let ret = JsEvent {
            name,
            value, 
        };
        Some((ret, cursor))
    } 
} 

impl ToTokens for JsEvent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let value = &self.value;
        let ret = quote! {
            .js_event(#name, #value)
        }; 
        tokens.extend(ret);
    } 
}

pub(crate) enum Attribute {
    Html(HtmlAttribute),
    Class(ClassAttribute),
    Id(IdAttribute),
    Listener(ListenerAttribute),
    Js(JsEvent)
}

impl Matches for Attribute {
    type Output = Attribute;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        if let Some((attr, cursor)) = HtmlAttribute::matches(cursor) {
            Some((Attribute::Html(attr), cursor))
        } else if let Some((attr, cursor)) = ClassAttribute::matches(cursor) {
            Some((Attribute::Class(attr), cursor))
        } else if let Some((attr, cursor)) = IdAttribute::matches(cursor) {
            Some((Attribute::Id(attr), cursor))
        } else if let Some((attr, cursor)) = ListenerAttribute::matches(cursor) {
            Some((Attribute::Listener(attr), cursor))
        } else if let Some((evt, cursor)) = JsEvent::matches(cursor) {
            Some((Attribute::Js(evt), cursor))
        } else {
            None
        }
    }
}


pub(crate) struct ClosingTag;

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

impl Matches for Element {
    type Output = Self;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        println!("Elemenet::parse - start");

        // match opening tag of the form <some-name
        let (elem_start, cursor) = ElementStart::matches(cursor)?;

        println!("Elemenet::parse - element started");

        // parse all element attributes
        let (attribtues, cursor) =
            MatchSequence::<Attribute>::matches(cursor)
                .unwrap_or_else(|| (Vec::new(), cursor));

        println!("Elemenet::parse - attributes matched");

        // now expect a ">" or a "/>",
        // in case there was only a ">", we continue parsing children
        let (punct, cursor) = cursor.punct()?;
        let punct = punct.as_char();
        let (children, cursor) = match punct {
            '/' => {
                // element is already done, expect a `>` and return no children
                let cursor = if let Some((punct, cursor)) = cursor.punct() {
                    let punct = punct.as_char();
                    if punct != '>' {
                        return None;
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

        let ret = Element::Html(HtmlElement {
            tag: elem_start.tag,
            attributes: attribtues,
            children,
            namespace: None
        });
        Some((ret, cursor))
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> SynResult<Self> {
        ParseAdapter::<Self>::parse(input).map(|x| x.unwrap())
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = match self {
            Element::Html(html) => {
                quote! {
                    use greenhorn::prelude::NodeBuilder;
                    #html
                }
            },
            Element::Expr(_) => {
                todo!()
            },
        };
        tokens.extend(ts);
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
        tokens.extend(ret);
    }
}

