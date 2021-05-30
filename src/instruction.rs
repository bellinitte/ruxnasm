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
