use super::{Identifier, Token};
use super::{Location, Span, Spanned, Spanning};
use crate::{Anomalies, Instruction, InstructionKind};
pub use anomalies::{Error, Warning};
use parsers::*;
use symbols::{Slice, Symbols};

mod anomalies;
mod parsers;
mod symbols;

macro_rules! macro_invoke {
    ($word: expr) => {
        $word
            .to_spanned_string()
            .unwrap()
            .map(|s| Token::MacroInvoke(s))
    };
}

pub fn tokenize(input_file_contents: &str) -> (Vec<Spanned<Token>>, Anomalies) {
    let symbols = scan(input_file_contents);
    let mut symbols: Symbols = symbols.as_slice().into();

    let mut tokens = Vec::new();
    let mut anomalies = Anomalies::new();

    'tokens: loop {
        let word = peek_word(&mut symbols);
        match word.first() {
            Some(Spanned { node: '(', .. }) => 'comment: loop {
                let word = peek_word(&mut symbols);
                match word.first() {
                    Some(Spanned { node: ')', .. }) => break 'comment,
                    Some(_) => (),
                    None => break 'tokens,
                }
            },
            Some(Spanned {
                node: '[' | ']' | ')',
                ..
            }) => (),
            Some(Spanned { node: '{', span }) => tokens.push(Token::OpeningBrace.spanning(span)),
            Some(Spanned { node: '}', span }) => tokens.push(Token::ClosingBrace.spanning(span)),
            Some(Spanned { node: '%', span }) => match parse_macro(span, word.slice(1..)) {
                Ok(name) => {
                    tokens.push(Token::MacroDefine(name).spanning(word.to_span().unwrap()));
                }
                Err(err) => anomalies.push_error(err),
            },
            Some(Spanned { node: '|', .. }) => match parse_hex_number(word.slice(1..)) {
                Ok(value) => {
                    tokens.push(Token::PadAbsolute(value).spanning(word.to_span().unwrap()))
                }
                Err(_) => tokens.push(macro_invoke!(word)),
            },
            Some(Spanned { node: '$', .. }) => match parse_hex_number(word.slice(1..)) {
                Ok(value) => {
                    tokens.push(Token::PadRelative(value).spanning(word.to_span().unwrap()))
                }
                Err(_) => tokens.push(macro_invoke!(word)),
            },
            Some(Spanned { node: '@', span }) => match parse_label(span, word.slice(1..)) {
                Ok(name) => {
                    tokens.push(Token::LabelDefine(name).spanning(word.to_span().unwrap()));
                }
                Err(err) => anomalies.push_error(err),
            },
            Some(Spanned { node: '&', .. }) => {
                let name = parse_sublabel(word.slice(1..));
                tokens.push(Token::SublabelDefine(name).spanning(word.to_span().unwrap()));
            }
            Some(Spanned { node: '#', .. }) => match word.slice(1..).len() {
                0 => anomalies.push_error(Error::HexNumberExpected {
                    span: word.to_span().unwrap(),
                }),
                1 => {
                    let Spanned { node: c, .. } = word.second().unwrap();
                    tokens.push(Token::LiteralHexByte(c as u8).spanning(word.to_span().unwrap()));
                }
                2 => match parse_hex_number(word.slice(1..)) {
                    Ok(x) => tokens
                        .push(Token::LiteralHexByte(x as u8).spanning(word.to_span().unwrap())),
                    Err(err) => anomalies.push_error(err),
                },
                3 => match parse_hex_number(word.slice(1..)) {
                    Ok(_) => anomalies.push_error(Error::HexNumberUnevenLength {
                        length: 3,
                        number: word.slice(1..).to_string(),
                        span: word.slice(1..).to_span().unwrap(),
                    }),
                    Err(err) => anomalies.push_error(err),
                },
                4 => match parse_hex_number(word.slice(1..)) {
                    Ok(x) => tokens
                        .push(Token::LiteralHexShort(x as u16).spanning(word.to_span().unwrap())),
                    Err(err) => anomalies.push_error(err),
                },
                length => match parse_hex_number(word.slice(1..)) {
                    Ok(_) => anomalies.push_error(Error::HexNumberTooLong {
                        length,
                        number: word.slice(1..).to_string(),
                        span: word.to_span().unwrap(),
                    }),
                    Err(err) => anomalies.push_error(err),
                },
            },
            Some(Spanned { node: '.', span }) => match parse_identifier(span, word.slice(1..)) {
                Ok(name) => tokens
                    .push(Token::LiteralZeroPageAddress(name).spanning(word.to_span().unwrap())),
                Err(err) => anomalies.push_error(err),
            },
            Some(Spanned { node: ',', span }) => match parse_identifier(span, word.slice(1..)) {
                Ok(name) => tokens
                    .push(Token::LiteralRelativeAddress(name).spanning(word.to_span().unwrap())),
                Err(err) => anomalies.push_error(err),
            },
            Some(Spanned { node: ';', span }) => match parse_identifier(span, word.slice(1..)) {
                Ok(name) => tokens
                    .push(Token::LiteralAbsoluteAddress(name).spanning(word.to_span().unwrap())),
                Err(err) => anomalies.push_error(err),
            },
            Some(Spanned { node: ':', span }) => match parse_identifier(span, word.slice(1..)) {
                Ok(name) => tokens.push(Token::RawAddress(name).spanning(word.to_span().unwrap())),
                Err(err) => anomalies.push_error(err),
            },
            Some(Spanned { node: '\'', span }) => match word.second() {
                Some(spanned) => tokens.push(spanned.map(|c| Token::RawChar(c as u8))),
                None => tokens.push(Token::RawChar(0x00).spanning(span)),
            },
            Some(Spanned { node: '"', span }) => {
                tokens.push(Token::RawWord(word.slice(1..).to_string()).spanning(span))
            }
            Some(_) => {
                let mut flag = false;
                flag |= match parse_hex_number(word) {
                    Ok(x) => {
                        match word.len() {
                            0 => unreachable!(),
                            length if length == 1 || length == 3 => {
                                anomalies.push_error(Error::HexNumberUnevenLength {
                                    length,
                                    number: word.to_string(),
                                    span: word.to_span().unwrap(),
                                })
                            }
                            2 => tokens
                                .push(Token::RawHexByte(x as u8).spanning(word.to_span().unwrap())),
                            4 => tokens.push(
                                Token::RawHexShort(x as u16).spanning(word.to_span().unwrap()),
                            ),
                            length => anomalies.push_error(Error::HexNumberTooLong {
                                length,
                                number: word.to_string(),
                                span: word.to_span().unwrap(),
                            }),
                        }
                        true
                    }
                    Err(_) => false,
                };
                flag |= match parse_instruction(word) {
                    Some((instruction, new_warnings)) => {
                        tokens.push(
                            Token::Instruction(instruction).spanning(word.to_span().unwrap()),
                        );
                        anomalies.push_warnings(new_warnings);
                        true
                    }
                    None => false,
                };
                if !flag {
                    tokens.push(macro_invoke!(word))
                }
            }
            None => break 'tokens,
        }
    }

    (tokens, anomalies)
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
