use crate::Span;

#[derive(Clone)]
pub enum Warning {
    TokenTrimmed { span: Span },
}

#[derive(Clone)]
pub enum Error {
    NoMatchingClosingParenthesis { span: Span },
    NoMatchingOpeningParenthesis { span: Span },
}
