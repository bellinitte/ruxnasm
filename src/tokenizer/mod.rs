use super::{Identifier, Token};
use super::{Span, Spanned, Spanning};
use crate::anomalies::{Error, Warning};
use crate::{Instruction, InstructionKind};
use std::fmt;

mod hex_number;

#[derive(Clone)]
pub(crate) enum Word {
    Fine {
        token: Spanned<Token>,
        warnings: Vec<Warning>,
    },
    Faulty {
        errors: Vec<Error>,
        warnings: Vec<Warning>,
    },
}

impl Word {
    pub(crate) fn new(symbols: &[Spanned<u8>]) -> Self {
        debug_assert!({
            const WHITESPACES: [u8; 6] = [b' ', b'\t', b'\n', 0x0b, 0x0c, b'\r'];

            let chars: Vec<u8> = symbols.iter().map(|Spanned { node: ch, .. }| *ch).collect();
            WHITESPACES.iter().all(|ch| !chars.contains(ch))
        });

        match tokenize(symbols) {
            Ok((token, warnings)) => Self::Fine { token, warnings },
            Err(error) => Self::Faulty {
                errors: vec![error],
                warnings: Vec::new(),
            },
        }
    }
}

fn tokenize(word: &[Spanned<u8>]) -> Result<(Spanned<Token>, Vec<Warning>), Error> {
    match word.first().cloned().unwrap() {
        Spanned { node: b'[', span } => {
            return Ok((Token::OpeningBracket.spanning(span), Vec::new()))
        }
        Spanned { node: b']', span } => {
            return Ok((Token::ClosingBracket.spanning(span), Vec::new()))
        }
        Spanned { node: b'{', span } => {
            return Ok((Token::OpeningBrace.spanning(span), Vec::new()))
        }
        Spanned { node: b'}', span } => {
            return Ok((Token::ClosingBrace.spanning(span), Vec::new()))
        }
        Spanned { node: b'%', span } => match parse_macro(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::MacroDefine(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: b'|', span } => hex_number::parse_hex_number_unconstrained(&word[1..])
            .map_err(|err| match err {
                hex_number::Error2::DigitExpected => Error::HexNumberExpected { span: span.into() },
                hex_number::Error2::DigitInvalid { digit, span } => Error::HexDigitInvalid {
                    digit,
                    number: String::from_utf8_lossy(&to_string(&word[1..])).into_owned(),
                    span: span.into(),
                },
                hex_number::Error2::TooLong { length } => Error::HexNumberTooLong {
                    length,
                    number: String::from_utf8_lossy(&to_string(&word[1..])).into_owned(),
                    span: to_span(&word[1..]).unwrap().into(),
                },
            })
            .map(|value| Token::PadAbsolute(value))
            .map(|token| (token.spanning(to_span(word).unwrap()), Vec::new())),
        Spanned { node: b'$', span } => hex_number::parse_hex_number_unconstrained(&word[1..])
            .map_err(|err| match err {
                hex_number::Error2::DigitExpected => Error::HexNumberExpected { span: span.into() },
                hex_number::Error2::DigitInvalid { digit, span } => Error::HexDigitInvalid {
                    digit,
                    number: String::from_utf8_lossy(&to_string(&word[1..])).into_owned(),
                    span: span.into(),
                },
                hex_number::Error2::TooLong { length } => Error::HexNumberTooLong {
                    length,
                    number: String::from_utf8_lossy(&to_string(&word[1..])).into_owned(),
                    span: to_span(&word[1..]).unwrap().into(),
                },
            })
            .map(|value| Token::PadRelative(value))
            .map(|token| (token.spanning(to_span(word).unwrap()), Vec::new())),
        Spanned { node: b'@', span } => {
            if !word[1..].is_empty() {
                if word[1].node != b'&' {
                    if let Some(position) = word[1..]
                        .iter()
                        .map(|Spanned { node: ch, .. }| *ch)
                        .position(|c| c == b'/')
                    {
                        Err(Error::SlashInLabelOrSublabel {
                            span: word[1 + position].span.into(),
                        })
                    } else {
                        Ok((
                            Token::LabelDefine(to_string(&word[1..]))
                                .spanning(to_span(word).unwrap()),
                            Vec::new(),
                        ))
                    }
                } else {
                    Err(Error::AmpersandAtTheStartOfLabel {
                        span: word[1].span.into(),
                    })
                }
            } else {
                Err(Error::LabelExpected { span: span.into() })
            }
        }
        Spanned { node: b'&', span } => {
            if !word[1..].is_empty() {
                if let Some(position) = word[1..]
                    .iter()
                    .map(|Spanned { node: ch, .. }| *ch)
                    .position(|c| c == b'/')
                {
                    Err(Error::SlashInLabelOrSublabel {
                        span: word[1 + position].span.into(),
                    })
                } else {
                    Ok((
                        Token::SublabelDefine(to_string(&word[1..]))
                            .spanning(to_span(word).unwrap()),
                        Vec::new(),
                    ))
                }
            } else {
                Err(Error::LabelExpected { span: span.into() })
            }
        }
        Spanned { node: b'#', span } => match hex_number::parse_hex_number(&word[1..]) {
            Ok(hex_number::HexNumber::Byte(value)) => Ok(Token::LiteralHexByte(value)),
            Ok(hex_number::HexNumber::Short(value)) => Ok(Token::LiteralHexShort(value)),
            Err(hex_number::Error::DigitExpected) => {
                Err(Error::HexNumberOrCharacterExpected { span: span.into() })
            }
            Err(hex_number::Error::DigitInvalid { digit, span }) => Err(Error::HexDigitInvalid {
                digit,
                number: String::from_utf8_lossy(&to_string(&word[1..])).into_owned(),
                span: span.into(),
            }),
            Err(hex_number::Error::UnevenLength { length: 1 }) => {
                Ok(Token::LiteralHexByte(word[1].node as u8))
            }
            Err(hex_number::Error::UnevenLength { length }) => Err(Error::HexNumberUnevenLength {
                length,
                number: String::from_utf8_lossy(&to_string(&word[1..])).into_owned(),
                span: to_span(&word[1..]).unwrap().into(),
            }),
            Err(hex_number::Error::TooLong { length }) => Err(Error::HexNumberTooLong {
                length,
                number: String::from_utf8_lossy(&to_string(&word[1..])).into_owned(),
                span: to_span(&word[1..]).unwrap().into(),
            }),
        }
        .map(|token| (token.spanning(to_span(word).unwrap()), Vec::new())),
        Spanned { node: b'.', span } => match parse_identifier(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::LiteralZeroPageAddress(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: b',', span } => match parse_identifier(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::LiteralRelativeAddress(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: b';', span } => match parse_identifier(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::LiteralAbsoluteAddress(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: b':', span } => match parse_identifier(span, &word[1..]) {
            Ok(name) => {
                return Ok((
                    Token::RawAddress(name).spanning(to_span(word).unwrap()),
                    Vec::new(),
                ));
            }
            Err(err) => Err(err),
        },
        Spanned { node: b'\'', span } => {
            let bytes: Vec<u8> = to_string(&word[1..]);
            match bytes.len() {
                0 => Err(Error::CharacterExpected { span: span.into() }),
                1 => Ok((
                    Token::RawChar(bytes[0]).spanning(Span::combine(&span, &word[1].span)),
                    Vec::new(),
                )),
                _ => {
                    let span = to_span(&word[1..]).unwrap();
                    Err(Error::MoreThanOneByteFound {
                        bytes,
                        span: span.into(),
                    })
                }
            }
        }
        Spanned { node: b'"', .. } => {
            return Ok((
                Token::RawWord(to_string(&word[1..])).spanning(to_span(word).unwrap()),
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
                Token::MacroInvoke(to_string(word)).spanning(to_span(word).unwrap()),
                Vec::new(),
            ));
        }
    }
}

fn to_string(symbols: &[Spanned<u8>]) -> Vec<u8> {
    symbols.iter().map(|Spanned { node: ch, .. }| *ch).collect()
}

fn to_span(symbols: &[Spanned<u8>]) -> Option<Span> {
    Some(Span::combine(&symbols.first()?.span, &symbols.last()?.span))
}

fn parse_macro(rune_span: Span, symbols: &[Spanned<u8>]) -> Result<Vec<u8>, Error> {
    if symbols.is_empty() {
        return Err(Error::MacroNameExpected {
            span: rune_span.into(),
        });
    }

    if let Ok(_) = hex_number::parse_hex_number(symbols) {
        return Err(Error::MacroCannotBeAHexNumber {
            span: to_span(symbols).unwrap().into(),
            number: String::from_utf8_lossy(&to_string(symbols)).into_owned(),
        });
    }
    if let Some(_) = parse_instruction(symbols) {
        return Err(Error::MacroCannotBeAnInstruction {
            span: to_span(symbols).unwrap().into(),
            instruction: String::from_utf8_lossy(&to_string(symbols)).into_owned(),
        });
    }

    Ok(to_string(symbols))
}

fn parse_identifier(rune_span: Span, symbols: &[Spanned<u8>]) -> Result<Identifier, Error> {
    if symbols.is_empty() {
        return Err(Error::IdentifierExpected {
            span: rune_span.into(),
        });
    }

    if let Some(Spanned { node: b'&', span }) = symbols.first() {
        let rune_span = Span::combine(&rune_span, &span);
        if symbols[1..].is_empty() {
            return Err(Error::SublabelExpected {
                span: rune_span.into(),
            });
        }
        return Ok(Identifier::Sublabel(to_string(&symbols[1..])));
    }

    match symbols
        .iter()
        .map(|Spanned { node: ch, .. }| *ch)
        .position(|c| c == b'/')
    {
        Some(position) => {
            if let Some(second_position) = symbols[position + 1..]
                .iter()
                .map(|Spanned { node: ch, .. }| *ch)
                .position(|c| c == b'/')
            {
                return Err(Error::MoreThanOneSlashInIdentifier {
                    span: symbols[position + 1 + second_position].span.into(),
                });
            }

            let label = {
                let label_symbols = &symbols[..position];
                if label_symbols.is_empty() {
                    return Err(Error::LabelExpected {
                        span: rune_span.into(),
                    });
                }
                to_string(label_symbols)
            };
            let sublabel = {
                let sublabel_symbols = &symbols[position + 1..];
                if sublabel_symbols.is_empty() {
                    return Err(Error::SublabelExpected {
                        span: symbols[position].span.into(),
                    });
                }
                to_string(sublabel_symbols)
            };
            Ok(Identifier::Path(label, sublabel))
        }
        None => {
            if symbols.is_empty() {
                return Err(Error::LabelExpected {
                    span: rune_span.into(),
                });
            }
            Ok(Identifier::Label(to_string(symbols)))
        }
    }
}

/// `symbols` must not be empty.
fn parse_instruction(symbols: &[Spanned<u8>]) -> Option<(Instruction, Vec<Warning>)> {
    if symbols.len() < 3 {
        return None;
    }

    let instruction_kind = match to_string(&symbols[..3]).as_slice() {
        b"BRK" | b"LIT" => Some(InstructionKind::BreakOrLiteral),
        b"INC" => Some(InstructionKind::Increment),
        b"POP" => Some(InstructionKind::Pop),
        b"NIP" => Some(InstructionKind::Nip),
        b"SWP" => Some(InstructionKind::Swap),
        b"ROT" => Some(InstructionKind::Rotate),
        b"DUP" => Some(InstructionKind::Duplicate),
        b"OVR" => Some(InstructionKind::Over),
        b"EQU" => Some(InstructionKind::Equal),
        b"NEQ" => Some(InstructionKind::NotEqual),
        b"GTH" => Some(InstructionKind::GreaterThan),
        b"LTH" => Some(InstructionKind::LesserThan),
        b"JMP" => Some(InstructionKind::Jump),
        b"JCN" => Some(InstructionKind::JumpCondition),
        b"JSR" => Some(InstructionKind::JumpStash),
        b"STH" => Some(InstructionKind::Stash),
        b"LDZ" => Some(InstructionKind::LoadZeroPage),
        b"STZ" => Some(InstructionKind::StoreZeroPage),
        b"LDR" => Some(InstructionKind::LoadRelative),
        b"STR" => Some(InstructionKind::StoreRelative),
        b"LDA" => Some(InstructionKind::LoadAbsolute),
        b"STA" => Some(InstructionKind::StoreAbsolute),
        b"DEI" => Some(InstructionKind::DeviceIn),
        b"DEO" => Some(InstructionKind::DeviceOut),
        b"ADD" => Some(InstructionKind::Add),
        b"SUB" => Some(InstructionKind::Subtract),
        b"MUL" => Some(InstructionKind::Multiply),
        b"DIV" => Some(InstructionKind::Divide),
        b"AND" => Some(InstructionKind::And),
        b"ORA" => Some(InstructionKind::Or),
        b"EOR" => Some(InstructionKind::ExclusiveOr),
        b"SFT" => Some(InstructionKind::Shift),
        _ => None,
    }?;

    let mut keep: Option<Span> = None;
    let mut r#return: Option<Span> = None;
    let mut short: Option<Span> = None;
    let mut warnings = Vec::new();

    for Spanned { node: ch, span } in &symbols[3..] {
        match ch {
            b'k' => {
                if let Some(other_span) = keep {
                    warnings.push(Warning::InstructionModeDefinedMoreThanOnce {
                        instruction_mode: 'k',
                        instruction: String::from_utf8_lossy(&to_string(&symbols[..3]))
                            .into_owned(),
                        span: (*span).into(),
                        other_span: other_span.into(),
                    });
                }
                keep = Some(*span);
            }
            b'r' => {
                if let Some(other_span) = r#return {
                    warnings.push(Warning::InstructionModeDefinedMoreThanOnce {
                        instruction_mode: 'r',
                        instruction: String::from_utf8_lossy(&to_string(&symbols[..3]))
                            .into_owned(),
                        span: (*span).into(),
                        other_span: other_span.into(),
                    });
                }
                r#return = Some(*span);
            }
            b'2' => {
                if let Some(other_span) = short {
                    warnings.push(Warning::InstructionModeDefinedMoreThanOnce {
                        instruction_mode: '2',
                        instruction: String::from_utf8_lossy(&to_string(&symbols[..3]))
                            .into_owned(),
                        span: (*span).into(),
                        other_span: other_span.into(),
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

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Word::Fine { token, warnings } => {
                let mut debug_struct = f.debug_struct("Fine");
                debug_struct.field("token", token);
                if !warnings.is_empty() {
                    debug_struct.field("warnings", warnings);
                }
                debug_struct.finish()
            }
            Word::Faulty { errors, warnings } => {
                let mut debug_struct = f.debug_struct("Faulty");
                debug_struct.field("errors", errors);
                if !warnings.is_empty() {
                    debug_struct.field("warnings", warnings);
                }
                debug_struct.finish()
            }
        }
    }
}
