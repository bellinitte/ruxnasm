use crate::Span;

#[derive(Debug, Clone)]
pub enum Warning {
    TokenTrimmed { span: Span },
}

#[derive(Debug, Clone)]
pub enum Error {
    NoMatchingClosingParenthesis { span: Span },
    NoMatchingOpeningParenthesis { span: Span },
}
