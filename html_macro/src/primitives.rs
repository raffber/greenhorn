use crate::matches::Matches;
use syn::buffer::Cursor;
use crate::matches::{MatchSequence, MatchTwo};

pub(crate) struct SmallerSign;

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


pub(crate) struct Hash;

impl Matches for Hash {
    type Output = ();

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (punct, cursor) = cursor.punct()?;
        if punct.as_char() == '#' {
            Some(((), cursor))
        } else {
            None
        }
    }
}


pub(crate) struct AtSign;

impl Matches for AtSign {
    type Output = ();

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (punct, cursor) = cursor.punct()?;
        if punct.as_char() == '@' {
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

pub struct HtmlName(String);

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
