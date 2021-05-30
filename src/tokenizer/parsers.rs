use super::{Error, Identifier, Instruction, InstructionKind, Slice, Span, Spanned, Symbols};

pub fn peek_word<'a>(symbols: &'a mut Symbols) -> Symbols<'a> {
    let s = symbols.clone();
    let next_non_whitespace_index = s.position(|ch| !ch.is_whitespace()).unwrap_or(s.len());
    let s = s.slice(next_non_whitespace_index..);
    let next_whitespace_index = s.position(|ch| ch.is_whitespace()).unwrap_or(s.len());
    let word = s.slice(..next_whitespace_index);
    *symbols = s.slice(next_whitespace_index..);
    return word;
}

pub fn parse_macro(rune_span: Span, word: Symbols) -> Result<String, Error> {
    if word.is_empty() {
        return Err(Error::MacroNameExpected { span: rune_span });
    }

    if let Ok(_) = parse_hex_number(word) {
        return Err(Error::MacroCannotBeAHexNumber {
            span: word.to_span().unwrap(),
            number: word.to_string(),
        });
    }
    if let Some(_) = parse_instruction(word) {
        return Err(Error::MacroCannotBeAnInstruction {
            span: word.to_span().unwrap(),
            instruction: word.to_string(),
        });
    }

    Ok(word.to_string())
}

pub fn parse_label(rune_span: Span, word: Symbols) -> Result<String, Error> {
    if word.is_empty() {
        return Err(Error::LabelExpected { span: rune_span });
    }

    Ok(word.to_string())
}

pub fn parse_sublabel(word: Symbols) -> String {
    word.to_string()
}

pub fn parse_identifier(rune_span: Span, word: Symbols) -> Result<Identifier, Error> {
    if word.is_empty() {
        return Err(Error::IdentifierExpected { span: rune_span });
    }

    if word.first().unwrap().node == '&' {
        return Ok(Identifier::Sublabel(parse_sublabel(word.slice(1..))));
    }

    match word.position(|c| c == '/') {
        Some(position) => {
            let label_word = word.slice(..position);
            let sublabel_word = word.slice(position + 1..);
            let label = parse_label(rune_span, label_word)?;
            let sublabel = parse_sublabel(sublabel_word);
            Ok(Identifier::Path(label, sublabel))
        }
        None => Ok(Identifier::Label(parse_label(rune_span, word)?)),
    }
}

pub fn parse_hex_number(word: Symbols) -> Result<usize, Error> {
    if word.is_empty() {
        return Ok(0);
    }

    let mut value: usize = 0;

    for Spanned { node: ch, span } in word {
        if is_hex_digit(ch) {
            value = (value << 4) + to_hex_digit(ch).unwrap() as usize;
        } else {
            return Err(Error::HexDigitInvalid {
                digit: ch,
                number: word.to_string(),
                span,
            });
        }
    }

    Ok(value)
}

/// `symbols` must not be empty.
pub fn parse_instruction(word: Symbols) -> Option<Instruction> {
    if word.len() < 3 {
        return None;
    }

    let instruction_kind = match word.slice(..3).to_string().as_str() {
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

    for Spanned { node: ch, span } in word.slice(3..) {
        match ch {
            'k' => {
                if let Some(_) = keep {
                    // TODO: transform into a warning
                    // return Err((
                    //     Error::InstructionModeDefinedMoreThanOnce {
                    //         instruction_mode: 'k',
                    //         instruction: word.to_string(),
                    //         span,
                    //         other_span,
                    //     },
                    //     symbols.slice(length..),
                    // ));
                }
                keep = Some(span);
            }
            'r' => {
                if let Some(_) = r#return {
                    // return Err((
                    //     Error::InstructionModeDefinedMoreThanOnce {
                    //         instruction_mode: 'r',
                    //         instruction: word.to_string(),
                    //         span,
                    //         other_span,
                    //     },
                    //     symbols.slice(length..),
                    // ));
                }
                r#return = Some(span);
            }
            '2' => {
                if let Some(_) = short {
                    // return Err((
                    //     Error::InstructionModeDefinedMoreThanOnce {
                    //         instruction_mode: '2',
                    //         instruction: word.to_string(),
                    //         span,
                    //         other_span,
                    //     },
                    //     symbols.slice(length..),
                    // ));
                }
                short = Some(span);
            }
            _ => {
                return None;
            }
        }
    }

    return Some(Instruction {
        instruction_kind,
        keep: keep.is_some(),
        r#return: r#return.is_some(),
        short: short.is_some(),
    });
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
