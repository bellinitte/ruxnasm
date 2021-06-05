use super::{Identifier, Token};
use super::{Span, Spanned, Spanning};
pub use crate::anomalies::{Error, Warning};
use crate::{Instruction, InstructionKind};
use std::fmt;

mod hex_number;

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

impl fmt::Debug for Evaluated {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fine { token, .. } => write!(f, "Fine {{ token: {:?} }}", token.node),
            Self::Faulty { .. } => write!(f, "Faulty"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Word {
    evaluated: Evaluated,
}

impl Word {
    pub fn new(symbols: &[Spanned<char>]) -> Self {
        debug_assert!({
            const WHITESPACES: [char; 6] = [' ', '\t', '\n', 0x0b as char, 0x0c as char, '\r'];

            let chars: Vec<char> = symbols.iter().map(|Spanned { node: ch, .. }| *ch).collect();
            WHITESPACES.iter().all(|ch| !chars.contains(ch))
        });

        let evaluated = match tokenize(symbols) {
            Ok((token, warnings)) => Evaluated::Fine { token, warnings },
            Err(error) => Evaluated::Faulty {
                errors: vec![error],
                warnings: Vec::new(),
            },
        };

        Self { evaluated }
    }

    pub fn get_token(&self) -> Result<(Spanned<Token>, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
        match &self.evaluated {
            Evaluated::Fine { token, warnings } => Ok((token.clone(), warnings.clone())),
            Evaluated::Faulty { errors, warnings } => Err((errors.clone(), warnings.clone())),
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
        Spanned { node: '|', span } => hex_number::parse_hex_number_unconstrained(&word[1..])
            .map_err(|err| match err {
                hex_number::Error2::DigitExpected => Error::HexNumberExpected { span },
                hex_number::Error2::DigitInvalid { digit, span } => Error::HexDigitInvalid {
                    digit,
                    number: to_string(&word[1..]),
                    span,
                },
            })
            .map(|value| Token::PadAbsolute(value))
            .map(|token| (token.spanning(to_span(word).unwrap()), Vec::new())),
        Spanned { node: '$', span } => hex_number::parse_hex_number_unconstrained(&word[1..])
            .map_err(|err| match err {
                hex_number::Error2::DigitExpected => Error::HexNumberExpected { span },
                hex_number::Error2::DigitInvalid { digit, span } => Error::HexDigitInvalid {
                    digit,
                    number: to_string(&word[1..]),
                    span,
                },
            })
            .map(|value| Token::PadRelative(value))
            .map(|token| (token.spanning(to_span(word).unwrap()), Vec::new())),
        Spanned { node: '@', span } => {
            if !word[1..].is_empty() {
                if word[1].node != '&' {
                    if let Some(position) = word[1..]
                        .iter()
                        .map(|Spanned { node: ch, .. }| *ch)
                        .position(|c| c == '/')
                    {
                        Err(Error::SlashInLabelOrSublabel {
                            span: word[1 + position].span,
                        })
                    } else {
                        Ok((
                            Token::LabelDefine(to_string(&word[1..]))
                                .spanning(to_span(word).unwrap()),
                            Vec::new(),
                        ))
                    }
                } else {
                    Err(Error::AmpersandAtTheStartOfLabel { span: word[1].span })
                }
            } else {
                Err(Error::LabelExpected { span })
            }
        }
        Spanned { node: '&', span } => {
            if !word[1..].is_empty() {
                if let Some(position) = word[1..]
                    .iter()
                    .map(|Spanned { node: ch, .. }| *ch)
                    .position(|c| c == '/')
                {
                    Err(Error::SlashInLabelOrSublabel {
                        span: word[1 + position].span,
                    })
                } else {
                    Ok((
                        Token::SublabelDefine(to_string(&word[1..]))
                            .spanning(to_span(word).unwrap()),
                        Vec::new(),
                    ))
                }
            } else {
                Err(Error::LabelExpected { span })
            }
        }
        Spanned { node: '#', span } => match hex_number::parse_hex_number(&word[1..]) {
            Ok(hex_number::HexNumber::Byte(value)) => Ok(Token::LiteralHexByte(value)),
            Ok(hex_number::HexNumber::Short(value)) => Ok(Token::LiteralHexShort(value)),
            Err(hex_number::Error::DigitExpected) => {
                Err(Error::HexNumberOrCharacterExpected { span })
            }
            Err(hex_number::Error::DigitInvalid { digit, span }) => Err(Error::HexDigitInvalid {
                digit,
                number: to_string(&word[1..]),
                span,
            }),
            Err(hex_number::Error::UnevenLength { length: 1 }) => {
                Ok(Token::LiteralHexByte(word[1].node as u8))
            }
            Err(hex_number::Error::UnevenLength { length }) => Err(Error::HexNumberUnevenLength {
                length,
                number: to_string(&word[1..]),
                span: to_span(&word[1..]).unwrap(),
            }),
            Err(hex_number::Error::TooLong { length }) => Err(Error::HexNumberTooLong {
                length,
                number: to_string(&word[1..]),
                span: to_span(&word[1..]).unwrap(),
            }),
        }
        .map(|token| (token.spanning(to_span(word).unwrap()), Vec::new())),
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
        Spanned { node: '\'', span } => {
            let bytes: Vec<u8> = to_string(&word[1..]).bytes().collect();
            match bytes.len() {
                0 => Err(Error::CharacterExpected { span }),
                1 => Ok((
                    Token::RawChar(bytes[0]).spanning(Span::combine(&span, &word[1].span)),
                    Vec::new(),
                )),
                _ => {
                    let span = to_span(&word[1..]).unwrap();
                    Err(Error::MoreThanOneByteFound { bytes, span })
                }
            }
        }
        Spanned { node: '"', span } => {
            return Ok((
                Token::RawWord(to_string(&word[1..])).spanning(span),
                Vec::new(),
            ));
        }
        _ => {
            if let Ok(hex_number) = hex_number::parse_hex_number(word) {
                return Ok((
                    match hex_number {
                        hex_number::HexNumber::Byte(value) => Token::RawHexByte(value),
                        hex_number::HexNumber::Short(value) => Token::RawHexShort(value),
                    }
                    .spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
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
                    .map(|s| (Token::MacroExpand(s))),
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

    if let Ok(_) = hex_number::parse_hex_number(symbols) {
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

fn parse_identifier(rune_span: Span, symbols: &[Spanned<char>]) -> Result<Identifier, Error> {
    if symbols.is_empty() {
        return Err(Error::IdentifierExpected { span: rune_span });
    }

    if let Some(Spanned { node: '&', span }) = symbols.first() {
        let rune_span = Span::combine(&rune_span, &span);
        if symbols[1..].is_empty() {
            return Err(Error::SublabelExpected { span: rune_span });
        }
        return Ok(Identifier::Sublabel(to_string(&symbols[1..])));
    }

    match symbols
        .iter()
        .map(|Spanned { node: ch, .. }| *ch)
        .position(|c| c == '/')
    {
        Some(position) => {
            if let Some(second_position) = symbols[position + 1..]
                .iter()
                .map(|Spanned { node: ch, .. }| *ch)
                .position(|c| c == '/')
            {
                return Err(Error::MoreThanOneSlashInIdentifier {
                    span: symbols[position + 1 + second_position].span,
                });
            }

            let label = {
                let label_symbols = &symbols[..position];
                if label_symbols.is_empty() {
                    return Err(Error::LabelExpected { span: rune_span });
                }
                to_string(label_symbols)
            };
            let sublabel = {
                let sublabel_symbols = &symbols[position + 1..];
                if sublabel_symbols.is_empty() {
                    return Err(Error::SublabelExpected {
                        span: symbols[position].span,
                    });
                }
                to_string(sublabel_symbols)
            };
            Ok(Identifier::Path(label, sublabel))
        }
        None => {
            if symbols.is_empty() {
                return Err(Error::LabelExpected { span: rune_span });
            }
            Ok(Identifier::Label(to_string(symbols)))
        }
    }
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
