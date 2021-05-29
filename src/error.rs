use crate::tokenizer;

pub enum Error {
    Tokenizer(tokenizer::Error),
}

impl From<tokenizer::Error> for Error {
    fn from(error: tokenizer::Error) -> Self {
        Self::Tokenizer(error)
    }
}
