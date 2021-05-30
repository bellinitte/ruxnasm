use super::Instruction;
use super::{impl_spanning, Span, Spanned, Spanning};

// TODO:
// - what's up with the ,&loop?
// - how do sublabels work?
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
    LiteralZeroPageAddress(Breadcrumbs),
    LiteralRelativeAddress(Breadcrumbs),
    LiteralAbsoluteAddress(Breadcrumbs),
    RawHexByte(u8),
    RawHexShort(u16),
    RawAddress(Breadcrumbs),
    RawChar(u8),
    RawWord(Vec<u8>),
    OpeningBrace,
    ClosingBrace,
}

#[derive(Debug)]
pub enum Breadcrumbs {
    Label(String),
    Sublabel(String, String),
}

impl_spanning!(Token);
