use crate::{
    instruction::Instruction,
    token::{Identifier, Statement, Token},
};
use std::{
    fmt,
    ops::{Add, AddAssign, Range},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Location {
    pub(crate) offset: usize,
}

impl Add<usize> for Location {
    type Output = Location;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            offset: self.offset + rhs,
        }
    }
}

impl AddAssign<usize> for Location {
    fn add_assign(&mut self, rhs: usize) {
        self.offset += rhs;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Span {
    pub(crate) from: Location,
    pub(crate) to: Location,
}

impl Span {
    pub(crate) fn new(location: Location) -> Self {
        Self {
            from: location,
            to: location + 1,
        }
    }

    pub(crate) fn combine(start: &Span, end: &Span) -> Self {
        Self {
            from: start.from,
            to: end.to,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        Self {
            start: span.from.offset,
            end: span.to.offset,
        }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}..{}", self.from.offset, self.to.offset))
    }
}

#[derive(PartialEq, Eq, Clone)]
pub(crate) struct Spanned<T> {
    pub(crate) node: T,
    pub(crate) span: Span,
}

impl<T> Spanned<T> {
    #[inline]
    pub(crate) fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}

impl<T> fmt::Debug for Spanned<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?} (spanned at {:?})", self.node, self.span))
    }
}

pub(crate) trait Spanning
where
    Self: Sized,
{
    fn spanning(self, span: Span) -> Spanned<Self>;
}

macro_rules! impl_spanning {
    ($impl_type:ty) => {
        impl<'a> Spanning for $impl_type {
            fn spanning(self, span: Span) -> Spanned<$impl_type> {
                Spanned { node: self, span }
            }
        }
    };
}

impl_spanning!(String);
impl_spanning!(u64);
impl_spanning!(usize);
impl_spanning!(&'a str);
impl_spanning!(Vec<u8>);
impl_spanning!(char);
impl_spanning!(Instruction);
impl_spanning!(Token);
impl_spanning!(Identifier);
impl_spanning!(Statement);
