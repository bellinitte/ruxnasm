use crate::{Instruction, InstructionKind};

use super::Token;
use super::{Location, Span, Spanned, Spanning};
pub use error::Error;

mod error;

pub fn tokenize(input_file_contents: &str) -> Result<Vec<Spanned<Token>>, Vec<Error>> {
    let symbols = scan(input_file_contents);

    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut i = 0;

    'tokens: loop {
        'gap: loop {
            match symbols.get(i).as_ref() {
                Some(Spanned { node: c, .. }) if c.is_whitespace() => {
                    i += 1;
                }
                Some(Spanned { node: '[', .. }) => {
                    i += 1;
                }
                Some(Spanned { node: ']', .. }) => {
                    i += 1;
                }
                Some(Spanned { node: '(', .. }) => {
                    i += 1;
                    let mut comment = true;
                    // comment
                    while comment {
                        match symbols.get(i) {
                            Some(Spanned { node: ')', .. }) => {
                                i += 1;
                                comment = false;
                            }
                            Some(_) => {
                                i += 1;
                            }
                            None => break 'gap,
                        }
                    }
                }
                _ => break 'gap,
            };
        }
        match symbols.get(i) {
            Some(Spanned { node: '{', span }) => {
                i += 1;
                tokens.push(Token::OpeningBrace.spanning(*span));
            }
            Some(Spanned { node: '}', span }) => {
                i += 1;
                tokens.push(Token::ClosingBrace.spanning(*span));
            }
            Some(Spanned { node: '%', span }) => {
                i += 1;
                // TODO: check if the name is a valid instruction or a hex number
                match parse_word(&symbols, &mut i) {
                    Some(Spanned { node, span }) => {
                        tokens.push(Token::MacroDefine(node).spanning(span))
                    }
                    None => errors.push(Error::IdentifierExpected { span: *span }),
                }
            }
            Some(Spanned { node: '|', .. }) => {
                i += 1;
                match parse_hex_number(&symbols, &mut i) {
                    Ok(Spanned { node, span }) => {
                        tokens.push(Token::PadAbsolute(node).spanning(span))
                    }
                    Err(err) => errors.push(err),
                }
            }
            Some(Spanned { node: '$', .. }) => {
                i += 1;
                match parse_hex_number(&symbols, &mut i) {
                    Ok(Spanned { node, span }) => {
                        tokens.push(Token::PadRelative(node).spanning(span))
                    }
                    Err(err) => errors.push(err),
                }
            }
            Some(Spanned { node: '@', span }) => {
                i += 1;
                match parse_word(&symbols, &mut i) {
                    Some(Spanned { node, span }) => {
                        tokens.push(Token::LabelDefine(node).spanning(span))
                    }
                    None => errors.push(Error::IdentifierExpected { span: *span }),
                }
            }
            Some(Spanned { node: '&', span }) => {
                i += 1;
                match parse_word(&symbols, &mut i) {
                    Some(Spanned { node, span }) => {
                        tokens.push(Token::SublabelDefine(node).spanning(span))
                    }
                    None => errors.push(Error::IdentifierExpected { span: *span }),
                }
            }
            Some(Spanned { node: '#', span }) => {
                i += 1;
                match peek_length(&symbols, i) {
                    0 => errors.push(Error::HexNumberExpected { span: *span }),
                    1 => {
                        tokens.push(Token::LiteralHexByte(symbols[i].node as u8).spanning(*span));
                        i += 1;
                    }
                    2 => match parse_hex_number(&symbols, &mut i) {
                        Ok(Spanned { node, span }) => {
                            tokens.push(Token::LiteralHexByte(node as u8).spanning(span));
                        }
                        Err(err) => errors.push(err),
                    },
                    3 => {
                        errors.push(Error::HexNumberUnevenLength { span: *span });
                        i += 3;
                    }
                    4 => match parse_hex_number(&symbols, &mut i) {
                        Ok(Spanned { node, span }) => {
                            tokens.push(Token::LiteralHexShort(node as u16).spanning(span));
                        }
                        Err(err) => errors.push(err),
                    },
                    length => errors.push(Error::HexNumberTooLarge {
                        length,
                        span: *span,
                    }),
                }
            }
            Some(Spanned { node: '.', span }) => {
                i += 1;
                match parse_word(&symbols, &mut i) {
                    Some(Spanned { node, span }) => {
                        tokens.push(Token::LiteralZeroPageAddress(node).spanning(span))
                    }
                    None => errors.push(Error::IdentifierExpected { span: *span }),
                }
            }
            Some(Spanned { node: ',', span }) => {
                i += 1;
                match parse_word(&symbols, &mut i) {
                    Some(Spanned { node, span }) => {
                        tokens.push(Token::LiteralRelativeAddress(node).spanning(span))
                    }
                    None => errors.push(Error::IdentifierExpected { span: *span }),
                }
            }
            Some(Spanned { node: ';', span }) => {
                i += 1;
                match parse_word(&symbols, &mut i) {
                    Some(Spanned { node, span }) => {
                        tokens.push(Token::LiteralAbsoluteAddress(node).spanning(span))
                    }
                    None => errors.push(Error::IdentifierExpected { span: *span }),
                }
            }
            Some(Spanned { node: ':', span }) => {
                i += 1;
                match parse_word(&symbols, &mut i) {
                    Some(Spanned { node, span }) => {
                        tokens.push(Token::RawAddress(node).spanning(span))
                    }
                    None => errors.push(Error::IdentifierExpected { span: *span }),
                }
            }
            Some(Spanned { node: '\'', span }) => {
                i += 1;
                match symbols.get(i) {
                    Some(Spanned { node, span }) => {
                        tokens.push(Token::RawChar(*node as u8).spanning(*span))
                    }
                    None => errors.push(Error::CharacterExpected { span: *span }),
                }
            }
            Some(Spanned { node: '"', span }) => {
                i += 1;
                match parse_word(&symbols, &mut i) {
                    Some(Spanned { node, span }) => tokens.push(
                        Token::RawWord(node.chars().map(|c| c as u8).collect()).spanning(span),
                    ),
                    None => errors.push(Error::CharacterSequenceExpected { span: *span }),
                }
            }
            Some(Spanned { node: c, span }) if is_hex_digit(*c) => match peek_length(&symbols, i) {
                0 => unreachable!(),
                length if length == 1 || length == 3 => {
                    errors.push(Error::HexNumberUnevenLength { span: *span });
                    i += length;
                }
                2 => match parse_hex_number(&symbols, &mut i) {
                    Ok(Spanned { node, span }) => {
                        tokens.push(Token::RawHexByte(node as u8).spanning(span));
                    }
                    Err(err) => errors.push(err),
                },
                4 => match parse_hex_number(&symbols, &mut i) {
                    Ok(Spanned { node, span }) => {
                        tokens.push(Token::RawHexShort(node as u16).spanning(span));
                    }
                    Err(err) => errors.push(err),
                },
                length => errors.push(Error::HexNumberTooLarge {
                    length,
                    span: *span,
                }),
            },
            Some(_) => match parse_instruction(&symbols, &mut i) {
                Ok(Spanned {
                    node: instruction,
                    span,
                }) => tokens.push(Token::Instruction(instruction).spanning(span)),
                Err(err) => errors.push(err),
            },
            None => break 'tokens,
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

fn scan(string: &str) -> Vec<Spanned<char>> {
    string
        .chars()
        .scan(Location { offset: 0 }, |location, grapheme| {
            let current_location = *location;
            (*location).offset += 1;
            Some(Spanned {
                node: grapheme,
                span: Span {
                    from: current_location,
                    to: *location,
                },
            })
        })
        .collect()
}

fn parse_word<'a>(symbols: &[Spanned<char>], i: &mut usize) -> Option<Spanned<String>> {
    let os = symbols.get(*i).unwrap().span;
    let mut oe = os;
    let mut identifier: String = String::new();
    let mut length: usize = 0;
    loop {
        match symbols.get(*i) {
            Some(Spanned { node: c, span: o }) if !c.is_whitespace() => {
                identifier.push(*c);
                oe = *o;
                length += 1;
                *i += 1;
            }
            _ => {
                return match length {
                    0 => None,
                    _ => Some(identifier.spanning(Span::combine(&os, &oe))),
                }
            }
        }
    }
}

