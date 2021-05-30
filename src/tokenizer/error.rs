use super::Span;

pub enum Error {
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
    HexNumberTooLarge {
        length: usize,
        number: String,
        span: Span,
    },
    CharacterExpected {
        span: Span,
    },
    InstructionInvalid {
        instruction: String,
        span: Span,
    },
    InstructionModeInvalid {
        instruction_mode: char,
        instruction: String,
        span: Span,
    },
    InstructionModeDefinedMoreThanOnce {
        instruction_mode: char,
        instruction: String,
        span: Span,
        other_span: Span,
    },
    IdentifierCannotBeAHexNumber {
        number: String,
        span: Span,
    },
    IdentifierCannotBeAnInstruction {
        instruction: String,
        span: Span,
    },
}
