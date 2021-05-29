use super::Span;

pub enum Error {
    IdentifierExpected { span: Span },
    HexNumberExpected { span: Span },
    HexDigitInvalid { digit: char, span: Span },
    HexNumberUnevenLength { span: Span },
    HexNumberTooLarge { length: usize, span: Span },
    CharacterExpected { span: Span },
    CharacterSequenceExpected { span: Span },
    InstructionInvalid { instruction: String, span: Span },
    InstructionModeInvalid { instruction_mode: char, span: Span },
}
