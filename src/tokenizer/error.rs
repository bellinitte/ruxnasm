use super::Span;

pub enum Error {
    MacroNameExpected {
        span: Span,
    },
    LabelExpected {
        span: Span,
    },
    IdentifierExpected {
        span: Span,
    },
    HexNumberExpected {
        span: Span,
    },
    HexDigitInvalid {
        digit: char,
        number: String,
        span: Span,
    },
    HexNumberUnevenLength {
        length: usize,
        number: String,
        span: Span,
    },
    HexNumberTooLong {
        length: usize,
        number: String,
        span: Span,
    },
    // InstructionModeDefinedMoreThanOnce {
    //     instruction_mode: char,
    //     instruction: String,
    //     span: Span,
    //     other_span: Span,
    // },
    MacroCannotBeAHexNumber {
        number: String,
        span: Span,
    },
    MacroCannotBeAnInstruction {
        instruction: String,
        span: Span,
    },
}