fn parse_hex_number<'a>(symbols: &[Spanned<char>], i: &mut usize) -> Result<Spanned<usize>, Error> {
    let os = symbols.get(*i).unwrap().span;
    let mut oe = os;
    let mut value: usize = 0;
    let mut length: usize = 0;
    loop {
        match symbols.get(*i) {
            Some(Spanned { node: c, span: o }) if is_hex_digit(*c) => {
                value = (value << 4) + to_hex_digit(*c).unwrap() as usize;
                oe = *o;
                length += 1;
                *i += 1;
            }
            // TODO: skip curosr ahead to the end of the number
            Some(Spanned { node: c, span: o }) if !c.is_whitespace() => {
                return Err(Error::HexDigitInvalid {
                    digit: *c,
                    span: *o,
                });
            }
            _ => {
                return match length {
                    0 => Err(Error::HexNumberExpected { span: os }),
                    _ => Ok(value.spanning(Span::combine(&os, &oe))),
                }
            }
        }
    }
}

fn peek_length(symbols: &[Spanned<char>], mut i: usize) -> usize {
    let mut length = 0;
    loop {
        match symbols.get(i) {
            Some(Spanned { node: c, .. }) if c.is_whitespace() => {
                return length;
            }
            _ => {
                length += 1;
                i += 1;
            }
        }
    }
}

