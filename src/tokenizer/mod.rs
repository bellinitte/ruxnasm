use super::{Breadcrumbs, Token};
use super::{Location, Span, Spanned, Spanning};
use crate::{Instruction, InstructionKind};
pub use error::Error;
use symbols::{Slice, Symbols};
use parsers::*;

mod error;
mod parsers;
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
