use std::collections::HashSet;

use crate::{
    instruction::InstructionKind,
    span::{Span, Spanned},
    token::{ScopedIdentifier, Statement},
    walker::Definitions,
    Error, Warning,
};

const LIT: u8 = 0x01;
const LIT2: u8 = 0x21;

struct Binary {
    data: [u8; 256 * 256 - 256],
    pointer: u16,
    length: u16,
}

impl Binary {
    pub fn new() -> Self {
        Self {
            data: [0; 256 * 256 - 256],
            pointer: 0,
            length: 0,
        }
    }

    pub fn push_byte(&mut self, byte: u8) {
        self.data[self.pointer as usize - 256] = byte;
        self.increment_pointer(1);
        self.length = self.pointer;
    }

    pub fn push_short(&mut self, short: u16) {
        self.push_byte(((short >> 8) & 0xff) as u8);
        self.push_byte((short & 0x00ff) as u8);
    }

    pub fn set_pointer(&mut self, to: u16) {
        self.pointer = to;
    }

    pub fn increment_pointer(&mut self, by: u16) {
        self.pointer += by;
    }

    pub fn get_pointer(&self) -> u16 {
        self.pointer
    }
}

impl From<Binary> for Vec<u8> {
    fn from(binary: Binary) -> Self {
        binary.data[0..binary.length as usize - 256].into()
    }
}

pub(crate) fn emit(
    statements: Vec<Spanned<Statement>>,
    definitions: Definitions,
) -> Result<(Vec<u8>, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
    let mut errors: Vec<Error> = Vec::new();
    let mut warnings: Vec<Warning> = Vec::new();

    let mut unused_labels: HashSet<&ScopedIdentifier> = definitions.labels.keys().collect();

    let mut binary = Binary::new();

    for statement in statements {
        match statement {
            Spanned {
                node: Statement::Instruction(instruction),
                ..
            } => {
                let opcode = match instruction.instruction_kind {
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
                } | ((instruction.short as u8) << 5)
                    | ((instruction.r#return as u8) << 6)
                    | ((instruction.keep as u8) << 7);
                binary.push_byte(opcode);
            }
            Spanned {
                node: Statement::PadAbsolute(value),
                ..
            } => {
                binary.set_pointer(value as u16);
            }
            Spanned {
                node: Statement::PadRelative(value),
                ..
            } => {
                binary.increment_pointer(value as u16);
            }
            Spanned {
                node: Statement::LiteralZeroPageAddress(scoped_identifier),
                span,
            } => match find_address(&scoped_identifier, &definitions, &span) {
                Ok((address, _)) => {
                    unused_labels.remove(&scoped_identifier);
                    if address <= 0xff {
                        binary.push_byte(LIT);
                        binary.push_byte((address & 0xff) as u8);
                    } else {
                        errors.push(Error::AddressNotZeroPage {
                            address,
                            identifier: scoped_identifier.to_string(),
                            span: span.into(),
                        });
                        binary.increment_pointer(2);
                    }
                }
                Err(err) => {
                    errors.push(err);
                    binary.increment_pointer(2);
                }
            },
            Spanned {
                node: Statement::LiteralRelativeAddress(scoped_identifier),
                span,
            } => match find_address(&scoped_identifier, &definitions, &span) {
                Ok((address, other_span)) => {
                    unused_labels.remove(&scoped_identifier);
                    let offset = address as isize - binary.get_pointer() as isize - 3;
                    if offset < -126 || offset > 126 {
                        errors.push(Error::AddressTooFar {
                            distance: offset.abs() as usize,
                            identifier: scoped_identifier.to_string(),
                            span: span.into(),
                            other_span: other_span.into(),
                        });
                        binary.increment_pointer(2);
                    } else {
                        binary.push_byte(LIT);
                        binary.push_byte(offset as u8);
                    }
                }
                Err(err) => {
                    errors.push(err);
                    binary.increment_pointer(2);
                }
            },
            Spanned {
                node: Statement::LiteralAbsoluteAddress(scoped_identifier),
                span,
            } => match find_address(&scoped_identifier, &definitions, &span) {
                Ok((address, _)) => {
                    unused_labels.remove(&scoped_identifier);
                    binary.push_byte(LIT2);
                    binary.push_short(address);
                }
                Err(err) => {
                    errors.push(err);
                    binary.increment_pointer(3);
                }
            },
            Spanned {
                node: Statement::RawAddress(scoped_identifier),
                span,
            } => match find_address(&scoped_identifier, &definitions, &span) {
                Ok((address, _)) => {
                    unused_labels.remove(&scoped_identifier);
                    binary.push_short(address);
                }
                Err(err) => {
                    errors.push(err);
                    binary.increment_pointer(2);
                }
            },
            Spanned {
                node: Statement::LiteralHexByte(value),
                ..
            } => {
                binary.push_byte(LIT);
                binary.push_byte(value);
            }
            Spanned {
                node: Statement::LiteralHexShort(value),
                ..
            } => {
                binary.push_byte(LIT2);
                binary.push_short(value);
            }
            Spanned {
                node: Statement::RawHexByte(value),
                ..
            } => {
                binary.push_byte(value);
            }
            Spanned {
                node: Statement::RawHexShort(value),
                ..
            } => {
                binary.push_short(value);
            }
            Spanned {
                node: Statement::RawChar(value),
                ..
            } => {
                binary.push_byte(value);
            }
            Spanned {
                node: Statement::RawWord(word),
                ..
            } => {
                for byte in word.bytes() {
                    binary.push_byte(byte);
                }
            }
        }
    }

    for unused_label_name in unused_labels
        .into_iter()
        .filter(|scoped_identifier| !scoped_identifier.is_captital())
    {
        let (_, span) = definitions.labels[&unused_label_name];
        warnings.push(Warning::LabelUnused {
            name: unused_label_name.to_string(),
            span: span.into(),
        });
    }

    if errors.is_empty() {
        Ok((binary.into(), warnings))
    } else {
        Err((errors, warnings))
    }
}

fn find_address(
    scoped_identifier: &ScopedIdentifier,
    definitions: &Definitions,
    span: &Span,
) -> Result<(u16, Span), Error> {
    match definitions.labels.get(scoped_identifier) {
        Some((address, span)) => {
            return Ok((*address, *span));
        }
        None => {
            return Err(Error::LabelUndefined {
                name: scoped_identifier.to_string(),
                span: (*span).into(),
            });
        }
    }
}
