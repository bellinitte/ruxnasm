use std::{
    iter::{Copied, Peekable},
    slice::Iter,
};

pub use crate::anomalies::{Error, Warning};
use crate::{tokenizer::Word, Location, Span, Spanned, Spanning};

const WHITESPACES: [u8; 6] = [b' ', b'\t', b'\n', 0x0b, 0x0c, b'\r'];
const DELIMITERS: [u8; 6] = [b'(', b')', b'[', b']', b'{', b'}'];

fn is_delimiter(x: Option<&u8>) -> bool {
    match x {
        Some(ch) => WHITESPACES.contains(ch) || DELIMITERS.contains(ch),
        None => true,
    }
}

fn is_whitespace(ch: u8) -> bool {
    WHITESPACES.contains(&ch)
}

pub(crate) struct Scanner<'a> {
    chars: Peekable<Copied<Iter<'a, u8>>>,
    location: Location,
}

impl<'a> Scanner<'a> {
    pub fn new(input_file_contents: &'a [u8]) -> Self {
        Self {
            chars: input_file_contents.into_iter().copied().peekable(),
            location: Location { offset: 0 },
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<(Word, Option<Warning>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = 'whitespace: loop {
            match self.chars.next() {
                Some(ch) if is_whitespace(ch) => {
                    self.location += 1;
                    continue;
                }
                Some(b'(') => {
                    let comment_start_location = self.location;
                    self.location += 1;
                    let mut comment_level: usize = 1;

                    'comment: loop {
                        match self.chars.next() {
                            Some(b'(') => {
                                self.location += 1;
                                comment_level += 1;
                            }
                            Some(b')') => {
                                self.location += 1;
                                comment_level -= 1;
                                if comment_level == 0 {
                                    break 'comment;
                                }
                            }
                            Some(_) => {
                                self.location += 1;
                            }
                            None => {
                                return Some(Err(Error::NoMatchingClosingParenthesis {
                                    span: Span::new(comment_start_location).into(),
                                }))
                            }
                        }
                    }
                }
                Some(b')') => {
                    return Some(Err(Error::NoMatchingOpeningParenthesis {
                        span: Span::new(self.location).into(),
                    }))
                }
                Some(ch) => break 'whitespace ch,
                None => return None,
            }
        };

        let mut symbols: Vec<Spanned<u8>> = Vec::new();
        symbols.push((ch).spanning(Span::new(self.location)));
        self.location += 1;
        let mut ignored_start: Option<Location> = None;

        // TODO: Refactor the string scanning
        if ch == b'"' || ch == b'\'' {
            while self.chars.peek().is_some() && !is_whitespace(*self.chars.peek().unwrap()) {
                let ch = self.chars.next().unwrap();
                if symbols.len() < 64 {
                    symbols.push(ch.spanning(Span::new(self.location)));
                } else {
                    if ignored_start.is_none() {
                        ignored_start = Some(self.location);
                    }
                }
                self.location += 1;
            }
        } else {
            while !is_delimiter(self.chars.peek()) {
                let ch = self.chars.next().unwrap();
                if symbols.len() < 64 {
                    symbols.push(ch.spanning(Span::new(self.location)));
                } else {
                    if ignored_start.is_none() {
                        ignored_start = Some(self.location);
                    }
                }
                self.location += 1;
            }
        }

        let word = Word::new(&symbols);

        if let Some(ignored_location) = ignored_start {
            let warning = Warning::TokenTrimmed {
                span: Span {
                    from: ignored_location,
                    to: self.location,
                }
                .into(),
            };
            Some(Ok((word, Some(warning))))
        } else {
            Some(Ok((word, None)))
        }
    }
}
