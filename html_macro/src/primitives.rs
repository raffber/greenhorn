use crate::matches::Matches;
use syn::buffer::Cursor;
use crate::matches::{MatchSequence, MatchTwo};


macro_rules! make_punct {
    ( $name:ident, $sign:expr) => {
        pub(crate) struct $name;

        impl Matches for $name {
            type Output = ();

            fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
                let (punct, cursor) = cursor.punct()?;
                if punct.as_char() == $sign {
                    Some(((), cursor))
                } else {
                    None
                }
            }
        }
    };
}

make_punct!(Hash, '#');
make_punct!(SmallerSign, '<');
make_punct!(AtSign, '@');
make_punct!(DollarSign, '$');
make_punct!(Dash, '-');
make_punct!(Equal, '=');
make_punct!(Dot, '.');


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
            let mut strings: Vec<String> = rest.drain(..).map(|x| x.1.to_ascii_lowercase()).collect();
            strings.insert(0, part.to_string());
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
