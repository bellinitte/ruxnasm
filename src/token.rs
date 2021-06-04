use super::Instruction;
use super::{impl_spanning, Span, Spanned, Spanning};

#[derive(Debug, Clone)]
pub enum Token {
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
pub enum Identifier {
    Label(String),
    Sublabel(String),
    Path(String, String),
}

impl_spanning!(Token);
impl_spanning!(Identifier);
