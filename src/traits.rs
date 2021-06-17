use crate::{Error, Warning};

pub trait UnzipCollect<S> {
    fn unzip_collect(self) -> Result<(Vec<S>, Vec<Warning>), (Vec<Error>, Vec<Warning>)>;
}

impl<T, S> UnzipCollect<S> for T
where
    T: Iterator<Item = Result<(S, Option<Warning>), Error>>,
{
    fn unzip_collect(self) -> Result<(Vec<S>, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
        let mut items: Vec<S> = Vec::new();
        let mut warnings: Vec<Warning> = Vec::new();
        let mut errors: Vec<Error> = Vec::new();

        for result in self {
            match result {
                Ok((item, Some(warning))) => {
                    items.push(item);
                    warnings.push(warning);
                }
                Ok((item, None)) => {
                    items.push(item);
                }
                Err(error) => {
                    errors.push(error);
                }
            }
        }

        if errors.is_empty() {
            Ok((items, warnings))
        } else {
            Err((errors, warnings))
        }
    }
}

pub trait Stockpile<T> {
    fn stockpile(self, warnings: &mut Vec<Warning>) -> Result<T, Vec<Error>>;
}

impl<T> Stockpile<T> for Result<(T, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
    fn stockpile(self, warnings: &mut Vec<Warning>) -> Result<T, Vec<Error>> {
        match self {
            Ok((items, new_warnings)) => {
                warnings.extend(new_warnings);
                Ok(items)
            }
            Err((errors, new_warnings)) => {
                warnings.extend(new_warnings);
                Err(errors)
            }
        }
    }
}
