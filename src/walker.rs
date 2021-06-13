use crate::span::Spanning;
use crate::token::Identifier;
use crate::token::ScopedIdentifier;
use crate::token::Statement;
use crate::{tokenizer::Word, Span, Spanned, Token};
use crate::{Error, Warning};
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Default)]
struct Environment {
    address: u16,
    opened_brackets: Vec<Span>,
    opened_braces: Vec<Span>,
    scope: Option<String>,
    macro_definitions: HashMap<String, (Vec<Word>, Span)>,
    unused_macros: HashSet<String>,
    label_definitions: HashMap<ScopedIdentifier, (u16, Span)>,
    zeroth_page_spans: Vec<Span>,
    overflow_spans: Vec<Span>,
}

impl Environment {
    pub fn push_bytes(&mut self, bytes: u16, span: Span) {
        if self.address < 256 {
            self.zeroth_page_spans.push(span);
        }
        self.increment_pointer(bytes, span);
    }

    pub fn set_pointer(&mut self, to: u16) -> Result<(), u16> {
        if to < self.address {
            return Err(self.address);
        }
        self.address = to;
        Ok(())
    }

    pub fn increment_pointer(&mut self, by: u16, span: Span) {
        match self.address.checked_add(by) {
            Some(result) => {
                self.address = result;
            }
            None => self.overflow_spans.push(span),
        }
    }
}

pub(crate) struct Definitions {
    pub labels: HashMap<ScopedIdentifier, (u16, Span)>,
}

pub(crate) fn walk(
    words: Vec<Word>,
) -> Result<(Vec<Spanned<Statement>>, Definitions, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut environment = Environment::default();

    let statements = match walk_rec(words, &mut environment) {
        Ok((statements, new_warnings)) => {
            warnings.extend(new_warnings);
            statements
        }
        Err((new_errors, new_warnings)) => {
            errors.extend(new_errors);
            warnings.extend(new_warnings);
            vec![]
        }
    };

    for opened_bracket in environment.opened_brackets {
        errors.push(Error::NoMatchingClosingBracket {
            span: opened_bracket.into(),
        })
    }

    for opened_brace in environment.opened_braces {
        errors.push(Error::NoMatchingClosingBrace {
            span: opened_brace.into(),
        })
    }

    if !environment.zeroth_page_spans.is_empty() {
        let mut entire_span = environment.zeroth_page_spans[0];
        for span in environment.zeroth_page_spans.into_iter().skip(1) {
            entire_span = Span::combine(&entire_span, &span)
        }
        errors.push(Error::BytesInZerothPage {
            span: entire_span.into(),
        });
    }

    if !environment.overflow_spans.is_empty() {
        let mut entire_span = environment.overflow_spans[0];
        for span in environment.overflow_spans.into_iter().skip(1) {
            entire_span = Span::combine(&entire_span, &span)
        }
        errors.push(Error::ProgramTooLong {
            span: entire_span.into(),
        });
    }

    for unused_macro_name in environment.unused_macros {
        let (_, span) = environment.macro_definitions[&unused_macro_name];
        warnings.push(Warning::MacroUnused {
            name: unused_macro_name,
            span: span.into(),
        });
    }

    if errors.is_empty() {
        Ok((
            statements,
            Definitions {
                labels: environment.label_definitions,
            },
            warnings,
        ))
    } else {
        Err((errors, warnings))
    }
}

