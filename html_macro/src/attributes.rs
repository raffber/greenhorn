use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::buffer::Cursor;

use crate::matches::Matches;
use proc_macro2::Delimiter;
use proc_macro2::{Literal, Span};
use quote::quote;

use crate::primitives::{HtmlName, Hash, AtSign, DollarSign, Equal, Dot};

pub(crate) struct HtmlAttribute {
    pub(crate) key: String,
    pub(crate) value: AttributeValue,
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
        let (grp_cursor, _grp, cursor) = cursor.group(Delimiter::Brace)?;
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

