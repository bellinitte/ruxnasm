use super::Span;

#[derive(Debug, Clone)]
pub enum Warning {
    InstructionModeDefinedMoreThanOnce {
        instruction_mode: char,
        instruction: String,
        span: Span,
        other_span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum Error {
    MacroNameExpected {
        span: Span,
    },
    LabelExpected {
        span: Span,
    },
    SublabelExpected {
        span: Span,
    },
    SlashInLabelOrSublabel {
        span: Span,
    },
    MoreThanOneSlashInIdentifier {
        span: Span,
    },
    AmpersandAtTheStartOfLabel {
        span: Span,
    },
    IdentifierExpected {
        span: Span,
    },
    HexNumberExpected {
        span: Span,
    },
    HexNumberOrCharacterExpected {
        span: Span,
    },
    CharacterExpected {
        span: Span,
    },
    MoreThanOneByteFound {
        bytes: Vec<u8>,
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
    MacroCannotBeAHexNumber {
        number: String,
        span: Span,
    },
    MacroCannotBeAnInstruction {
        instruction: String,
        span: Span,
    },
}
