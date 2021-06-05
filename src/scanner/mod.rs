use crate::{tokenizer::Word, Location, Span, Spanned, Spanning};
pub use anomalies::{Error, Warning};

mod anomalies;

const WHITESPACES: [char; 6] = [' ', '\t', '\n', 0x0b as char, 0x0c as char, '\r'];
const DELIMITERS: [char; 6] = ['(', ')', '[', ']', '{', '}'];

fn is_delimiter(x: Option<&char>) -> bool {
    match x {
        Some(ch) => WHITESPACES.contains(ch) || DELIMITERS.contains(ch),
        None => true,
    }
}

fn is_whitespace(ch: char) -> bool {
    WHITESPACES.contains(&ch)
}

pub fn scan<'a>(
    input_file_contents: &'a str,
) -> Result<(Vec<Word>, Vec<crate::Warning>), crate::Error> {
    let mut chars = input_file_contents.chars().peekable();
    let mut location = Location { offset: 0 };
    let mut words = Vec::new();
    let mut warnings = Vec::new();

    'chars: loop {
        let ch = 'whitespace: loop {
            match chars.next() {
                // TODO: include other whitespace
                Some(ch) if is_whitespace(ch) => {
                    location += 1;
                    continue;
                }
                Some('(') => {
                    let comment_start_location = location;
                    location += 1;
                    let mut comment_level: usize = 1;

                    'comment: loop {
                        match chars.next() {
                            Some('(') => {
                                location += 1;
                                comment_level += 1;
                            }
                            Some(')') => {
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
                                    span: Span::new(comment_start_location),
                                }
                                .into())
                            }
                        }
                    }
                }
                Some(')') => {
                    return Err(Error::NoMatchingOpeningParenthesis {
                        span: Span::new(location),
                    }
                    .into())
                }
                Some(ch) => break 'whitespace ch,
                None => break 'chars,
            }
        };

        let mut symbols: Vec<Spanned<char>> = Vec::new();
        symbols.push(ch.spanning(Span::new(location)));
        location += 1;
        let mut ignored_start: Option<Location> = None;

        // TODO: Refactor the string scanning
        if ch == '"' || ch == '\'' {
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
            warnings.push(
                Warning::TokenTrimmed {
                    span: Span {
                        from: ignored_location,
                        to: location,
                    },
                }
                .into(),
            );
        }
    }

    Ok((words, warnings))
}
