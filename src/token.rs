use super::Instruction;

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    Instruction(Instruction),
    PadAbsolute(usize),
    PadRelative(usize),
    LiteralZeroPageAddress(ScopedIdentifier),
    LiteralRelativeAddress(ScopedIdentifier),
    LiteralAbsoluteAddress(ScopedIdentifier),
    RawAddress(ScopedIdentifier),
    LiteralHexByte(u8),
    LiteralHexShort(u16),
    RawHexByte(u8),
    RawHexShort(u16),
    RawChar(u8),
    RawWord(String),
}

#[derive(Debug, Clone)]
pub(crate) enum ScopedIdentifier {
    Label(String),
    Sublabel(String, String),
}

impl ToString for ScopedIdentifier {
    fn to_string(&self) -> String {
        match self {
            Self::Label(name) => name.to_owned(),
            Self::Sublabel(label_name, sublabel_name) => {
                format!("{}/{}", label_name, sublabel_name)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Token {
    OpeningBracket,
    ClosingBracket,
    OpeningBrace,
    ClosingBrace,
    Instruction(Instruction),
    MacroDefine(String),
    MacroInvoke(String),
    PadAbsolute(usize),
    PadRelative(usize),
    LabelDefine(String),
    SublabelDefine(String),
    LiteralZeroPageAddress(Identifier),
    LiteralRelativeAddress(Identifier),
    LiteralAbsoluteAddress(Identifier),
    RawAddress(Identifier),
    LiteralHexByte(u8),
    LiteralHexShort(u16),
    RawHexByte(u8),
    RawHexShort(u16),
    RawChar(u8),
    RawWord(String),
}

#[derive(Debug, Clone)]
pub(crate) enum Identifier {
    Label(String),
    Sublabel(String),
    Path(String, String),
}
