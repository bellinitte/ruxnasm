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

pub(crate) fn scan<'a>(input_file_contents: &'a [u8]) -> Result<(Vec<Word>, Vec<Warning>), Error> {
    let mut chars = input_file_contents.into_iter().copied().peekable();
    let mut location = Location { offset: 0 };
    let mut words = Vec::new();
    let mut warnings = Vec::new();

    'chars: loop {
        let ch = 'whitespace: loop {
            match chars.next() {
                Some(ch) if is_whitespace(ch) => {
                    location += 1;
                    continue;
                }
                Some(b'(') => {
                    let comment_start_location = location;
                    location += 1;
                    let mut comment_level: usize = 1;

                    'comment: loop {
                        match chars.next() {
                            Some(b'(') => {
                                location += 1;
                                comment_level += 1;
                            }
                            Some(b')') => {
                                location += 1;
                                comment_level -= 1;
                                if comment_level == 0 {
                                    break 'comment;
                                }
                            }
                            Some(_) => {
                                location += 1;
                            }
                            None => {
                                return Err(Error::NoMatchingClosingParenthesis {
                                    span: Span::new(comment_start_location).into(),
                                })
                            }
                        }
                    }
                }
                Some(b')') => {
                    return Err(Error::NoMatchingOpeningParenthesis {
                        span: Span::new(location).into(),
                    })
                }
                Some(ch) => break 'whitespace ch,
                None => break 'chars,
            }
        };

        let mut symbols: Vec<Spanned<u8>> = Vec::new();
        symbols.push((ch).spanning(Span::new(location)));
        location += 1;
        let mut ignored_start: Option<Location> = None;

        // TODO: Refactor the string scanning
        if ch == b'"' || ch == b'\'' {
            while chars.peek().is_none() || !is_whitespace(*chars.peek().unwrap()) {
                let ch = chars.next().unwrap();
                if symbols.len() < 64 {
                    symbols.push(ch.spanning(Span::new(location)));
                } else {
                    if ignored_start.is_none() {
                        ignored_start = Some(location);
                    }
                }
                location += 1;
            }
        } else {
            while !is_delimiter(chars.peek()) {
                let ch = chars.next().unwrap();
                if symbols.len() < 64 {
                    symbols.push(ch.spanning(Span::new(location)));
                } else {
                    if ignored_start.is_none() {
                        ignored_start = Some(location);
                    }
                }
                location += 1;
            }
        }

        words.push(Word::new(&symbols));

        if let Some(ignored_location) = ignored_start {
            warnings.push(Warning::TokenTrimmed {
                span: Span {
                    from: ignored_location,
                    to: location,
                }
                .into(),
            });
        }
    }

    Ok((words, warnings))
}
