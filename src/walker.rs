use crate::span::Spanning;
use crate::token::Identifier;
use crate::token::ScopedIdentifier;
use crate::token::Statement;
use crate::{tokenizer::Word, Span, Spanned, Token};
use crate::{Error, Warning};
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::Peekable;
use std::slice::Iter;

pub(crate) struct Definitions {
    pub labels: HashMap<ScopedIdentifier, (u16, Span)>,
}

pub(crate) struct Walker<'words> {
    statements: Vec<Spanned<Statement>>,
    errors: Vec<Error>,
    warnings: Vec<Warning>,
    pointer: u16,
    length: u16,
    opened_brackets: Vec<Span>,
    opened_braces: Vec<Span>,
    scope: Option<Vec<u8>>,
    macro_definitions: HashMap<Vec<u8>, (Vec<&'words Word>, Span)>,
    unused_macros: HashSet<Vec<u8>>,
    label_definitions: HashMap<ScopedIdentifier, (u16, Span)>,
    zeroth_page_spans: Vec<Span>,
    overflow_spans: Vec<Span>,
}

impl<'words> Walker<'words> {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            pointer: 0,
            length: 0,
            opened_brackets: Vec::new(),
            opened_braces: Vec::new(),
            scope: None,
            macro_definitions: HashMap::new(),
            unused_macros: HashSet::new(),
            label_definitions: HashMap::new(),
            zeroth_page_spans: Vec::new(),
            overflow_spans: Vec::new(),
        }
    }

    pub fn push_bytes(&mut self, bytes: u16, span: Span) {
        if self.pointer < 256 {
            self.zeroth_page_spans.push(span);
        }
        self.increment_pointer(bytes, span);
        self.length = self.pointer;
    }

    pub fn set_pointer(&mut self, to: u16) -> Result<(), u16> {
        if self.length > 0 && to < self.pointer {
            return Err(self.pointer);
        }
        self.pointer = to;
        Ok(())
    }

    pub fn increment_pointer(&mut self, by: u16, span: Span) {
        match self.pointer.checked_add(by) {
            Some(result) => {
                self.pointer = result;
            }
            None => self.overflow_spans.push(span),
        }
    }

    pub fn walk(
        &mut self,
        words: &[&'words Word],
    ) -> Option<(Vec<&'words Word>, Vec<&'words Word>)> {
        let mut words = words.iter().peekable();

        loop {
            match words.next() {
                Some(Word::Fine {
                    token,
                    warnings: new_warnings,
                }) => {
                    self.warnings.extend(new_warnings.iter().cloned());
                    match token {
                        Spanned {
                            node: Token::OpeningBracket,
                            span,
                        } => {
                            self.opened_brackets.push(*span);
                        }
                        Spanned {
                            node: Token::ClosingBracket,
                            span,
                        } => {
                            if self.opened_brackets.pop().is_none() {
                                self.errors.push(Error::NoMatchingOpeningBracket {
                                    span: (*span).into(),
                                });
                            }
                        }
                        Spanned {
                            node: Token::OpeningBrace,
                            span,
                        } => {
                            self.opened_braces.push(*span);
                            self.errors
                                .push(Error::OpeningBraceNotAfterMacroDefinition {
                                    span: (*span).into(),
                                });
                        }
                        Spanned {
                            node: Token::ClosingBrace,
                            span,
                        } => {
                            if self.opened_braces.pop().is_none() {
                                self.errors.push(Error::NoMatchingOpeningBrace {
                                    span: (*span).into(),
                                });
                            }
                        }
                        Spanned {
                            node: Token::Instruction(instruction),
                            span,
                        } => {
                            self.statements
                                .push(Statement::Instruction(*instruction).spanning(*span));
                            self.push_bytes(1, *span);
                        }
                        Spanned {
                            node: Token::MacroDefine(name),
                            span,
                        } => {
                            words = self.walk_macro_definition(name, *span, words);
                        }
                        Spanned {
                            node: Token::MacroInvoke(name),
                            span,
                        } => match self.macro_definitions.get(name) {
                            Some((items, _)) => {
                                self.unused_macros.remove(name);
                                return Some((items.clone(), words.copied().collect()));
                            }
                            None => self.errors.push(Error::MacroUndefined {
                                name: String::from_utf8_lossy(&name).into_owned(),
                                span: (*span).into(),
                            }),
                        },
                        Spanned {
                            node: Token::PadAbsolute(value),
                            span,
                        } => {
                            self.statements
                                .push(Statement::PadAbsolute(*value).spanning(*span));
                            match self.set_pointer(*value as u16) {
                                Ok(()) => (),
                                Err(previous_address) => self.errors.push(Error::PaddedBackwards {
                                    previous_pointer: previous_address as usize,
                                    desired_pointer: *value as usize,
                                    span: (*span).into(),
                                }),
                            }
                        }
                        Spanned {
                            node: Token::PadRelative(value),
                            span,
                        } => {
                            self.statements
                                .push(Statement::PadRelative(*value).spanning(*span));
                            self.increment_pointer(*value as u16, *span);
                        }
                        Spanned {
                            node: Token::LabelDefine(name),
                            span,
                        } => {
                            if let Some((_, other_span)) = self.label_definitions.insert(
                                ScopedIdentifier::Label(name.clone()),
                                (self.pointer, *span),
                            ) {
                                self.errors.push(Error::LabelDefinedMoreThanOnce {
                                    name: String::from_utf8_lossy(&name).into_owned(),
                                    span: (*span).into(),
                                    other_span: other_span.into(),
                                });
                            }
                            self.scope = Some(name.clone());
                        }
                        Spanned {
                            node: Token::SublabelDefine(name),
                            span,
                        } => match &self.scope {
                            Some(scope_name) => {
                                if let Some((_, other_span)) = self.label_definitions.insert(
                                    ScopedIdentifier::Sublabel(scope_name.to_owned(), name.clone()),
                                    (self.pointer, *span),
                                ) {
                                    self.errors.push(Error::LabelDefinedMoreThanOnce {
                                        name: String::from_utf8_lossy(&name).into_owned(),
                                        span: (*span).into(),
                                        other_span: other_span.into(),
                                    });
                                }
                            }
                            None => self.errors.push(Error::SublabelDefinedWithoutScope {
                                name: String::from_utf8_lossy(&name).into_owned(),
                                span: (*span).into(),
                            }),
                        },
                        Spanned {
                            node: Token::LiteralZeroPageAddress(identifier),
                            span,
                        } => match scope_identifier(identifier, &self.scope, span) {
                            Ok(scoped_identifier) => {
                                self.statements.push(
                                    Statement::LiteralZeroPageAddress(scoped_identifier)
                                        .spanning(*span),
                                );
                                self.push_bytes(2, *span);
                            }
                            Err(err) => self.errors.push(err),
                        },
                        Spanned {
                            node: Token::LiteralRelativeAddress(identifier),
                            span,
                        } => match scope_identifier(identifier, &self.scope, span) {
                            Ok(scoped_identifier) => {
                                self.statements.push(
                                    Statement::LiteralRelativeAddress(scoped_identifier)
                                        .spanning(*span),
                                );
                                self.push_bytes(2, *span);
                            }
                            Err(err) => self.errors.push(err),
                        },
                        Spanned {
                            node: Token::LiteralAbsoluteAddress(identifier),
                            span,
                        } => match scope_identifier(identifier, &self.scope, &span) {
                            Ok(scoped_identifier) => {
                                self.statements.push(
                                    Statement::LiteralAbsoluteAddress(scoped_identifier)
                                        .spanning(*span),
                                );
                                self.push_bytes(3, *span);
                            }
                            Err(err) => self.errors.push(err),
                        },
                        Spanned {
                            node: Token::RawAddress(identifier),
                            span,
                        } => match scope_identifier(identifier, &self.scope, span) {
                            Ok(scoped_identifier) => {
                                self.statements
                                    .push(Statement::RawAddress(scoped_identifier).spanning(*span));
                                self.push_bytes(2, *span);
                            }
                            Err(err) => self.errors.push(err),
                        },
                        Spanned {
                            node: Token::LiteralHexByte(value),
                            span,
                        } => {
                            self.statements
                                .push(Statement::LiteralHexByte(*value).spanning(*span));
                            self.push_bytes(2, *span);
                        }
                        Spanned {
                            node: Token::LiteralHexShort(value),
                            span,
                        } => {
                            self.statements
                                .push(Statement::LiteralHexShort(*value).spanning(*span));
                            self.push_bytes(3, *span);
                        }
                        Spanned {
                            node: Token::RawHexByte(value),
                            span,
                        } => {
                            self.statements
                                .push(Statement::RawHexByte(*value).spanning(*span));
                            self.push_bytes(1, *span);
                        }
                        Spanned {
                            node: Token::RawHexShort(value),
                            span,
                        } => {
                            self.statements
                                .push(Statement::RawHexShort(*value).spanning(*span));
                            self.push_bytes(2, *span);
                        }
                        Spanned {
                            node: Token::RawChar(value),
                            span,
                        } => {
                            self.statements
                                .push(Statement::RawChar(*value).spanning(*span));
                            self.push_bytes(1, *span);
                        }
                        Spanned {
                            node: Token::RawWord(word),
                            span,
                        } => {
                            self.push_bytes(word.len() as u16, *span);
                            self.statements
                                .push(Statement::RawWord(word.clone()).spanning(*span));
                        }
                    }
                }
                Some(Word::Faulty {
                    errors: new_errors,
                    warnings: new_warnings,
                }) => {
                    self.errors.extend(new_errors.iter().cloned());
                    self.warnings.extend(new_warnings.iter().cloned());
                }
                None => break,
            }
        }

        return None;
    }

    pub fn finalize(
        mut self,
    ) -> Result<(Vec<Spanned<Statement>>, Definitions, Vec<Warning>), (Vec<Error>, Vec<Warning>)>
    {
        for opened_bracket in self.opened_brackets {
            self.errors.push(Error::NoMatchingClosingBracket {
                span: opened_bracket.into(),
            })
        }

        for opened_brace in self.opened_braces {
            self.errors.push(Error::NoMatchingClosingBrace {
                span: opened_brace.into(),
            })
        }

        if !self.zeroth_page_spans.is_empty() {
            let mut entire_span = self.zeroth_page_spans[0];
            for span in self.zeroth_page_spans.into_iter().skip(1) {
                entire_span = Span::combine(&entire_span, &span)
            }
            self.errors.push(Error::BytesInZerothPage {
                span: entire_span.into(),
            });
        }

        if !self.overflow_spans.is_empty() {
            let mut entire_span = self.overflow_spans[0];
            for span in self.overflow_spans.into_iter().skip(1) {
                entire_span = Span::combine(&entire_span, &span)
            }
            self.errors.push(Error::ProgramTooLong {
                span: entire_span.into(),
            });
        }

        for unused_macro_name in self.unused_macros {
            let (_, span) = self.macro_definitions[&unused_macro_name];
            self.warnings.push(Warning::MacroUnused {
                name: String::from_utf8_lossy(&unused_macro_name).into_owned(),
                span: span.into(),
            });
        }

        if self.errors.is_empty() {
            Ok((
                self.statements,
                Definitions {
                    labels: self.label_definitions,
                },
                self.warnings,
            ))
        } else {
            Err((self.errors, self.warnings))
        }
    }

    fn walk_macro_definition<'a>(
        &mut self,
        name: &Vec<u8>,
        span: Span,
        mut words: Peekable<Iter<'a, &'words Word>>,
    ) -> Peekable<Iter<'a, &'words Word>> {
        let mut items: Vec<&'words Word> = Vec::new();

        match words.peek() {
            Some(Word::Fine {
                token:
                    Spanned {
                        node: Token::OpeningBrace,
                        span: opening_brace_span,
                    },
                warnings: new_warnings,
            }) => {
                let brace_level = self.opened_braces.len();
                self.opened_braces.push(*opening_brace_span);
                self.warnings.extend(new_warnings.iter().cloned());
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
                            self.opened_braces.push(*span);
                            self.warnings.extend(new_warnings.iter().cloned());
                            items.push(option_word.unwrap().clone());
                        }
                        Some(Word::Fine {
                            token:
                                Spanned {
                                    node: Token::ClosingBrace,
                                    ..
                                },
                            warnings: new_warnings,
                        }) => {
                            self.opened_braces.pop().unwrap();
                            self.warnings.extend(new_warnings.iter().cloned());
                            if self.opened_braces.len() == brace_level {
                                break 'macro_define;
                            } else {
                                items.push(option_word.unwrap().clone());
                            }
                        }
                        Some(word) => {
                            items.push((*word).clone());
                        }
                        None => break 'macro_define,
                    }
                }
            }
            _ => (),
        }

        if let Some((_, other_span)) = self.macro_definitions.insert(name.clone(), (items, span)) {
            self.errors.push(Error::MacroDefinedMoreThanOnce {
                name: String::from_utf8_lossy(&name).into_owned(),
                span: span.into(),
                other_span: other_span.into(),
            });
        }
        self.unused_macros.insert(name.to_owned());

        words
    }
}

fn scope_identifier(
    identifier: &Identifier,
    scope: &Option<Vec<u8>>,
    span: &Span,
) -> Result<ScopedIdentifier, Error> {
    match identifier {
        Identifier::Label(name) => Ok(ScopedIdentifier::Label(name.clone())),
        Identifier::Path(label, sublabel) => {
            Ok(ScopedIdentifier::Sublabel(label.clone(), sublabel.clone()))
        }
        Identifier::Sublabel(sublabel) => match scope {
            Some(scope_name) => Ok(ScopedIdentifier::Sublabel(
                scope_name.to_owned(),
                sublabel.clone(),
            )),
            None => Err(Error::SublabelReferencedWithoutScope {
                name: String::from_utf8_lossy(&sublabel).into_owned(),
                span: (*span).into(),
            }),
        },
    }
}
