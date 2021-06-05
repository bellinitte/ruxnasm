use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    TokenTrimmed {
        span: Range<usize>,
    },
    InstructionModeDefinedMoreThanOnce {
        instruction_mode: char,
        instruction: String,
        span: Range<usize>,
        other_span: Range<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NoMatchingClosingParenthesis {
        span: Range<usize>,
    },
    NoMatchingOpeningParenthesis {
        span: Range<usize>,
    },
    MacroNameExpected {
        span: Range<usize>,
    },
    LabelExpected {
        span: Range<usize>,
    },
    SublabelExpected {
        span: Range<usize>,
    },
    SlashInLabelOrSublabel {
        span: Range<usize>,
    },
    MoreThanOneSlashInIdentifier {
        span: Range<usize>,
    },
    AmpersandAtTheStartOfLabel {
        span: Range<usize>,
    },
    IdentifierExpected {
        span: Range<usize>,
    },
    HexNumberExpected {
        span: Range<usize>,
    },
    HexNumberOrCharacterExpected {
        span: Range<usize>,
    },
    CharacterExpected {
        span: Range<usize>,
    },
    MoreThanOneByteFound {
        bytes: Vec<u8>,
        span: Range<usize>,
    },
    HexDigitInvalid {
        digit: char,
        number: String,
        span: Range<usize>,
    },
    HexNumberUnevenLength {
        length: usize,
        number: String,
        span: Range<usize>,
    },
    HexNumberTooLong {
        length: usize,
        number: String,
        span: Range<usize>,
    },
    MacroCannotBeAHexNumber {
        number: String,
        span: Range<usize>,
    },
    MacroCannotBeAnInstruction {
        instruction: String,
        span: Range<usize>,
    },
    MacroUndefined {
        name: String,
        span: Range<usize>,
    },
    MacroDefinedMoreThanOnce {
        name: String,
        span: Range<usize>,
        other_span: Range<usize>,
    },
    LabelDefinedMoreThanOnce {
        name: String,
        span: Range<usize>,
        other_span: Range<usize>,
    },
    OpeningBraceNotAfterMacroDefinition {
        span: Range<usize>,
    },
    NoMatchingOpeningBrace {
        span: Range<usize>,
    },
    NoMatchingClosingBrace {
        span: Range<usize>,
    },
    SublabelDefinedWithoutScope {
        name: String,
        span: Range<usize>,
    },
    NoMatchingOpeningBracket {
        span: Range<usize>,
    },
    NoMatchingClosingBracket {
        span: Range<usize>,
    },
}
