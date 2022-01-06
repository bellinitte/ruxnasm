use super::{Span, Spanned};

pub(crate) enum HexNumber {
    Byte(u8),
    Short(u16),
}

pub(crate) enum Error {
    DigitExpected,
    DigitInvalid { digit: char, span: Span },
    UnevenLength { length: usize },
    TooLong { length: usize },
}

pub(crate) fn parse_hex_number(symbols: &[Spanned<u8>]) -> Result<HexNumber, Error> {
    let mut value: usize = 0;

    for Spanned { node: ch, span } in symbols {
        if let Some(digit) = to_hex_digit(*ch) {
            value = (value << 4) + digit as usize;
        } else {
            return Err(Error::DigitInvalid {
                digit: *ch as char,
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
    TooLong { length: usize },
}

pub(crate) fn parse_hex_number_unconstrained(symbols: &[Spanned<u8>]) -> Result<u16, Error2> {
    let mut value: u16 = 0;

    for Spanned { node: ch, span } in symbols {
        if let Some(digit) = to_hex_digit(*ch) {
            value = (value << 4) + digit as u16;
        } else {
            return Err(Error2::DigitInvalid {
                digit: *ch as char,
                span: *span,
            });
        }
    }

    match symbols.len() {
        0 => Err(Error2::DigitExpected),
        length if length > 4 => Err(Error2::TooLong { length }),
        _ => Ok(value),
    }
}

fn to_hex_digit(c: u8) -> Option<usize> {
    match c {
        b'0'..=b'9' => Some(c as usize - b'0' as usize),
        b'a'..=b'f' => Some(c as usize - b'a' as usize + 10),
        _ => None,
    }
}
