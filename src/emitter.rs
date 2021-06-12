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
    zeroth_page_spans: HashSet<Span>,
}

impl Binary {
    pub fn new() -> Self {
        Self {
            data: [0; 256 * 256 - 256],
            pointer: 0,
            zeroth_page_spans: HashSet::new(),
        }
    }

    pub fn push_byte(&mut self, byte: u8, span: Span) {
        if self.pointer < 256 {
            self.zeroth_page_spans.insert(span);
        } else {
            self.data[self.pointer as usize - 256] = byte;
        }
        self.increment_pointer(1);
    }

    pub fn push_short(&mut self, short: u16, span: Span) {
        self.push_byte(((short >> 8) & 0xff) as u8, span);
        self.push_byte((short & 0x00ff) as u8, span);
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

    pub fn get_zeroth_page_spans(&self) -> Vec<Span> {
        self.zeroth_page_spans.iter().copied().collect()
    }
}

impl From<Binary> for Vec<u8> {
    fn from(binary: Binary) -> Self {
        let position = binary
            .data
            .iter()
            .rposition(|byte| *byte != 0x00)
            .map(|i| i + 1)
            .unwrap_or(0);
        binary.data[0..position].into()
    }
}

pub(crate) fn emit(
    statements: Vec<Spanned<Statement>>,
    definitions: Definitions,
) -> Result<(Vec<u8>, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
    let mut errors: Vec<Error> = Vec::new();
    let mut _warnings: Vec<Warning> = Vec::new();

    let mut binary = Binary::new();

    for statement in statements {
        match statement {
            Spanned {
                node: Statement::Instruction(instruction),
                span,
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
                binary.push_byte(opcode, span);
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
                    if address <= 0xff {
                        binary.push_byte(LIT, span);
                        binary.push_byte((address & 0xff) as u8, span);
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
                    let offset = address as isize - binary.get_pointer() as isize - 3;
                    if offset < -126 || offset > 126 {
                        errors.push(Error::AddressTooFar {
                            distance: offset.abs() as usize,
                            identifier: scoped_identifier.to_string(),
                            span: span.into(),
                            other_span: other_span.into(),
                            debug: format!("address: {}, pointer: {}, a - p - 3 = {}", address, binary.get_pointer(), offset),
                        });
                        binary.increment_pointer(2);
                    } else {
                        binary.push_byte(LIT, span);
                        binary.push_byte(offset as u8, span);
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
                    binary.push_byte(LIT2, span);
                    binary.push_short(address, span);
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
                    binary.push_short(address, span);
                }
                Err(err) => {
                    errors.push(err);
                    binary.increment_pointer(2);
                }
            },
            Spanned {
                node: Statement::LiteralHexByte(value),
                span,
            } => {
                binary.push_byte(LIT, span);
                binary.push_byte(value, span);
            }
            Spanned {
                node: Statement::LiteralHexShort(value),
                span,
            } => {
                binary.push_byte(LIT2, span);
                binary.push_short(value, span);
            }
            Spanned {
                node: Statement::RawHexByte(value),
                span,
            } => {
                binary.push_byte(value, span);
            }
            Spanned {
                node: Statement::RawHexShort(value),
                span,
            } => {
                binary.push_short(value, span);
            }
            Spanned {
                node: Statement::RawChar(value),
                span,
            } => {
                binary.push_byte(value, span);
            }
            Spanned {
                node: Statement::RawWord(word),
                span,
            } => {
                for byte in word.bytes() {
                    binary.push_byte(byte, span);
                }
            }
        }
    }

    let zeroth_page_spans = binary.get_zeroth_page_spans();
    if !zeroth_page_spans.is_empty() {
        errors.push(Error::BytesInZerothPage {
            spans: zeroth_page_spans
                .into_iter()
                .map(|span| span.into())
                .collect(),
        })
    }

    if errors.is_empty() {
        Ok((binary.into(), _warnings))
    } else {
        Err((errors, _warnings))
    }
}

fn find_address(
    scoped_identifier: &ScopedIdentifier,
    definitions: &Definitions,
    span: &Span,
) -> Result<(u16, Span), Error> {
    match scoped_identifier {
        ScopedIdentifier::Label(name) => match definitions.labels.get(name) {
            Some((address, span)) => {
                return Ok((*address, *span));
            }
            None => {
                return Err(Error::LabelUndefined {
                    name: name.to_owned(),
                    span: (*span).into(),
                });
            }
        },
        ScopedIdentifier::Sublabel(label_name, sublabel_name) => {
            match definitions
                .sublabels
                .get(&(label_name.to_owned(), sublabel_name.to_owned()))
            {
                Some((address, span)) => {
                    return Ok((*address, *span));
                }
                None => {
                    return Err(Error::SublabelUndefined {
                        label_name: label_name.to_owned(),
                        sublabel_name: sublabel_name.to_owned(),
                        span: (*span).into(),
                    });
                }
            }
        }
    }
}
