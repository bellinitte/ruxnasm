use super::Span;

#[derive(Debug, Clone)]
pub enum Error {
    MacroUndefined {
        name: String,
        span: Span,
    },
    MacroDefinedMoreThanOnce {
        name: String,
        span: Span,
        other_span: Span,
    },
    LabelDefinedMoreThanOnce {
        name: String,
        span: Span,
        other_span: Span,
    },
    OpeningBraceNotAfterMacroDefinition {
        span: Span,
    },
    NoMatchingOpeningBrace {
        span: Span,
    },
    NoMatchingClosingBrace {
        span: Span,
    },
    SublabelDefinedWithoutScope {
        name: String,
        span: Span,
    },
    NoMatchingOpeningBracket {
        span: Span,
    },
    NoMatchingClosingBracket {
        span: Span,
    },
}
