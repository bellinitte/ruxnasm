use super::{Breadcrumbs, Token};
use super::{Location, Span, Spanned, Spanning};
use crate::{Instruction, InstructionKind};
pub use error::Error;
use symbols::{Slice, Symbols};

mod error;
mod symbols;

pub fn tokenize(input_file_contents: &str) -> Result<Vec<Spanned<Token>>, Vec<Error>> {
    let symbols = scan(input_file_contents);
    let mut symbols: Symbols = symbols.as_slice().into();

    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    'tokens: loop {
        'gap: loop {
            match symbols.first() {
                Some(Spanned { node: c, .. }) if c.is_whitespace() => {
                    symbols = symbols.slice(1..);
                }
                Some(Spanned {
                    node: '[' | ']', ..
                }) => {
                    symbols = symbols.slice(1..);
                }
                Some(Spanned { node: '(', .. }) => {
                    symbols = symbols.slice(1..);
                    let mut comment = true;
                    while comment {
                        match symbols.first() {
                            Some(Spanned { node: ')', .. }) => {
                                symbols = symbols.slice(1..);
                                comment = false;
                            }
                            Some(_) => {
                                symbols = symbols.slice(1..);
                            }
                            None => break 'gap,
                        }
                    }
                }
                _ => break 'gap,
            };
        }
        match symbols.first() {
            Some(Spanned { node: '{', span }) => {
                symbols = symbols.slice(1..);
                tokens.push(Token::OpeningBrace.spanning(span));
            }
            Some(Spanned { node: '}', span }) => {
                symbols = symbols.slice(1..);
                tokens.push(Token::ClosingBrace.spanning(span));
            }
            Some(Spanned { node: '%', span }) => match parse_identifier(span, symbols.slice(1..)) {
                Ok((Spanned { node, span }, new_symbols)) => {
                    tokens.push(Token::MacroDefine(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: '|', span }) => match parse_hex_number(span, symbols.slice(1..)) {
                Ok((Spanned { node, span }, new_symbols, _)) => {
                    tokens.push(Token::PadAbsolute(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: '$', span }) => match parse_hex_number(span, symbols.slice(1..)) {
                Ok((Spanned { node, span }, new_symbols, _)) => {
                    tokens.push(Token::PadRelative(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: '@', span }) => match parse_identifier(span, symbols.slice(1..)) {
                Ok((Spanned { node, span }, new_symbols)) => {
                    tokens.push(Token::LabelDefine(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: '&', span }) => match parse_identifier(span, symbols.slice(1..)) {
                Ok((Spanned { node, span }, new_symbols)) => {
                    tokens.push(Token::SublabelDefine(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: '#', span }) => {
                symbols = symbols.slice(1..);
                match peek_word(symbols).len() {
                    0 => errors.push(Error::HexNumberExpected { span }),
                    1 => {
                        let Spanned {
                            node: ch,
                            span: other_span,
                        } = symbols.first().unwrap();
                        tokens.push(
                            Token::LiteralHexByte(ch as u8)
                                .spanning(Span::combine(&span, &other_span)),
                        );
                        symbols = symbols.slice(1..);
                    }
                    2 => match parse_hex_number(span, symbols) {
                        Ok((Spanned { node, span }, new_symbols, _)) => {
                            tokens.push(Token::LiteralHexByte(node as u8).spanning(span));
                            symbols = new_symbols;
                        }
                        Err((err, new_symbols)) => {
                            errors.push(err);
                            symbols = new_symbols;
                        }
                    },
                    3 => match parse_hex_number(span, symbols) {
                        Ok((Spanned { span, .. }, new_symbols, parsed)) => {
                            errors.push(Error::HexNumberUnevenLength {
                                length: 3,
                                number: parsed.to_string(),
                                span,
                            });
                            symbols = new_symbols;
                        }
                        Err((err, new_symbols)) => {
                            errors.push(err);
                            symbols = new_symbols;
                        }
                    },
                    4 => match parse_hex_number(span, symbols) {
                        Ok((Spanned { node, span }, new_symbols, _)) => {
                            tokens.push(Token::LiteralHexShort(node as u16).spanning(span));
                            symbols = new_symbols;
                        }
                        Err((err, new_symbols)) => {
                            errors.push(err);
                            symbols = new_symbols;
                        }
                    },
                    length => match parse_hex_number(span, symbols) {
                        Ok((Spanned { span, .. }, new_symbols, parsed)) => {
                            errors.push(Error::HexNumberTooLarge {
                                length,
                                number: parsed.to_string(),
                                span,
                            });
                            symbols = new_symbols;
                        }
                        Err((err, new_symbols)) => {
                            errors.push(err);
                            symbols = new_symbols;
                        }
                    },
                }
            }
            Some(Spanned { node: '.', span }) => {
                match parse_breadcrumbs(span, symbols.slice(1..)) {
                    Ok((Spanned { node, span }, new_symbols)) => {
                        tokens.push(Token::LiteralZeroPageAddress(node).spanning(span));
                        symbols = new_symbols;
                    }
                    Err((err, new_symbols)) => {
                        errors.push(err);
                        symbols = new_symbols;
                    }
                }
            }
            Some(Spanned { node: ',', span }) => {
                match parse_breadcrumbs(span, symbols.slice(1..)) {
                    Ok((Spanned { node, span }, new_symbols)) => {
                        tokens.push(Token::LiteralRelativeAddress(node).spanning(span));
                        symbols = new_symbols;
                    }
                    Err((err, new_symbols)) => {
                        errors.push(err);
                        symbols = new_symbols;
                    }
                }
            }
            Some(Spanned { node: ';', span }) => {
                match parse_breadcrumbs(span, symbols.slice(1..)) {
                    Ok((Spanned { node, span }, new_symbols)) => {
                        tokens.push(Token::LiteralAbsoluteAddress(node).spanning(span));
                        symbols = new_symbols;
                    }
                    Err((err, new_symbols)) => {
                        errors.push(err);
                        symbols = new_symbols;
                    }
                }
            }
            Some(Spanned { node: ':', span }) => {
                match parse_breadcrumbs(span, symbols.slice(1..)) {
                    Ok((Spanned { node, span }, new_symbols)) => {
                        tokens.push(Token::RawAddress(node).spanning(span));
                        symbols = new_symbols;
                    }
                    Err((err, new_symbols)) => {
                        errors.push(err);
                        symbols = new_symbols;
                    }
                }
            }
            // TODO: what happens when there are several characters after the '?
            Some(Spanned { node: '\'', span }) => {
                symbols = symbols.slice(1..);
                match symbols.first() {
                    Some(Spanned { node, span }) => {
                        tokens.push(Token::RawChar(node as u8).spanning(span));
                        symbols = symbols.slice(1..);
                    }
                    None => {
                        errors.push(Error::CharacterExpected { span: span });
                        symbols = symbols.slice(1..);
                    }
                }
            }
            Some(Spanned { node: '"', span }) => {
                let (Spanned { node: word, span }, new_symbols) =
                    parse_word(span, symbols.slice(1..));
                tokens.push(Token::RawWord(word.chars().map(|c| c as u8).collect()).spanning(span));
                symbols = new_symbols;
            }
            Some(Spanned { node: c, span }) if is_hex_digit(c) => match peek_word(symbols).len() {
                0 => unreachable!(),
                length if length == 1 || length == 3 => match parse_hex_number(span, symbols) {
                    Ok((Spanned { span, .. }, new_symbols, parsed)) => {
                        errors.push(Error::HexNumberUnevenLength {
                            length,
                            number: parsed.to_string(),
                            span,
                        });
                        symbols = new_symbols;
                    }
                    Err((err, new_symbols)) => {
                        errors.push(err);
                        symbols = new_symbols;
                    }
                },
                2 => match parse_hex_number(span, symbols) {
                    Ok((Spanned { node, span }, new_symbols, _)) => {
                        tokens.push(Token::RawHexByte(node as u8).spanning(span));
                        symbols = new_symbols;
                    }
                    Err((err, new_symbols)) => {
                        errors.push(err);
                        symbols = new_symbols;
                    }
                },
                4 => match parse_hex_number(span, symbols) {
                    Ok((Spanned { node, span }, new_symbols, _)) => {
                        tokens.push(Token::RawHexShort(node as u16).spanning(span));
                        symbols = new_symbols;
                    }
                    Err((err, new_symbols)) => {
                        errors.push(err);
                        symbols = new_symbols;
                    }
                },
                length => match parse_hex_number(span, symbols) {
                    Ok((Spanned { span, .. }, new_symbols, parsed)) => {
                        errors.push(Error::HexNumberTooLarge {
                            length,
                            number: parsed.to_string(),
                            span,
                        });
                        symbols = new_symbols;
                    }
                    Err((err, new_symbols)) => {
                        errors.push(err);
                        symbols = new_symbols;
                    }
                },
            },
            Some(_) => match parse_instruction(symbols) {
                Ok((
                    Spanned {
                        node: instruction,
                        span,
                    },
                    new_symbols,
                    _,
                )) => {
                    tokens.push(Token::Instruction(instruction).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
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
        .scan(Location { offset: 0 }, |location, ch| {
            let current_location = *location;
            (*location).offset += 1;
            Some(Spanned {
                node: ch,
                span: Span {
                    from: current_location,
                    to: *location,
                },
            })
        })
        .collect()
}

fn peek_word(symbols: Symbols) -> Symbols {
    let mut i = 0;
    loop {
        match symbols.get(i) {
            Some(Spanned { node: c, .. }) if !c.is_whitespace() => {
                i += 1;
            }
            _ => {
                return symbols.slice(..i);
            }
        }
    }
}

fn parse_word(first_span: Span, symbols: Symbols) -> (Spanned<String>, Symbols) {
    let word = peek_word(symbols);
    let length = word.len();
    let last_span = word
        .last()
        .map(|Spanned { span, .. }| span)
        .unwrap_or(first_span);
    let word: String = word.to_string();
    let word = word.spanning(Span::combine(&first_span, &last_span));
    (word, symbols.slice(length..))
}

fn parse_identifier(
    first_span: Span,
    symbols: Symbols,
) -> Result<(Spanned<String>, Symbols), (Error, Symbols)> {
    let word = peek_word(symbols);
    let length = word.len();

    if word.is_empty() {
        return Err((Error::IdentifierExpected { span: first_span }, symbols));
    }

    if let Ok((Spanned { span, .. }, symbols, parsed)) = parse_hex_number(first_span, symbols) {
        return Err((
            Error::IdentifierCannotBeAHexNumber {
                span,
                number: parsed.to_string(),
            },
            symbols,
        ));
    }

    if let Ok((Spanned { span, .. }, symbols, parsed)) = parse_instruction(symbols) {
        return Err((
            Error::IdentifierCannotBeAnInstruction {
                span,
                instruction: parsed.to_string(),
            },
            symbols,
        ));
    }

    let identifier: String = word.to_string();
    let first_span = word.first().unwrap().span;
    let last_span = word.last().unwrap().span;
    let identifier = identifier.spanning(Span::combine(&first_span, &last_span));

    return Ok((identifier, symbols.slice(length..)));
}

fn parse_breadcrumbs(
    first_span: Span,
    symbols: Symbols,
) -> Result<(Spanned<Breadcrumbs>, Symbols), (Error, Symbols)> {
    todo!()
}

fn parse_hex_number(
    first_span: Span,
    symbols: Symbols,
) -> Result<(Spanned<usize>, Symbols, Symbols), (Error, Symbols)> {
    let mut word = peek_word(symbols);
    let length = word.len();

    if word.is_empty() {
        return Err((Error::HexNumberExpected { span: first_span }, symbols));
    }

    let first_span = word.first().unwrap().span;
    let last_span = word.last().unwrap().span;

    let mut value: usize = 0;

    for Spanned { node: ch, span } in word {
        if is_hex_digit(ch) {
            value = (value << 4) + to_hex_digit(ch).unwrap() as usize;
            word = word.slice(1..);
        } else {
            return Err((
                Error::HexDigitInvalid {
                    digit: ch,
                    number: word.to_string(),
                    span,
                },
                symbols.slice(length..),
            ));
        }
    }

    let hex_number = value.spanning(Span::combine(&first_span, &last_span));
    return Ok((hex_number, symbols.slice(length..), symbols.slice(..length)));
}

/// `symbols` must not be empty.
fn parse_instruction(
    symbols: Symbols,
) -> Result<(Spanned<Instruction>, Symbols, Symbols), (Error, Symbols)> {
    fn split_uppercase_prefix(symbols: Symbols) -> (Symbols, Symbols) {
        let mut i: usize = 0;
        loop {
            match symbols.get(i) {
                Some(Spanned { node: ch, .. }) if ch.is_uppercase() => {
                    i += 1;
                }
                _ => return (symbols.slice(..i), symbols.slice(i..)),
            }
        }
    }

    fn from_mnemonic(symbols: Symbols) -> Option<InstructionKind> {
        if symbols.len() != 3 {
            return None;
        }

        let string = symbols.to_string();

        let instruction_kind = match string.as_str() {
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
        };

        return instruction_kind;
    }

    let word = peek_word(symbols);
    let length = word.len();

    let (mnemonic, modes) = split_uppercase_prefix(word);

    let instruction_kind = from_mnemonic(mnemonic).ok_or_else(|| {
        let instruction_string: String = word.to_string();
        let first_span = word.first().unwrap().span;
        let last_span = word.last().unwrap().span;
        let span = Span::combine(&first_span, &last_span);

        (
            Error::InstructionInvalid {
                instruction: instruction_string,
                span,
            },
            symbols.slice(length..),
        )
    })?;

    let mut keep: Option<Span> = None;
    let mut r#return: Option<Span> = None;
    let mut short: Option<Span> = None;

    for Spanned { node: ch, span } in modes {
        match ch {
            'k' => {
                if let Some(other_span) = keep {
                    return Err((
                        Error::InstructionModeDefinedMoreThanOnce {
                            instruction_mode: 'k',
                            instruction: word.to_string(),
                            span,
                            other_span,
                        },
                        symbols.slice(length..),
                    ));
                } else {
                    keep = Some(span);
                }
            }
            'r' => {
                if let Some(other_span) = r#return {
                    return Err((
                        Error::InstructionModeDefinedMoreThanOnce {
                            instruction_mode: 'r',
                            instruction: word.to_string(),
                            span,
                            other_span,
                        },
                        symbols.slice(length..),
                    ));
                } else {
                    r#return = Some(span);
                }
            }
            '2' => {
                if let Some(other_span) = short {
                    return Err((
                        Error::InstructionModeDefinedMoreThanOnce {
                            instruction_mode: '2',
                            instruction: word.to_string(),
                            span,
                            other_span,
                        },
                        symbols.slice(length..),
                    ));
                } else {
                    short = Some(span);
                }
            }
            instruction_mode => {
                return Err((
                    Error::InstructionModeInvalid {
                        instruction_mode,
                        instruction: word.to_string(),
                        span,
                    },
                    symbols.slice(length..),
                ));
            }
        }
    }

    let keep = keep.is_some();
    let r#return = r#return.is_some();
    let short = short.is_some();

    let first_span = word.first().unwrap().span;
    let last_span = word.last().unwrap().span;
    let span = Span::combine(&first_span, &last_span);

    let instruction = Instruction {
        instruction_kind,
        keep,
        r#return,
        short,
    }
    .spanning(span);

    return Ok((
        instruction,
        symbols.slice(length..),
        symbols.slice(..length),
    ));
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
