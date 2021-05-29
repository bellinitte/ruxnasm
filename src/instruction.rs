use super::{impl_spanning, Span, Spanned, Spanning};

#[derive(Debug, Copy, Clone)]
pub struct Instruction {
    pub instruction_kind: InstructionKind,
    pub keep: bool,
    pub r#return: bool,
    pub short: bool,
}

impl_spanning!(Instruction);

#[derive(Debug, Copy, Clone)]
pub enum InstructionKind {
    Break,
    Literal,
    NoOperation,
    Pop,
    Duplicate,
    Swap,
    Over,
    Rotate,
    Equal,
    NotEqual,
    GreaterThan,
    LesserThan,
    Jump,
    JumpCondition,
    JumpStash,
    Stash,
    LoadZeroPage,
    StoreZeroPage,
    LoadRelative,
    StoreRelative,
    LoadAbsolute,
    StoreAbsolute,
    DeviceIn,
    DeviceOut,
    Add,
    Subtract,
    Multiply,
    Divide,
    And,
    Or,
    ExclusiveOr,
    Shift,
}

impl From<InstructionKind> for u8 {
    fn from(instruction_kind: InstructionKind) -> Self {
        match instruction_kind {
            InstructionKind::Break => 0x00,
            InstructionKind::Literal => 0x01,
            InstructionKind::NoOperation => 0x02,
            InstructionKind::Pop => 0x03,
            InstructionKind::Duplicate => 0x04,
            InstructionKind::Swap => 0x05,
            InstructionKind::Over => 0x06,
            InstructionKind::Rotate => 0x07,
            InstructionKind::Equal => 0x08,
            InstructionKind::NotEqual => 0x09,
            InstructionKind::GreaterThan => 0x0a,
            InstructionKind::LesserThan => 0x0b,
            InstructionKind::Jump => 0x0c,
            InstructionKind::JumpCondition => 0x0d,
            InstructionKind::JumpStash => 0x0e,
            InstructionKind::Stash => 0x0f,
            InstructionKind::LoadZeroPage => 0x10,
            InstructionKind::StoreZeroPage => 0x11,
            InstructionKind::LoadRelative => 0x12,
            InstructionKind::StoreRelative => 0x13,
            InstructionKind::LoadAbsolute => 0x14,
            InstructionKind::StoreAbsolute => 0x15,
            InstructionKind::DeviceIn => 0x16,
            InstructionKind::DeviceOut => 0x17,
            InstructionKind::Add => 0x18,
            InstructionKind::Subtract => 0x19,
            InstructionKind::Multiply => 0x1a,
            InstructionKind::Divide => 0x1b,
            InstructionKind::And => 0x1c,
            InstructionKind::Or => 0x1d,
            InstructionKind::ExclusiveOr => 0x1e,
            InstructionKind::Shift => 0x1f,
        }
    }
}
