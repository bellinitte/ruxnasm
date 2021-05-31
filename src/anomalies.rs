use crate::tokenizer;

#[derive(Clone)]
pub enum Warning {
    Tokenizer(tokenizer::Warning),
}

impl From<tokenizer::Warning> for Warning {
    fn from(warning: tokenizer::Warning) -> Self {
        Self::Tokenizer(warning)
    }
}

#[derive(Clone)]
pub enum Error {
    Tokenizer(tokenizer::Error),
}

impl From<tokenizer::Error> for Error {
    fn from(error: tokenizer::Error) -> Self {
        Self::Tokenizer(error)
    }
}

pub struct Anomalies {
    warnings: Vec<Warning>,
    errors: Vec<Error>,
}

impl Anomalies {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn push_warning<T: Into<Warning>>(&mut self, warning: T) {
        self.warnings.push(warning.into());
    }

    pub fn push_error<T: Into<Error>>(&mut self, error: T) {
        self.errors.push(error.into());
    }

    pub fn push_warnings<I: IntoIterator<Item = T>, T: Into<Warning>>(&mut self, warnings: I) {
        self.warnings.extend(warnings.into_iter().map(T::into));
    }

    pub fn push_errors<I: IntoIterator<Item = T>, T: Into<Error>>(&mut self, errors: I) {
        self.errors.extend(errors.into_iter().map(T::into));
    }

    pub fn are_fatal(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn warnings(&self) -> Warnings {
        Warnings {
            warnings: self.warnings.as_slice(),
            counter: 0,
        }
    }

    pub fn errors(&self) -> Errors {
        Errors {
            errors: self.errors.as_slice(),
            counter: 0,
        }
    }
}

pub struct Warnings<'a> {
    warnings: &'a [Warning],
    counter: usize,
}

impl<'a> Iterator for Warnings<'a> {
    type Item = &'a Warning;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.warnings.get(self.counter);
        self.counter += 1;
        result
    }
}

pub struct Errors<'a> {
    errors: &'a [Error],
    counter: usize,
}

impl<'a> Iterator for Errors<'a> {
    type Item = &'a Error;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.errors.get(self.counter);
        self.counter += 1;
        result
    }
}
