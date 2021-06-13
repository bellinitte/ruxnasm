use std::ops::Range;

/// Enum representing every warning that can be reported from Ruxnasm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    /// This warnings gets reported when a token is longer than 64 characters and must be cut off.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// @a-really-long-label-name-like-seriously-this-is-so-long-why-would-anyone-do-this
    /// ```
    TokenTrimmed {
        /// Span of the cut off part of the token.
        span: Range<usize>,
    },
    /// This warning gets reported when an instruction mode is defined multiple times for a
    /// single instruction, which is valid, but unnecessary.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// ADD2k2
    /// ```
    InstructionModeDefinedMoreThanOnce {
        /// Character representing the instruction mode.
        instruction_mode: char,
        /// The whole instruction.
        instruction: String,
        /// Span of the unnecessary instruction mode character.
        span: Range<usize>,
        /// Span of the instruction mode character defined for the first time.
        other_span: Range<usize>,
    },
    MacroUnused {
        name: String,
        span: Range<usize>,
    },
    LabelUnused {
        name: String,
        span: Range<usize>,
    },
}

/// Enum representing every error that can be reported from Ruxnasm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// This error gets reported when an opening parenthesis is not closed i.e. it has
    /// no matching closing parenthesis.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// (
    /// ```
    NoMatchingClosingParenthesis {
        /// Span of the opening parenthesis with no matching closing parenthesis.
        span: Range<usize>,
    },
    /// This error gets reported when a closing parenthesis has no matching opening
    /// parenthesis.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// )
    /// ```
    NoMatchingOpeningParenthesis {
        /// Span of the closing parenthesis with no matching opening parenthesis.
        span: Range<usize>,
    },
    /// This error gets reported when there is no macro name after a macro definition
    /// rune.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// %
    /// ```
    MacroNameExpected {
        /// Span of the macro definition rune.
        span: Range<usize>,
    },
    /// This error gets reported when there is no label name after a label definition
    /// rune.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// @
    /// ```
    LabelExpected {
        /// Span of the label definition rune.
        span: Range<usize>,
    },
    /// This error gets reported when there is no sublabel name after a sublabel
    /// definition rune.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// &
    /// ```
    SublabelExpected {
        /// Span of the sublabel definition rune.
        span: Range<usize>,
    },
    /// This error gets reported when a label or a sublabel name contains a slash
    /// character.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// @label/name
    /// ```
    SlashInLabelOrSublabel {
        /// Span of the slash in the label of sublabel.
        span: Range<usize>,
    },
    /// This error gets reported when an identifier contains more than one slash
    /// character.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// .label-name/sublabel/name
    /// ```
    MoreThanOneSlashInIdentifier {
        /// Span of the slash in the identifier.
        span: Range<usize>,
    },
    /// This error gets reported when a label name in a label definition has an
    /// ampersand as the first character.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// @&label-name
    /// ```
    AmpersandAtTheStartOfLabel {
        /// Span of the ampersand at the start of the label.
        span: Range<usize>,
    },
    /// This error gets reported when there is no identifier after an address rune
    /// (literal zero-page address rune, literal relative address rune, literal
    /// absolute address runem, or raw address rune).
    ///
    /// # Example
    ///
    /// ```uxntal
    /// .
    /// ```
    IdentifierExpected {
        /// Span of the address rune.
        span: Range<usize>,
    },
    /// This error gets reported when there is no hexadecimal number after an
    /// absolute or relative pad rune.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// |
    /// ```
    HexNumberExpected {
        /// Span of the abolute or relative pad rune.
        span: Range<usize>,
    },
    /// This error gets reported when there is no character or hexadecimal number
    /// after a literal hex rune.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// #
    /// ```
    HexNumberOrCharacterExpected {
        /// Span of the literal hex rune.
        span: Range<usize>,
    },
    /// This error gets reported when there is no character after a character rune.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// '
    /// ```
    CharacterExpected {
        /// Span of the character rune.
        span: Range<usize>,
    },
    /// This error gets reported when there is more than one byte after a character
    /// rune.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// 'characters
    /// ```
    MoreThanOneByteFound {
        /// Sequence of bytes after the character rune.
        bytes: Vec<u8>,
        /// Span of the characters after the character rune.
        span: Range<usize>,
    },
    /// This error gets reported when a hexadecimal number contains an invalid
    /// digit.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// #00g0
    /// ```
    HexDigitInvalid {
        /// The invalid digit.
        digit: char,
        /// The whole hexadecimal number with the invalid digit.
        number: String,
        /// Span of the hexadecimal number.
        span: Range<usize>,
    },
    /// This error gets reported when a hexadecimal number after a literal hex
    /// rune has a length of 3, i.e. it is made out of exactly 3 hexadecimal digits.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// #000
    /// ```
    HexNumberUnevenLength {
        /// Length of the hexadecimal number.
        length: usize,
        /// The hexadecimal number.
        number: String,
        /// Span of the hexadecimal number.
        span: Range<usize>,
    },
    /// This error gets reported when the hexadecimal number after a literal hex
    /// rune is longer than 4 hexadecimal digits.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// #fffff
    /// ```
    HexNumberTooLong {
        /// Length of the hexadecimal number.
        length: usize,
        /// The hexadecimal number.
        number: String,
        /// Span of the hexadecimal number.
        span: Range<usize>,
    },
    /// This error gets reported when the macro name after a macro definition
    /// rune is a valid hexadecimal number i.e. it contains exactly 2 or 4 valid
    /// hexadecimal digits.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// %01
    /// ```
    MacroCannotBeAHexNumber {
        /// The hexadecimal number that was meant to be a macro name.
        number: String,
        /// Span of the hexadecimal number that was meant to be a macro name.
        span: Range<usize>,
    },
    /// This error gets reported when the macro name after a macro definition
    /// rune is a valid instruction.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// %ADD
    /// ```
    MacroCannotBeAnInstruction {
        /// The instruction that was meant to be a macro name.
        instruction: String,
        /// Span of the instruction that was meant to be a macro name.
        span: Range<usize>,
    },
    /// This error gets reported during an attempt to expand a macro that has not
    /// been previously defined.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// macro
    /// ```
    MacroUndefined {
        /// Name of the macro.
        name: String,
        /// Span of the macro invocation.
        span: Range<usize>,
    },
    /// This error gets reported when a macro with the same name is defined
    /// multiple times.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// %macro { ADD }
    /// %macro { ADD }
    /// ```
    MacroDefinedMoreThanOnce {
        /// Name of the macro.
        name: String,
        /// Span of the current macro definition.
        span: Range<usize>,
        /// Span of the previous macro definition.
        other_span: Range<usize>,
    },
    /// This error gets reported when a label with the same name is defined
    /// multiple times.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// @label
    /// @label
    /// ```
    LabelDefinedMoreThanOnce {
        /// Name of the label.
        name: String,
        /// Span of the current label definition.
        span: Range<usize>,
        /// Span of the previous label definition.
        other_span: Range<usize>,
    },
    /// This error gets reported when an opening brace character is not directly
    /// preceded by a macro definition.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// { ADD }
    /// ```
    OpeningBraceNotAfterMacroDefinition {
        /// Span of the opening brace.
        span: Range<usize>,
    },
    /// This error gets reported when a closing brace has no matching opening
    /// brace.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// }
    /// ```
    NoMatchingOpeningBrace {
        /// Span of the closing brace with no matching opening brace.
        span: Range<usize>,
    },
    /// This error gets reported when an opening brace is not closed i.e. it has
    /// no matching closing brace.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// %macro {
    /// ```
    NoMatchingClosingBrace {
        /// Span of the opening brace with no matching closing brace.
        span: Range<usize>,
    },
    /// This error gets reported during an attempt to define a sublabel, when
    /// no previous label has been defined.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// &sublabel
    /// ```
    SublabelDefinedWithoutScope {
        /// Name of the sublabel.
        name: String,
        /// Span of the sublabel definition.
        span: Range<usize>,
    },
    /// This error gets reported when a closing bracket has no matching opening
    /// bracket.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// ]
    /// ```
    NoMatchingOpeningBracket {
        /// Span of the closing bracket with no matching opening bracket.
        span: Range<usize>,
    },
    /// This error gets reported when an opening bracket is not closed i.e. it has
    /// no matching closing bracket.
    ///
    /// # Example
    ///
    /// ```uxntal
    /// [
    /// ```
    NoMatchingClosingBracket {
        /// Span of the opening bracket with no matching closing bracket.
        span: Range<usize>,
    },
    MacroError {
        original_error: Box<Error>,
        span: Range<usize>,
    },
    SublabelReferencedWithoutScope {
        /// Name of the sublabel.
        name: String,
        /// Span of the sublabel reference.
        span: Range<usize>,
    },
    LabelUndefined {
        /// Name of the label.
        name: String,
        /// Span of the label reference.
        span: Range<usize>,
    },
    AddressNotZeroPage {
        address: u16,
        identifier: String,
        span: Range<usize>,
    },
    AddressTooFar {
        distance: usize,
        identifier: String,
        span: Range<usize>,
        other_span: Range<usize>,
    },
    BytesInZerothPage {
        spans: Vec<Range<usize>>,
    },
    PaddedBackwards {
        previous_pointer: usize,
        desired_pointer: usize,
        span: Range<usize>,
    }
}
