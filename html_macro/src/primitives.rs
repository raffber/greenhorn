use crate::matches::Matches;
use syn::buffer::Cursor;
use crate::matches::{MatchSequence, MatchTwo};
use syn::{Result, Error};


macro_rules! make_punct {
    ( $name:ident, $sign:expr) => {
        pub(crate) struct $name;

        impl Matches for $name {
            type Output = ();

            fn matches(cursor: Cursor) -> Result<(Self::Output, Cursor)> {
                let err = || {
                    let msg = format!("Expected `{}`.", stringify!($sign));
                    Err(Error::new(cursor.span(), msg))
                };
                if let Some((punct, cursor)) = cursor.punct() {
                    if punct.as_char() == $sign {
                        Ok(((), cursor))
                    } else {
                        return err()
                    }
                } else {
                    return err()
                }
            }
        }
    };
}

make_punct!(Slash, '/');
make_punct!(Hash, '#');
make_punct!(SmallerSign, '<');
make_punct!(BiggerSign, '>');
make_punct!(AtSign, '@');
make_punct!(DollarSign, '$');
make_punct!(Dash, '-');
make_punct!(Equal, '=');
make_punct!(Dot, '.');


struct HtmlNamePart;

impl Matches for HtmlNamePart {
    type Output = String;

    fn matches(cursor: Cursor) -> Result<(Self::Output, Cursor)> {
        if let Some((part, cursor)) = cursor.ident() {
            Ok((part.to_string(), cursor))
        } else {
            Err(Error::new(cursor.span(), format!("Expected a identfier.")))
        }
    }
}

pub struct HtmlName(String);

impl Matches for HtmlName {
    type Output = String;

    fn matches(cursor: Cursor) -> Result<(Self::Output, Cursor)> {
        let (part, cursor) = if let Some((part, cursor)) = cursor.ident() {
            (part, cursor)
        } else {
            return Err(Error::new(cursor.span(), "Expected an identifier"));
        };
        type Rest = MatchSequence<MatchTwo<Dash, HtmlNamePart>>;
        if let Ok((rest, cursor)) = Rest::matches(cursor) {
            let mut rest: Vec<((), String)> = rest;
            let mut strings: Vec<String> = rest.drain(..).map(|x| x.1.to_ascii_lowercase()).collect();
            strings.insert(0, part.to_string());
            let ret = strings.join("-");
            if !ret.is_ascii() {
                return Err(Error::new(cursor.span(), format!("Expected an ascii string but got {}", ret)));
            }
            Ok((ret,cursor))
        } else {
            let part = part.to_string();
            if !part.is_ascii() {
                return Err(Error::new(cursor.span(), format!("Expected an ascii string but got {}", part)));
            }
            Ok((part,cursor))
        }
    }
}
