use super::{Identifier, Token};
use super::{Span, Spanned, Spanning};
use crate::{Instruction, InstructionKind};
pub use anomalies::{Error, Warning};

mod anomalies;

#[derive(Clone)]
enum WordStatus {
    Pending { symbols: Vec<Spanned<char>> },
    Evaluated(Evaluated),
}

#[derive(Clone)]
enum Evaluated {
    Fine {
        token: Spanned<Token>,
        warnings: Vec<Warning>,
    },
    Faulty {
        errors: Vec<Error>,
        warnings: Vec<Warning>,
    },
}

#[derive(Clone)]
pub struct Word {
    status: WordStatus,
}

impl Word {
    pub fn new(symbols: &[Spanned<char>]) -> Self {
        debug_assert!({
            const WHITESPACES: [char; 6] = [' ', '\t', '\n', 0x0b as char, 0x0c as char, '\r'];

            let chars: Vec<char> = symbols.iter().map(|Spanned { node: ch, .. }| *ch).collect();
            WHITESPACES.iter().all(|ch| !chars.contains(ch))
        });

        Self {
            status: WordStatus::Pending {
                symbols: symbols.to_vec(),
            },
        }
    }

    pub fn get_token(
        &mut self,
    ) -> Result<(Spanned<Token>, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
        match &self.status {
            WordStatus::Evaluated(Evaluated::Fine { token, warnings }) => {
                Ok((token.clone(), warnings.clone()))
            }
            WordStatus::Evaluated(Evaluated::Faulty { errors, warnings }) => {
                Err((errors.clone(), warnings.clone()))
            }
            WordStatus::Pending { symbols } => match tokenize(symbols) {
                Ok((token, warnings)) => {
                    self.status = WordStatus::Evaluated(Evaluated::Fine {
                        token: token.clone(),
                        warnings: warnings.clone(),
                    });
                    Ok((token, warnings))
                }
                Err(error) => {
                    self.status = WordStatus::Evaluated(Evaluated::Faulty {
                        errors: vec![error.clone()],
                        warnings: Vec::new(),
                    });
                    Err((vec![error], Vec::new()))
                }
            },
        }
    }
}