fn parse_instruction<'a>(
    symbols: &[Spanned<char>],
    i: &mut usize,
) -> Result<Spanned<Instruction>, Error> {
    let os = symbols.get(*i).unwrap().span;
    let mut oe = os;
    let mut mnemonic: String = String::new();
    let start_i = *i;

    loop {
        match symbols.get(*i) {
            Some(Spanned { node: c, span }) if !c.is_whitespace() => {
                mnemonic.push(*c);
                oe = *span;
                *i += 1;
            }
            _ => {
                break;
            }
        }
    }

    let instruction_kind = match from_mnemonic(&mnemonic.chars().take(3).collect::<String>()) {
        Some(instruction_kind) => instruction_kind,
        None => {
            return Err(Error::InstructionInvalid {
                instruction: mnemonic,
                span: Span::combine(&os, &oe),
            })
        }
    };
    *i = start_i + 3;

    let mut keep = false;
    let mut r#return = false;
    let mut short = false;

    loop {
        match symbols.get(*i) {
            // TODO: handle duplicate modes
            Some(Spanned { node: 'k', span }) => {
                keep = true;
                oe = *span;
                *i += 1;
            }
            Some(Spanned { node: 'r', span }) => {
                r#return = true;
                oe = *span;
                *i += 1;
            }
            Some(Spanned { node: '2', span }) => {
                short = true;
                oe = *span;
                *i += 1;
            }
            Some(Spanned { node: c, span }) if !c.is_whitespace() => {
                return Err(Error::InstructionModeInvalid {
                    instruction_mode: *c,
                    span: *span,
                });
            }
            _ => {
                return Ok(Instruction {
                    instruction_kind,
                    keep,
                    r#return,
                    short,
                }
                .spanning(Span::combine(&os, &oe)))
            }
        }
    }
}

fn from_mnemonic(s: &str) -> Option<InstructionKind> {
    match s {
        "BRK" => Some(InstructionKind::Break),
        "LIT" => Some(InstructionKind::Literal),
        "NOP" => Some(InstructionKind::NoOperation),
        "POP" => Some(InstructionKind::Pop),
        "DUP" => Some(InstructionKind::Duplicate),
        "SWP" => Some(InstructionKind::Swap),
        "OVR" => Some(InstructionKind::Over),
        "ROT" => Some(InstructionKind::Rotate),
        "EQU" => Some(InstructionKind::Equal),
        "NEQ" => Some(InstructionKind::NotEqual),
        "GTH" => Some(InstructionKind::GreaterThan),
        "LTH" => Some(InstructionKind::LesserThan),
        "JMP" => Some(InstructionKind::Jump),
        "JCN" => Some(InstructionKind::JumpCondition),
        "JSR" => Some(InstructionKind::JumpStash),
        "STH" => Some(InstructionKind::Stash),
        "LDZ" => Some(InstructionKind::LoadZeroPage),
        "STZ" => Some(InstructionKind::StoreZeroPage),
        "LDR" => Some(InstructionKind::LoadRelative),
        "STR" => Some(InstructionKind::StoreRelative),
        "LDA" => Some(InstructionKind::LoadAbsolute),
        "STA" => Some(InstructionKind::StoreAbsolute),
        "DEI" => Some(InstructionKind::DeviceIn),
        "DEO" => Some(InstructionKind::DeviceOut),
        "ADD" => Some(InstructionKind::Add),
        "SUB" => Some(InstructionKind::Subtract),
        "MUL" => Some(InstructionKind::Multiply),
        "DIV" => Some(InstructionKind::Divide),
        "AND" => Some(InstructionKind::And),
        "ORA" => Some(InstructionKind::Or),
        "EOR" => Some(InstructionKind::ExclusiveOr),
        "SFT" => Some(InstructionKind::Shift),
        _ => None,
    }
}

fn to_hex_digit(c: char) -> Option<usize> {
    match c {
        '0'..='9' => Some(c as usize - '0' as usize),
        'a'..='f' => Some(c as usize - 'a' as usize + 10),
        _ => None,
    }
}

fn is_hex_digit(c: char) -> bool {
    to_hex_digit(c).is_some()
}
