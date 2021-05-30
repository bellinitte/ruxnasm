use super::{Spanned, Span, Spanning, Slice, Error, Breadcrumbs, Symbols, Instruction, InstructionKind};

pub fn peek_word(symbols: Symbols) -> Symbols {
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

pub fn peek_spacer(symbols: Symbols) -> Option<(Symbols, Span, Symbols)> {
    let mut i = 0;
    loop {
        match symbols.get(i) {
            Some(Spanned { node: c, span }) if c == '/' => {
                return Some((symbols.slice(..i), span, symbols.slice(i + 1..)));
            }
            Some(_) => {
                i += 1;
            }
            None => {
                return None;
            }
        }
    }
}

pub fn parse_word(first_span: Span, symbols: Symbols) -> (Spanned<String>, Symbols) {
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

pub fn parse_identifier(
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

pub fn parse_breadcrumbs(
    first_span: Span,
    symbols: Symbols,
) -> Result<(Spanned<Breadcrumbs>, Symbols), (Error, Symbols)> {
    let word = peek_word(symbols);
    let length = word.len();

    if let Some((label_symbols, spacer_span, sublabel_symbols)) = peek_spacer(word) {
        let (Spanned { node: label, .. }, _) = parse_identifier(first_span, label_symbols)?;
        let (Spanned { node: sublabel, .. }, _) = parse_identifier(spacer_span, sublabel_symbols)?;
        let last_span = word.last().unwrap().span;
        let breadcrumbs = Breadcrumbs::Sublabel(label, sublabel).spanning(Span::combine(&first_span, &last_span));
        return Ok((breadcrumbs, symbols.slice(length..)));
    } else {
        let (Spanned { node: label, .. }, _) = parse_identifier(first_span, word)?;
        let last_span = word.last().unwrap().span;
        let breadcrumbs = Breadcrumbs::Label(label).spanning(Span::combine(&first_span, &last_span));
        return Ok((breadcrumbs, symbols.slice(length..)));
    }


}

pub fn parse_hex_number(
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
pub fn parse_instruction(
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

    return Ok((
        Instruction {
            instruction_kind,
            keep: keep.is_some(),
            r#return: r#return.is_some(),
            short: short.is_some(),
        }
        .spanning(Span::combine(&word.first().unwrap().span, &word.last().unwrap().span)),
        symbols.slice(length..),
        symbols.slice(..length),
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
