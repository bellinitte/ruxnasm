use crate::{tokenizer::Word, Span, Spanned, Token};
pub use anomalies::Error;
use std::collections::HashMap;

mod anomalies;

#[derive(Default)]
struct Environment {
    address: u16,
    opened_brackets: Vec<Span>,
    opened_braces: Vec<Span>,
    scope: Option<String>,
    macro_definitions: HashMap<String, (Vec<Word>, Span)>,
    macro_references: HashMap<String, usize>,
    label_definitions: HashMap<String, (u16, Span)>,
    sublabel_definitions: HashMap<(String, String), (u16, Span)>,
}

pub fn walk(words: &[Word]) -> (Vec<crate::Error>, Vec<crate::Warning>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut environment = Environment::default();

    let (new_errors, new_warnings) = walk_rec(words, &mut environment);
    errors.extend(new_errors);
    warnings.extend(new_warnings);

    for opened_bracket in environment.opened_brackets {
        errors.push(
            Error::NoMatchingClosingBracket {
                span: opened_bracket,
            }
            .into(),
        )
    }

    for opened_brace in environment.opened_braces {
        errors.push(Error::NoMatchingClosingBrace { span: opened_brace }.into())
    }

    println!("macros: {:?}\n", environment.macro_definitions.keys());
    println!("labels: {:?}\n", environment.label_definitions.keys());
    println!("sublabels: {:?}\n", environment.sublabel_definitions.keys());
    println!("address: {:?}\n", environment.address);

    for (macro_name, (macro_items, _)) in environment.macro_definitions.iter() {
        println!("macro {}: {:?}", macro_name, macro_items);
    }

    (errors, warnings)
}