fn tokenize(word: &[Spanned<char>]) -> Result<(Spanned<Token>, Vec<Warning>), Error> {
    match word.first().cloned().unwrap() {
        Spanned { node: '[', span } => {
            return Ok((Token::OpeningBracket.spanning(span), Vec::new()))
        }
        Spanned { node: ']', span } => {
            return Ok((Token::ClosingBracket.spanning(span), Vec::new()))
        }
        Spanned { node: '{', span } => return Ok((Token::OpeningBrace.spanning(span), Vec::new())),
        Spanned { node: '}', span } => return Ok((Token::ClosingBrace.spanning(span), Vec::new())),
        Spanned { node: '%', span } => match parse_macro(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::MacroDefine(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: '|', .. } => match parse_hex_number(&word[1..]) {
            Ok(value) => {
                return Ok((
                    Token::PadAbsolute(value as u16).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ))
            }
            Err(err) => Err(err),
        },
        Spanned { node: '$', .. } => match parse_hex_number(&word[1..]) {
            Ok(value) => {
                return Ok((
                    Token::PadRelative(value as u16).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ))
            }
            Err(err) => Err(err),
        },
        Spanned { node: '@', span } => match parse_label(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::LabelDefine(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: '&', span } => match parse_sublabel(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::SublabelDefine(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: '#', .. } => match word[1..].len() {
            0 => Err(Error::HexNumberExpected {
                span: to_span(word).unwrap(),
            }),
            1 => {
                let Spanned { node: ch, .. } = word.get(1).unwrap();
                return Ok((
                    Token::LiteralHexByte(*ch as u8).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            2 => match parse_hex_number(&word[1..]) {
                Ok(x) => {
                    return Ok((
                        Token::LiteralHexByte(x as u8).spanning(to_span(word).unwrap()),
                        Vec::new(),
                    ));
                }
                Err(err) => Err(err),
            },
            3 => match parse_hex_number(&word[1..]) {
                Ok(_) => Err(Error::HexNumberUnevenLength {
                    length: 3,
                    number: to_string(&word[1..]),
                    span: to_span(&word[1..]).unwrap(),
                }),
                Err(err) => Err(err),
            },
            4 => match parse_hex_number(&word[1..]) {
                Ok(x) => {
                    return Ok((
                        Token::LiteralHexShort(x as u16).spanning(to_span(word).unwrap()),
                        Vec::new(),
                    ));
                }
                Err(err) => Err(err),
            },
            length => match parse_hex_number(&word[1..]) {
                Ok(_) => Err(Error::HexNumberTooLong {
                    length,
                    number: to_string(&word[1..]),
                    span: to_span(&word[1..]).unwrap(),
                }),
                Err(err) => Err(err),
            },
        },
        Spanned { node: '.', span } => match parse_identifier(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::LiteralZeroPageAddress(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: ',', span } => match parse_identifier(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::LiteralRelativeAddress(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: ';', span } => match parse_identifier(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::LiteralAbsoluteAddress(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: ':', span } => match parse_identifier(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::RawAddress(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: '\'', span } => match word.get(1) {
            Some(spanned) => {
                return Ok((spanned.clone().map(|c| Token::RawChar(c as u8)), Vec::new()));
            }
            None => return Ok((Token::RawChar(0x00).spanning(span), Vec::new())),
        },
        Spanned { node: '"', span } => {
            return Ok((
                Token::RawWord(to_string(&word[1..])).spanning(span),
                Vec::new(),
            ));
        }
        _ => {
            if let Ok(x) = parse_hex_number(word) {
                match word.len() {
                    0 => unreachable!(),
                    length if length == 1 || length == 3 => {
                        return Err(Error::HexNumberUnevenLength {
                            length,
                            number: to_string(word),
                            span: to_span(word).unwrap(),
                        });
                    }
                    2 => {
                        return Ok((
                            Token::RawHexByte(x as u8).spanning(to_span(word).unwrap()),
                            Vec::new(),
                        ));
                    }
                    4 => {
                        return Ok((
                            Token::RawHexShort(x as u16).spanning(to_span(word).unwrap()),
                            Vec::new(),
                        ));
                    }
                    length => {
                        return Err(Error::HexNumberTooLong {
                            length,
                            number: to_string(word),
                            span: to_span(word).unwrap(),
                        });
                    }
                }
            };
            if let Some((instruction, new_warnings)) = parse_instruction(word) {
                return Ok((
                    Token::Instruction(instruction).spanning(to_span(word).unwrap()),
                    new_warnings,
                ));
            };
            return Ok((
                to_spanned_string(word)
                    .unwrap()
                    .map(|s| (Token::MacroInvoke(s))),
                Vec::new(),
            ));
        }
    }
}

fn to_string(symbols: &[Spanned<char>]) -> String {
    symbols.iter().map(|Spanned { node: ch, .. }| *ch).collect()
}

fn to_span(symbols: &[Spanned<char>]) -> Option<Span> {
    Some(Span::combine(&symbols.first()?.span, &symbols.last()?.span))
}

fn to_spanned_string(symbols: &[Spanned<char>]) -> Option<Spanned<String>> {
    to_span(symbols).map(|span| to_string(symbols).spanning(span))
}

fn parse_macro(rune_span: Span, symbols: &[Spanned<char>]) -> Result<String, Error> {
    if symbols.is_empty() {
        return Err(Error::MacroNameExpected { span: rune_span });
    }

    if let Ok(_) = parse_hex_number(symbols) {
        return Err(Error::MacroCannotBeAHexNumber {
            span: to_span(symbols).unwrap(),
            number: to_string(symbols),
        });
    }
    if let Some(_) = parse_instruction(symbols) {
        return Err(Error::MacroCannotBeAnInstruction {
            span: to_span(symbols).unwrap(),
            instruction: to_string(symbols),
        });
    }

    Ok(to_string(symbols))
}

fn parse_label(rune_span: Span, symbols: &[Spanned<char>]) -> Result<String, Error> {
    if symbols.is_empty() {
        return Err(Error::LabelExpected { span: rune_span });
    }

    Ok(to_string(symbols))
}

fn parse_sublabel(rune_span: Span, symbols: &[Spanned<char>]) -> Result<String, Error> {
    if symbols.is_empty() {
        return Err(Error::LabelExpected { span: rune_span });
    }

    Ok(to_string(symbols))
}

fn parse_identifier(rune_span: Span, symbols: &[Spanned<char>]) -> Result<Identifier, Error> {
    if symbols.is_empty() {
        return Err(Error::IdentifierExpected { span: rune_span });
    }

    if let Some(Spanned { node: '&', span }) = symbols.first() {
        let rune_span = Span::combine(&rune_span, &span);
        return Ok(Identifier::Sublabel(parse_sublabel(
            rune_span,
            &symbols[1..],
        )?));
    }

    match symbols
        .iter()
        .map(|Spanned { node: ch, .. }| *ch)
        .position(|c| c == '/')
    {
        Some(position) => {
            let label_symbols = &symbols[..position];
            let sublabel_symbols = &symbols[position + 1..];
            let label = parse_label(rune_span, label_symbols)?;
            let sublabel = parse_sublabel(symbols[position].span, sublabel_symbols)?;
            Ok(Identifier::Path(label, sublabel))
        }
        None => Ok(Identifier::Label(parse_label(rune_span, symbols)?)),
    }
}

fn parse_hex_number(symbols: &[Spanned<char>]) -> Result<usize, Error> {
    if symbols.is_empty() {
        return Ok(0);
    }

    let mut value: usize = 0;

    for Spanned { node: ch, span } in symbols {
        if is_hex_digit(*ch) {
            value = (value << 4) + to_hex_digit(*ch).unwrap() as usize;
        } else {
            return Err(Error::HexDigitInvalid {
                digit: *ch,
                number: to_string(symbols),
                span: *span,
            });
        }
    }

    Ok(value)
}

/// `symbols` must not be empty.
fn parse_instruction(symbols: &[Spanned<char>]) -> Option<(Instruction, Vec<Warning>)> {
    if symbols.len() < 3 {
        return None;
    }

    let instruction_kind = match to_string(&symbols[..3]).as_str() {
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
    }?;

    let mut keep: Option<Span> = None;
    let mut r#return: Option<Span> = None;
    let mut short: Option<Span> = None;
    let mut warnings = Vec::new();

    for Spanned { node: ch, span } in &symbols[3..] {
        match ch {
            'k' => {
                if let Some(other_span) = keep {
                    warnings.push(Warning::InstructionModeDefinedMoreThanOnce {
                        instruction_mode: 'k',
                        instruction: to_string(&symbols[..3]),
                        span: *span,
                        other_span,
                    });
                }
                keep = Some(*span);
            }
            'r' => {
                if let Some(other_span) = r#return {
                    warnings.push(Warning::InstructionModeDefinedMoreThanOnce {
                        instruction_mode: 'r',
                        instruction: to_string(&symbols[..3]),
                        span: *span,
                        other_span,
                    });
                }
                r#return = Some(*span);
            }
            '2' => {
                if let Some(other_span) = short {
                    warnings.push(Warning::InstructionModeDefinedMoreThanOnce {
                        instruction_mode: '2',
                        instruction: to_string(&symbols[..3]),
                        span: *span,
                        other_span,
                    });
                }
                short = Some(*span);
            }
            _ => {
                return None;
            }
        }
    }

    return Some((
        Instruction {
            instruction_kind,
            keep: keep.is_some(),
            r#return: r#return.is_some(),
            short: short.is_some(),
        },
        warnings,
    ));
}

pub fn to_hex_digit(c: char) -> Option<usize> {
    match c {
        '0'..='9' => Some(c as usize - '0' as usize),
        'a'..='f' => Some(c as usize - 'a' as usize + 10),
        _ => None,
    }
}

pub fn is_hex_digit(c: char) -> bool {
    to_hex_digit(c).is_some()
}
