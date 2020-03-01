use std::marker::PhantomData;
use syn::buffer::Cursor;
use syn::parse::Parse;
use syn::Error;
use syn::parse::ParseBuffer;


pub(crate) trait Matches {
    type Output;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)>;
}

pub(crate)struct MatchTwo<A: Matches, B: Matches> {
    a: PhantomData<A>,
    b: PhantomData<B>,
}

impl<A: Matches, B: Matches> Matches for MatchTwo<A, B> {
    type Output = (A::Output, B::Output);

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (out_a, cursor) = A::matches(cursor)?;
        let (out_b, cursor) = B::matches(cursor)?;
        Some(((out_a, out_b), cursor))
    }
}

pub(crate) struct MatchSequence<T: Matches> {
    t: PhantomData<T>
}

impl<T: Matches> Matches for MatchSequence<T> {
    type Output = Vec<T::Output>;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let mut c = cursor;
        let mut ret = Vec::new();
        while let Some((out, cursor)) = T::matches(c) {
            ret.push(out);
            c = cursor;
        }
        if ret.is_empty() {
            None
        } else {
            Some((ret,c))
        }
    }
}

pub(crate) struct ParseAdapter<T: Matches> {
    value: T::Output
}

impl<T: Matches> ParseAdapter<T> {
    pub(crate) fn unwrap(self) -> T::Output {
        self.value
    }
}

impl<T: Matches> Parse for ParseAdapter<T> {
    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        input.step(|cursor| {
            let cursor = *cursor;
            if let Some((value, cursor)) = T::matches(cursor) {
                let ret = Self {
                    value
                };
                Ok((ret, cursor))
            } else {
                Err(Error::new(cursor.span(), "Whoops"))
            }
        })
    }
}