fn walk_rec(
    words: &[Word],
    environment: &mut Environment,
) -> (Vec<crate::Error>, Vec<crate::Warning>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut words = words.iter().peekable();

    loop {
        if let Some(word) = words.next() {
            match word.get_token() {
                Ok((token, new_warnings)) => {
                    warnings.extend(new_warnings);
                    match token {
                        Spanned {
                            node: Token::OpeningBracket,
                            span,
                        } => {
                            environment.opened_brackets.push(span);
                        }
                        Spanned {
                            node: Token::ClosingBracket,
                            span,
                        } => {
                            if environment.opened_brackets.pop().is_none() {
                                errors.push(Error::NoMatchingOpeningBracket { span }.into());
                            }
                        }
                        Spanned {
                            node: Token::OpeningBrace,
                            span,
                        } => {
                            environment.opened_braces.push(span);
                            errors.push(Error::OpeningBraceNotAfterMacroDefinition { span }.into());
                        }
                        Spanned {
                            node: Token::ClosingBrace,
                            span,
                        } => {
                            if environment.opened_braces.pop().is_none() {
                                errors.push(Error::NoMatchingOpeningBrace { span }.into());
                            }
                        }
                        Spanned {
                            node: Token::Instruction(_),
                            ..
                        } => environment.address += 2,
                        Spanned {
                            node: Token::MacroDefine(name),
                            span,
                        } => {
                            let mut items: Vec<Word> = Vec::new();
                            if let Some(word) = words.peek() {
                                match word.get_token() {
                                    Ok((
                                        Spanned {
                                            node: Token::OpeningBrace,
                                            span: opening_brace_span,
                                        },
                                        new_warnings,
                                    )) => {
                                        let brace_level = environment.opened_braces.len();
                                        environment.opened_braces.push(opening_brace_span);
                                        warnings.extend(new_warnings);
                                        words.next();
                                        'macro_define: loop {
                                            if let Some(word) = words.next() {
                                                match word.get_token() {
                                                    Ok((
                                                        Spanned {
                                                            node: Token::OpeningBrace,
                                                            span,
                                                        },
                                                        new_warnings,
                                                    )) => {
                                                        environment.opened_braces.push(span);
                                                        warnings.extend(new_warnings);
                                                        items.push(word.clone());
                                                    }
                                                    Ok((
                                                        Spanned {
                                                            node: Token::ClosingBrace,
                                                            ..
                                                        },
                                                        new_warnings,
                                                    )) => {
                                                        environment.opened_braces.pop().unwrap();
                                                        warnings.extend(new_warnings);
                                                        if environment.opened_braces.len()
                                                            == brace_level
                                                        {
                                                            break 'macro_define;
                                                        } else {
                                                            items.push(word.clone());
                                                        }
                                                    }
                                                    _ => {
                                                        items.push(word.clone());
                                                    }
                                                }
                                            } else {
                                                break 'macro_define;
                                            }
                                        }
                                    }
                                    Ok(_) => (),
                                    Err((new_errors, new_warnings)) => {
                                        errors.extend(new_errors);
                                        warnings.extend(new_warnings);
                                    }
                                }
                            }
                            if let Some((_, other_span)) = environment
                                .macro_definitions
                                .insert(name.clone(), (items, span))
                            {
                                errors.push(
                                    Error::MacroDefinedMoreThanOnce {
                                        name: name.clone(),
                                        span,
                                        other_span,
                                    }
                                    .into(),
                                );
                            }
                            environment.macro_references.insert(name, 0);
                        }
                        Spanned {
                            node: Token::MacroExpand(name),
                            span,
                        } => match environment.macro_definitions.get(&name).cloned() {
                            Some((items, _)) => {
                                let (new_errors, new_warnings) = walk_rec(&items, environment);
                                errors.extend(new_errors);
                                warnings.extend(new_warnings);
                            }
                            None => errors.push(Error::MacroUndefined { name, span }.into()),
                        },
                        Spanned {
                            node: Token::PadAbsolute(value),
                            ..
                        } => environment.address = value as u16,
                        Spanned {
                            node: Token::PadRelative(value),
                            ..
                        } => environment.address += value as u16,
                        Spanned {
                            node: Token::LabelDefine(name),
                            span,
                        } => {
                            if let Some((_, other_span)) = environment
                                .label_definitions
                                .insert(name.clone(), (environment.address, span))
                            {
                                errors.push(
                                    Error::LabelDefinedMoreThanOnce {
                                        name: name.clone(),
                                        span,
                                        other_span,
                                    }
                                    .into(),
                                );
                            }
                            environment.scope = Some(name);
                        }
                        Spanned {
                            node: Token::SublabelDefine(name),
                            span,
                        } => match &environment.scope {
                            Some(scope_name) => {
                                if let Some((_, other_span)) =
                                    environment.sublabel_definitions.insert(
                                        (scope_name.clone(), name.clone()),
                                        (environment.address, span),
                                    )
                                {
                                    errors.push(
                                        Error::LabelDefinedMoreThanOnce {
                                            name,
                                            span,
                                            other_span,
                                        }
                                        .into(),
                                    );
                                }
                            }
                            None => errors
                                .push(Error::SublabelDefinedWithoutScope { name, span }.into()),
                        },
                        Spanned {
                            node: Token::LiteralZeroPageAddress(_),
                            ..
                        } => environment.address += 2,
                        Spanned {
                            node: Token::LiteralRelativeAddress(_),
                            ..
                        } => environment.address += 2,
                        Spanned {
                            node: Token::LiteralAbsoluteAddress(_),
                            ..
                        } => environment.address += 3,
                        Spanned {
                            node: Token::RawAddress(_),
                            ..
                        } => environment.address += 2,
                        Spanned {
                            node: Token::LiteralHexByte(_),
                            ..
                        } => environment.address += 2,
                        Spanned {
                            node: Token::LiteralHexShort(_),
                            ..
                        } => environment.address += 3,
                        Spanned {
                            node: Token::RawHexByte(_),
                            ..
                        } => environment.address += 1,
                        Spanned {
                            node: Token::RawHexShort(_),
                            ..
                        } => environment.address += 2,
                        Spanned {
                            node: Token::RawChar(_),
                            ..
                        } => environment.address += 1,
                        Spanned {
                            node: Token::RawWord(word),
                            ..
                        } => environment.address += word.bytes().len() as u16,
                    }
                }
                Err((new_errors, new_warnings)) => {
                    errors.extend(new_errors);
                    warnings.extend(new_warnings);
                }
            }
        } else {
            break;
        }
    }

    (errors, warnings)
}
