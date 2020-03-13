use std::marker::PhantomData;
use syn::buffer::Cursor;
use syn::Result;


pub(crate) trait Matches {
    type Output;

    fn matches(cursor: Cursor) -> Result<(Self::Output, Cursor)>;
}

pub(crate)struct MatchTwo<A: Matches, B: Matches> {
    a: PhantomData<A>,
    b: PhantomData<B>,
}

impl<A: Matches, B: Matches> Matches for MatchTwo<A, B> {
    type Output = (A::Output, B::Output);

    fn matches(cursor: Cursor) -> Result<(Self::Output, Cursor)> {
        let (out_a, cursor) = A::matches(cursor)?;
        let (out_b, cursor) = B::matches(cursor)?;
        Ok(((out_a, out_b), cursor))
    }
}

pub(crate) struct MatchSequence<T: Matches> {
    t: PhantomData<T>
}

impl<T: Matches> Matches for MatchSequence<T> {
    type Output = Vec<T::Output>;

    fn matches(cursor: Cursor) -> Result<(Self::Output, Cursor)> {
        let mut c = cursor;
        let mut ret = Vec::new();
        loop {
            match T::matches(c) {
                Ok((out, cursor)) => {
                    ret.push(out);
                    c = cursor;
                },
                Err(_) => {
                    break
                },
            }
        }
        Ok((ret,c))
    }
}
