use super::{Span, Spanned};

pub enum HexNumber {
    Byte(u8),
    Short(u16),
}

pub enum Error {
    DigitExpected,
    DigitInvalid { digit: char, span: Span },
    UnevenLength { length: usize },
    TooLong { length: usize },
}

pub(crate) fn parse_hex_number(symbols: &[Spanned<char>]) -> Result<HexNumber, Error> {
    let mut value: usize = 0;

    for Spanned { node: ch, span } in symbols {
        if is_hex_digit(*ch) {
            value = (value << 4) + to_hex_digit(*ch).unwrap() as usize;
        } else {
            return Err(Error::DigitInvalid {
                digit: *ch,
                span: *span,
            });
        }
    }

    match symbols.len() {
        0 => Err(Error::DigitExpected),
        1 => Err(Error::UnevenLength { length: 1 }),
        2 => Ok(HexNumber::Byte(value as u8)),
        3 => Err(Error::UnevenLength { length: 3 }),
        4 => Ok(HexNumber::Short(value as u16)),
        length => Err(Error::TooLong { length }),
    }
}

pub(crate) enum Error2 {
    DigitExpected,
    DigitInvalid { digit: char, span: Span },
}

pub(crate) fn parse_hex_number_unconstrained(symbols: &[Spanned<char>]) -> Result<usize, Error2> {
    let mut value: usize = 0;

    for Spanned { node: ch, span } in symbols {
        if is_hex_digit(*ch) {
            value = (value << 4) + to_hex_digit(*ch).unwrap() as usize;
        } else {
            return Err(Error2::DigitInvalid {
                digit: *ch,
                span: *span,
            });
        }
    }

    match symbols.len() {
        0 => Err(Error2::DigitExpected),
        _ => Ok(value),
    }
}

fn to_hex_digit(c: char) -> Option<usize> {
    match c {
        '0'..='9' => Some(c as usize - '0' as usize),
        'a'..='f' => Some(c as usize - 'a' as usize + 10),
        _ => None,
    }
}

fn is_hex_digit(c: char) -> bool {
    to_hex_digit(c).is_some()
}
