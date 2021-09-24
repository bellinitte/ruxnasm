#[derive(Debug, Copy, Clone)]
pub(crate) struct Instruction {
    pub(crate) instruction_kind: InstructionKind,
    pub(crate) keep: bool,
    pub(crate) r#return: bool,
    pub(crate) short: bool,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum InstructionKind {
    // Stack
    BreakOrLiteral = 0x00,
    Increment,
    Pop,
    Duplicate,
    NoOperation,
    Swap,
    Over,
    Rotate = 0x07,
    // Logic
    Equal = 0x08,
    NotEqual,
    GreaterThan,
    LesserThan,
    Jump,
    JumpCondition,
    JumpStash,
    Stash = 0x0f,
    // Memory
    LoadZeroPage = 0x10,
    StoreZeroPage,
    LoadRelative,
    StoreRelative,
    LoadAbsolute,
    StoreAbsolute,
    DeviceIn,
    DeviceOut = 0x17,
    // Arithmetic
    Add = 0x18,
    Subtract,
    Multiply,
    Divide,
    And,
    Or,
    ExclusiveOr,
    Shift = 0x1f,
}
