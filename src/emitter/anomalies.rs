use super::Span;

#[derive(Clone)]
pub enum Warning {
    ClosingBraceMisplaced { span: Span },
}
