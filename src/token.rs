use super::Instruction;

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    Instruction(Instruction),
    PadAbsolute(u16),
    PadRelative(u16),
    LiteralZeroPageAddress(ScopedIdentifier),
    LiteralRelativeAddress(ScopedIdentifier),
    LiteralAbsoluteAddress(ScopedIdentifier),
    RawAddress(ScopedIdentifier),
    LiteralHexByte(u8),
    LiteralHexShort(u16),
    RawHexByte(u8),
    RawHexShort(u16),
    RawChar(u8),
    RawWord(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ScopedIdentifier {
    Label(Vec<u8>),
    Sublabel(Vec<u8>, Vec<u8>),
}

impl ScopedIdentifier {
    pub fn is_captital(&self) -> bool {
        match self {
            Self::Label(name) => name.iter().next().unwrap().is_ascii_uppercase(),
            Self::Sublabel(name, _) => name.iter().next().unwrap().is_ascii_uppercase(),
        }
    }
}

impl ToString for ScopedIdentifier {
    fn to_string(&self) -> String {
        match self {
            Self::Label(name) => String::from_utf8_lossy(name).into_owned(),
            Self::Sublabel(label_name, sublabel_name) => {
                format!(
                    "{}/{}",
                    String::from_utf8_lossy(label_name),
                    String::from_utf8_lossy(sublabel_name)
                )
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
    MacroDefine(Vec<u8>),
    MacroInvoke(Vec<u8>),
    PadAbsolute(u16),
    PadRelative(u16),
    LabelDefine(Vec<u8>),
    SublabelDefine(Vec<u8>),
    LiteralZeroPageAddress(Identifier),
    LiteralRelativeAddress(Identifier),
    LiteralAbsoluteAddress(Identifier),
    RawAddress(Identifier),
    LiteralHexByte(u8),
    LiteralHexShort(u16),
    RawHexByte(u8),
    RawHexShort(u16),
    RawChar(u8),
    RawWord(Vec<u8>),
}

#[derive(Debug, Clone)]
pub(crate) enum Identifier {
    Label(Vec<u8>),
    Sublabel(Vec<u8>),
    Path(Vec<u8>, Vec<u8>),
}
