use std::marker::PhantomData;
use syn::buffer::Cursor;


pub(crate) trait Match {
    type Output;

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)>;
}

pub(crate)struct MatchTwo<A: Match, B: Match> {
    a: PhantomData<A>,
    b: PhantomData<B>,
}

impl<A: Match, B: Match> Match for MatchTwo<A, B> {
    type Output = (A::Output, B::Output);

    fn matches(cursor: Cursor) -> Option<(Self::Output, Cursor)> {
        let (out_a, cursor) = A::matches(cursor)?;
        let (out_b, cursor) = B::matches(cursor)?;
        Some(((out_a, out_b), cursor))
    }
}

pub(crate) struct MatchSequence<T: Match> {
    t: PhantomData<T>
}

impl<T: Match> Match for MatchSequence<T> {
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