fn walk_rec(
    words: Vec<Word>,
    environment: &mut Environment,
) -> Result<(Vec<Spanned<Statement>>, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
    let mut errors: Vec<Error> = Vec::new();
    let mut warnings: Vec<Warning> = Vec::new();

    let mut statements: Vec<Spanned<Statement>> = Vec::new();
    let mut words = words.into_iter().peekable();

    loop {
        match words.next() {
            Some(Word::Fine {
                token,
                warnings: new_warnings,
            }) => {
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
                            errors.push(Error::NoMatchingOpeningBracket { span: span.into() });
                        }
                    }
                    Spanned {
                        node: Token::OpeningBrace,
                        span,
                    } => {
                        environment.opened_braces.push(span);
                        errors
                            .push(Error::OpeningBraceNotAfterMacroDefinition { span: span.into() });
                    }
                    Spanned {
                        node: Token::ClosingBrace,
                        span,
                    } => {
                        if environment.opened_braces.pop().is_none() {
                            errors.push(Error::NoMatchingOpeningBrace { span: span.into() });
                        }
                    }
                    Spanned {
                        node: Token::Instruction(instruction),
                        span,
                    } => {
                        statements.push(Statement::Instruction(instruction).spanning(span));
                        environment.push_bytes(1, span);
                    }
                    Spanned {
                        node: Token::MacroDefine(name),
                        span,
                    } => {
                        words = walk_macro_definition(
                            name,
                            span,
                            words,
                            environment,
                            &mut errors,
                            &mut warnings,
                        );
                    }
                    Spanned {
                        node: Token::MacroInvoke(name),
                        span,
                    } => match environment.macro_definitions.get(&name).cloned() {
                        Some((items, _)) => {
                            environment.unused_macros.remove(&name);
                            match walk_rec(items, environment) {
                                Ok((new_statemtents, new_warnings)) => {
                                    statements.extend(new_statemtents);
                                    warnings.extend(new_warnings);
                                }
                                Err((new_errors, new_warnings)) => {
                                    errors.extend(new_errors.into_iter().map(|error| {
                                        Error::MacroError {
                                            original_error: Box::new(error),
                                            span: span.into(),
                                        }
                                    }));
                                    warnings.extend(new_warnings);
                                }
                            }
                        }
                        None => errors.push(Error::MacroUndefined {
                            name: name,
                            span: span.into(),
                        }),
                    },
                    Spanned {
                        node: Token::PadAbsolute(value),
                        span,
                    } => {
                        statements.push(Statement::PadAbsolute(value).spanning(span));
                        match environment.set_pointer(value as u16) {
                            Ok(()) => (),
                            Err(previous_address) => errors.push(Error::PaddedBackwards {
                                previous_pointer: previous_address as usize,
                                desired_pointer: value,
                                span: span.into(),
                            })
                        }
                    }
                    Spanned {
                        node: Token::PadRelative(value),
                        span,
                    } => {
                        statements.push(Statement::PadRelative(value).spanning(span));
                        environment.increment_pointer(value as u16, span);
                    }
                    Spanned {
                        node: Token::LabelDefine(name),
                        span,
                    } => {
                        if let Some((_, other_span)) = environment.label_definitions.insert(
                            ScopedIdentifier::Label(name.clone()),
                            (environment.address, span),
                        ) {
                            errors.push(Error::LabelDefinedMoreThanOnce {
                                name: name.clone(),
                                span: span.into(),
                                other_span: other_span.into(),
                            });
                        }
                        environment.scope = Some(name);
                    }
                    Spanned {
                        node: Token::SublabelDefine(name),
                        span,
                    } => match &environment.scope {
                        Some(scope_name) => {
                            if let Some((_, other_span)) = environment.label_definitions.insert(
                                ScopedIdentifier::Sublabel(scope_name.to_owned(), name.clone()),
                                (environment.address, span),
                            ) {
                                errors.push(Error::LabelDefinedMoreThanOnce {
                                    name,
                                    span: span.into(),
                                    other_span: other_span.into(),
                                });
                            }
                        }
                        None => errors.push(Error::SublabelDefinedWithoutScope {
                            name,
                            span: span.into(),
                        }),
                    },
                    Spanned {
                        node: Token::LiteralZeroPageAddress(identifier),
                        span,
                    } => match scope_identifier(identifier, &environment.scope, &span) {
                        Ok(scoped_identifier) => {
                            statements.push(
                                Statement::LiteralZeroPageAddress(scoped_identifier).spanning(span),
                            );
                            environment.push_bytes(2, span);
                        }
                        Err(err) => errors.push(err),
                    },
                    Spanned {
                        node: Token::LiteralRelativeAddress(identifier),
                        span,
                    } => match scope_identifier(identifier, &environment.scope, &span) {
                        Ok(scoped_identifier) => {
                            statements.push(
                                Statement::LiteralRelativeAddress(scoped_identifier).spanning(span),
                            );
                            environment.push_bytes(2, span);
                        }
                        Err(err) => errors.push(err),
                    },
                    Spanned {
                        node: Token::LiteralAbsoluteAddress(identifier),
                        span,
                    } => match scope_identifier(identifier, &environment.scope, &span) {
                        Ok(scoped_identifier) => {
                            statements.push(
                                Statement::LiteralAbsoluteAddress(scoped_identifier).spanning(span),
                            );
                            environment.push_bytes(3, span);
                        }
                        Err(err) => errors.push(err),
                    },
                    Spanned {
                        node: Token::RawAddress(identifier),
                        span,
                    } => match scope_identifier(identifier, &environment.scope, &span) {
                        Ok(scoped_identifier) => {
                            statements
                                .push(Statement::RawAddress(scoped_identifier).spanning(span));
                            environment.push_bytes(2, span);
                        }
                        Err(err) => errors.push(err),
                    },
                    Spanned {
                        node: Token::LiteralHexByte(value),
                        span,
                    } => {
                        statements.push(Statement::LiteralHexByte(value).spanning(span));
                        environment.push_bytes(2, span);
                    }
                    Spanned {
                        node: Token::LiteralHexShort(value),
                        span,
                    } => {
                        statements.push(Statement::LiteralHexShort(value).spanning(span));
                        environment.push_bytes(3, span);
                    }
                    Spanned {
                        node: Token::RawHexByte(value),
                        span,
                    } => {
                        statements.push(Statement::RawHexByte(value).spanning(span));
                        environment.push_bytes(1, span);
                    }
                    Spanned {
                        node: Token::RawHexShort(value),
                        span,
                    } => {
                        statements.push(Statement::RawHexShort(value).spanning(span));
                        environment.push_bytes(2, span);
                    }
                    Spanned {
                        node: Token::RawChar(value),
                        span,
                    } => {
                        statements.push(Statement::RawChar(value).spanning(span));
                        environment.push_bytes(1, span);
                    }
                    Spanned {
                        node: Token::RawWord(word),
                        span,
                    } => {
                        environment.push_bytes(word.bytes().len() as u16, span);
                        statements.push(Statement::RawWord(word).spanning(span));
                    }
                }
            }
            Some(Word::Faulty {
                errors: new_errors,
                warnings: new_warnings,
            }) => {
                errors.extend(new_errors);
                warnings.extend(new_warnings);
            }
            None => break,
        }
    }

    if errors.is_empty() {
        Ok((statements, warnings))
    } else {
        Err((errors, warnings))
    }
}

