use super::Instruction;
use super::{impl_spanning, Span, Spanned, Spanning};

#[derive(Debug)]
pub enum Token {
    Instruction(Instruction),
    MacroDefine(String),
    PadAbsolute(usize),
    PadRelative(usize),
    LabelDefine(String),
    SublabelDefine(String),
    LiteralHexByte(u8),
    LiteralHexShort(u16),
    LiteralZeroPageAddress(String),
    LiteralRelativeAddress(String),
    LiteralAbsoluteAddress(String),
    RawHexByte(u8),
    RawHexShort(u16),
    RawAddress(String),
    RawChar(u8),
    RawWord(Vec<u8>),
    OpeningBrace,
    ClosingBrace,
}

impl_spanning!(Token);
