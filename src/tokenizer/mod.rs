use crate::{Instruction, InstructionKind};

use super::Token;
use super::{Location, Span, Spanned, Spanning};
pub use error::Error;

mod error;

pub fn tokenize(input_file_contents: &str) -> Result<Vec<Spanned<Token>>, Vec<Error>> {
    let symbols = scan(input_file_contents);
    let mut symbols = symbols.as_slice();

    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    'tokens: loop {
        'gap: loop {
            match symbols.first() {
                Some(Spanned { node: c, .. }) if c.is_whitespace() => {
                    symbols = &symbols[1..];
                }
                Some(Spanned {
                    node: '[' | ']', ..
                }) => {
                    symbols = &symbols[1..];
                }
                Some(Spanned { node: '(', .. }) => {
                    symbols = &symbols[1..];
                    let mut comment = true;
                    while comment {
                        match symbols.first() {
                            Some(Spanned { node: ')', .. }) => {
                                symbols = &symbols[1..];
                                comment = false;
                            }
                            Some(_) => {
                                symbols = &symbols[1..];
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
                symbols = &symbols[1..];
                tokens.push(Token::OpeningBrace.spanning(*span));
            }
            Some(Spanned { node: '}', span }) => {
                symbols = &symbols[1..];
                tokens.push(Token::ClosingBrace.spanning(*span));
            }
            Some(Spanned { node: '%', span }) => match parse_identifier(*span, &symbols[1..]) {
                Ok((Spanned { node, span }, new_symbols)) => {
                    tokens.push(Token::MacroDefine(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: '|', span }) => match parse_hex_number(*span, &symbols[1..]) {
                Ok((Spanned { node, span }, new_symbols)) => {
                    tokens.push(Token::PadAbsolute(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: '$', span }) => match parse_hex_number(*span, &symbols[1..]) {
                Ok((Spanned { node, span }, new_symbols)) => {
                    tokens.push(Token::PadRelative(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: '@', span }) => match parse_identifier(*span, &symbols[1..]) {
                Ok((Spanned { node, span }, new_symbols)) => {
                    tokens.push(Token::LabelDefine(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: '&', span }) => match parse_identifier(*span, &symbols[1..]) {
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
                symbols = &symbols[1..];
                match peek_word(symbols).len() {
                    0 => errors.push(Error::HexNumberExpected { span: *span }),
                    1 => {
                        let Spanned {
                            node: ch,
                            span: other_span,
                        } = symbols.first().unwrap();
                        tokens.push(
                            Token::LiteralHexByte(*ch as u8)
                                .spanning(Span::combine(span, other_span)),
                        );
                        symbols = &symbols[1..];
                    }
                    2 => match parse_hex_number(*span, symbols) {
                        Ok((Spanned { node, span }, new_symbols)) => {
                            tokens.push(Token::LiteralHexByte(node as u8).spanning(span));
                            symbols = new_symbols;
                        }
                        Err((err, new_symbols)) => {
                            errors.push(err);
                            symbols = new_symbols;
                        }
                    },
                    3 => {
                        errors.push(Error::HexNumberUnevenLength {
                            span: Span::combine(span, &symbols[2].span),
                        });
                        symbols = &symbols[3..];
                    }
                    4 => match parse_hex_number(*span, symbols) {
                        Ok((Spanned { node, span }, new_symbols)) => {
                            tokens.push(Token::LiteralHexShort(node as u16).spanning(span));
                            symbols = new_symbols;
                        }
                        Err((err, new_symbols)) => {
                            errors.push(err);
                            symbols = new_symbols;
                        }
                    },
                    length => {
                        errors.push(Error::HexNumberTooLarge {
                            length,
                            span: Span::combine(span, &symbols[length - 1].span),
                        });
                        symbols = &symbols[length..];
                    }
                }
            }
            Some(Spanned { node: '.', span }) => match parse_identifier(*span, &symbols[1..]) {
                Ok((Spanned { node, span }, new_symbols)) => {
                    tokens.push(Token::LiteralZeroPageAddress(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: ',', span }) => match parse_identifier(*span, &symbols[1..]) {
                Ok((Spanned { node, span }, new_symbols)) => {
                    tokens.push(Token::LiteralRelativeAddress(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: ';', span }) => match parse_identifier(*span, &symbols[1..]) {
                Ok((Spanned { node, span }, new_symbols)) => {
                    tokens.push(Token::LiteralAbsoluteAddress(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            Some(Spanned { node: ':', span }) => match parse_identifier(*span, &symbols[1..]) {
                Ok((Spanned { node, span }, new_symbols)) => {
                    tokens.push(Token::RawAddress(node).spanning(span));
                    symbols = new_symbols;
                }
                Err((err, new_symbols)) => {
                    errors.push(err);
                    symbols = new_symbols;
                }
            },
            // TODO: what happens when there are several characters after the '?
            Some(Spanned { node: '\'', span }) => {
                symbols = &symbols[1..];
                match symbols.first() {
                    Some(Spanned { node, span }) => {
                        tokens.push(Token::RawChar(*node as u8).spanning(*span));
                        symbols = &symbols[1..];
                    }
                    None => {
                        errors.push(Error::CharacterExpected { span: *span });
                        symbols = &symbols[1..];
                    }
                }
            }
            Some(Spanned { node: '"', span }) => {
                let (Spanned { node: word, span }, new_symbols) = parse_word(*span, &symbols[1..]);
                tokens.push(Token::RawWord(word.chars().map(|c| c as u8).collect()).spanning(span));
                symbols = new_symbols;
            }
            Some(Spanned { node: c, span }) if is_hex_digit(*c) => match peek_word(symbols).len() {
                0 => unreachable!(),
                length if length == 1 || length == 3 => {
                    errors.push(Error::HexNumberUnevenLength {
                        span: Span::combine(span, &symbols[length - 1].span),
                    });
                    symbols = &symbols[length..];
                }
                2 => match parse_hex_number(*span, symbols) {
                    Ok((Spanned { node, span }, new_symbols)) => {
                        tokens.push(Token::RawHexByte(node as u8).spanning(span));
                        symbols = new_symbols;
                    }
                    Err((err, new_symbols)) => {
                        errors.push(err);
                        symbols = new_symbols;
                    }
                },
                4 => match parse_hex_number(*span, symbols) {
                    Ok((Spanned { node, span }, new_symbols)) => {
                        tokens.push(Token::RawHexShort(node as u16).spanning(span));
                        symbols = new_symbols;
                    }
                    Err((err, new_symbols)) => {
                        errors.push(err);
                        symbols = new_symbols;
                    }
                },
                length => {
                    errors.push(Error::HexNumberTooLarge {
                        length,
                        span: Span::combine(span, &symbols[length - 1].span),
                    });
                    symbols = &symbols[length..];
                }
            },
            Some(_) => match parse_instruction(&symbols) {
                Ok((
                    Spanned {
                        node: instruction,
                        span,
                    },
                    new_symbols,
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

fn peek_word(symbols: &[Spanned<char>]) -> &[Spanned<char>] {
    let mut i = 0;
    loop {
        match symbols.get(i) {
            Some(Spanned { node: c, .. }) if !c.is_whitespace() => {
                i += 1;
            }
            _ => {
                return &symbols[..i];
            }
        }
    }
}

fn parse_word(first_span: Span, symbols: &[Spanned<char>]) -> (Spanned<String>, &[Spanned<char>]) {
    let word = peek_word(symbols);
    let length = word.len();
    let last_span = word
        .last()
        .map(|Spanned { span, .. }| *span)
        .unwrap_or(first_span);
    let word: String = word
        .into_iter()
        .map(|Spanned { node: ch, .. }| *ch)
        .collect();
    let word = word.spanning(Span::combine(&first_span, &last_span));
    (word, &symbols[length..])
}

fn parse_identifier(
    first_span: Span,
    symbols: &[Spanned<char>],
) -> Result<(Spanned<String>, &[Spanned<char>]), (Error, &[Spanned<char>])> {
    let word = peek_word(symbols);
    let length = word.len();

    if word.is_empty() {
        return Err((Error::IdentifierExpected { span: first_span }, symbols));
    }

    if let Ok((Spanned { span, .. }, symbols)) = parse_hex_number(first_span, symbols) {
        return Err((Error::IdentifierCannotBeAHexNumber { span }, symbols));
    }

    if let Ok((Spanned { span, .. }, symbols)) = parse_instruction(symbols) {
        return Err((Error::IdentifierCannotBeAnInstructon { span }, symbols));
    }

    let identifier: String = word
        .into_iter()
        .map(|Spanned { node: ch, .. }| *ch)
        .collect();
    let first_span = word.first().unwrap().span;
    let last_span = word.last().unwrap().span;
    let identifier = identifier.spanning(Span::combine(&first_span, &last_span));

    return Ok((identifier, &symbols[length..]));
}

fn parse_hex_number(
    first_span: Span,
    symbols: &[Spanned<char>],
) -> Result<(Spanned<usize>, &[Spanned<char>]), (Error, &[Spanned<char>])> {
    let mut word = peek_word(symbols);
    let length = word.len();

    if word.is_empty() {
        return Err((Error::HexNumberExpected { span: first_span }, symbols));
    }

    let first_span = word.first().unwrap().span;
    let last_span = word.last().unwrap().span;

    let mut value: usize = 0;

    for Spanned { node: ch, span } in word {
        if is_hex_digit(*ch) {
            value = (value << 4) + to_hex_digit(*ch).unwrap() as usize;
            word = &word[1..];
        } else {
            return Err((
                Error::HexDigitInvalid {
                    digit: *ch,
                    span: *span,
                },
                &symbols[length..],
            ));
        }
    }

    let hex_number = value.spanning(Span::combine(&first_span, &last_span));
    return Ok((hex_number, &symbols[length..]));
}

/// `symbols` must not be empty.
fn parse_instruction(
    symbols: &[Spanned<char>],
) -> Result<(Spanned<Instruction>, &[Spanned<char>]), (Error, &[Spanned<char>])> {
    fn split_uppercase_prefix(symbols: &[Spanned<char>]) -> (&[Spanned<char>], &[Spanned<char>]) {
        let mut i: usize = 0;
        loop {
            match symbols.get(i) {
                Some(Spanned { node: ch, .. }) if ch.is_uppercase() => {
                    i += 1;
                }
                _ => return (&symbols[..i], &symbols[i..]),
            }
        }
    }

    fn from_mnemonic(symbols: &[Spanned<char>]) -> Option<InstructionKind> {
        if symbols.len() != 3 {
            return None;
        }

        let string = symbols
            .into_iter()
            .map(|Spanned { node: ch, .. }| *ch)
            .collect::<String>();

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
        let instruction_string: String = word
            .into_iter()
            .map(|Spanned { node: ch, .. }| *ch)
            .collect();
        let first_span = word.first().unwrap().span;
        let last_span = word.last().unwrap().span;
        let span = Span::combine(&first_span, &last_span);

        (
            Error::InstructionInvalid {
                instruction: instruction_string,
                span,
            },
            &symbols[length..],
        )
    })?;

    let mut keep: Option<Span> = None;
    let mut r#return: Option<Span> = None;
    let mut short: Option<Span> = None;

    for Spanned { node: ch, span } in modes {
        match *ch {
            'k' => {
                if let Some(other_span) = keep {
                    return Err((
                        Error::InstructionModeDefinedMoreThanOnce {
                            instruction_mode: 'k',
                            span: *span,
                            other_span,
                        },
                        &symbols[length..],
                    ));
                } else {
                    keep = Some(*span);
                }
            }
            'r' => {
                if let Some(other_span) = r#return {
                    return Err((
                        Error::InstructionModeDefinedMoreThanOnce {
                            instruction_mode: 'r',
                            span: *span,
                            other_span,
                        },
                        &symbols[length..],
                    ));
                } else {
                    r#return = Some(*span);
                }
            }
            '2' => {
                if let Some(other_span) = short {
                    return Err((
                        Error::InstructionModeDefinedMoreThanOnce {
                            instruction_mode: '2',
                            span: *span,
                            other_span,
                        },
                        &symbols[length..],
                    ));
                } else {
                    short = Some(*span);
                }
            }
            instruction_mode => {
                return Err((
                    Error::InstructionModeInvalid {
                        instruction_mode,
                        span: *span,
                    },
                    &symbols[length..],
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

    return Ok((instruction, &symbols[length..]));
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
