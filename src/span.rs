use std::{
    fmt,
    ops::{Add, AddAssign},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Location {
    pub offset: usize,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    pub from: Location,
    pub to: Location,
}

impl Span {
    pub fn new(location: Location) -> Self {
        Self {
            from: location,
            to: location + 1,
        }
    }

    pub fn combine(start: &Span, end: &Span) -> Self {
        Self {
            from: start.from,
            to: end.to,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}

impl<T> fmt::Display for Spanned<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.node)
    }
}

impl<T> std::error::Error for Spanned<T> where T: std::error::Error {}

pub trait Spanning
where
    Self: Sized,
{
    fn spanning(self, span: Span) -> Spanned<Self>;
}

#[macro_export]
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
