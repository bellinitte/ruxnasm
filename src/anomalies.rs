use super::Span;

#[derive(Debug, Clone)]
pub enum Warning {
    TokenTrimmed {
        span: Span,
    },
    InstructionModeDefinedMoreThanOnce {
        instruction_mode: char,
        instruction: String,
        span: Span,
        other_span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum Error {
    NoMatchingClosingParenthesis {
        span: Span,
    },
    NoMatchingOpeningParenthesis {
        span: Span,
    },
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