fn walk_macro_definition(
    name: String,
    span: Span,
    mut words: Peekable<IntoIter<Word>>,
    environment: &mut Environment,
    errors: &mut Vec<Error>,
    warnings: &mut Vec<Warning>,
) -> Peekable<IntoIter<Word>> {
    let mut items: Vec<Word> = Vec::new();

    match words.peek() {
        Some(Word::Fine {
            token:
                Spanned {
                    node: Token::OpeningBrace,
                    span: opening_brace_span,
                },
            warnings: new_warnings,
        }) => {
            let brace_level = environment.opened_braces.len();
            environment.opened_braces.push(*opening_brace_span);
            warnings.extend(new_warnings.iter().cloned());
            words.next();
            'macro_define: loop {
                let option_word = words.next();
                match &option_word {
                    Some(Word::Fine {
                        token:
                            Spanned {
                                node: Token::OpeningBrace,
                                span,
                            },
                        warnings: new_warnings,
                    }) => {
                        environment.opened_braces.push(*span);
                        warnings.extend(new_warnings.iter().cloned());
                        items.push(option_word.unwrap());
                    }
                    Some(Word::Fine {
                        token:
                            Spanned {
                                node: Token::ClosingBrace,
                                ..
                            },
                        warnings: new_warnings,
                    }) => {
                        environment.opened_braces.pop().unwrap();
                        warnings.extend(new_warnings.iter().cloned());
                        if environment.opened_braces.len() == brace_level {
                            break 'macro_define;
                        } else {
                            items.push(option_word.unwrap());
                        }
                    }
                    Some(word) => {
                        items.push(word.clone());
                    }
                    None => break 'macro_define,
                }
            }
        }
        _ => (),
    }

    if let Some((_, other_span)) = environment
        .macro_definitions
        .insert(name.clone(), (items, span))
    {
        errors.push(Error::MacroDefinedMoreThanOnce {
            name: name.clone(),
            span: span.into(),
            other_span: other_span.into(),
        });
    }
    environment.unused_macros.insert(name);

    words
}

fn scope_identifier(
    identifier: Identifier,
    scope: &Option<String>,
    span: &Span,
) -> Result<ScopedIdentifier, Error> {
    match identifier {
        Identifier::Label(name) => Ok(ScopedIdentifier::Label(name)),
        Identifier::Path(label, sublabel) => Ok(ScopedIdentifier::Sublabel(label, sublabel)),
        Identifier::Sublabel(sublabel) => match scope {
            Some(scope_name) => Ok(ScopedIdentifier::Sublabel(scope_name.to_owned(), sublabel)),
            None => Err(Error::SublabelReferencedWithoutScope {
                name: sublabel,
                span: (*span).into(),
            }),
        },
    }
}
