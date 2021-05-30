use super::{Span, Spanned, Spanning};
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

#[derive(Debug, Copy, Clone)]
pub struct Symbols<'a> {
    inner: &'a [Spanned<char>],
}

impl<'a> Symbols<'a> {
    pub fn first(&self) -> Option<Spanned<char>> {
        self.inner.first().cloned()
    }

    pub fn second(&self) -> Option<Spanned<char>> {
        self.inner.get(1).cloned()
    }

    pub fn last(&self) -> Option<Spanned<char>> {
        self.inner.last().cloned()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(char) -> bool,
    {
        self.inner
            .iter()
            .position(|Spanned { node: ch, .. }| predicate(*ch))
    }

    pub fn to_string(&self) -> String {
        self.inner
            .into_iter()
            .map(|Spanned { node: ch, .. }| *ch)
            .collect()
    }

    pub fn to_span(&self) -> Option<Span> {
        Some(Span::combine(&self.first()?.span, &self.last()?.span))
    }

    pub fn to_spanned_string(&self) -> Option<Spanned<String>> {
        self.to_span().map(|span| self.to_string().spanning(span))
    }
}

impl<'a> From<&'a [Spanned<char>]> for Symbols<'a> {
    fn from(inner: &'a [Spanned<char>]) -> Self {
        Self { inner }
    }
}

impl<'a> IntoIterator for Symbols<'a> {
    type Item = Spanned<char>;
    type IntoIter = SymbolsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SymbolsIter {
            inner: self.inner,
            counter: 0,
        }
    }
}

pub struct SymbolsIter<'a> {
    inner: &'a [Spanned<char>],
    counter: usize,
}

impl<'a> Iterator for SymbolsIter<'a> {
    type Item = Spanned<char>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.inner.get(self.counter).cloned();
        self.counter += 1;
        result
    }
}

pub trait Slice<T> {
    fn slice(self, range: T) -> Self;
}

impl<'a> Slice<Range<usize>> for Symbols<'a> {
    fn slice(self, range: Range<usize>) -> Self {
        Self {
            inner: &self.inner[range],
        }
    }
}

impl<'a> Slice<RangeFrom<usize>> for Symbols<'a> {
    fn slice(self, range: RangeFrom<usize>) -> Self {
        Self {
            inner: &self.inner[range],
        }
    }
}

impl<'a> Slice<RangeTo<usize>> for Symbols<'a> {
    fn slice(self, range: RangeTo<usize>) -> Self {
        Self {
            inner: &self.inner[range],
        }
    }
}

impl<'a> Slice<RangeFull> for Symbols<'a> {
    fn slice(self, range: RangeFull) -> Self {
        Self {
            inner: &self.inner[range],
        }
    }
}
