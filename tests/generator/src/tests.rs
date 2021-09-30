use crate::{Test, TestsMod};
use anyhow::{bail, Context as _, Result};
use proc_macro2::TokenStream;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Tests {
    tests: Vec<Test>,
}

impl Tests {
    pub fn discover(dir: impl AsRef<Path>) -> Result<Self> {
        if !dir.as_ref().exists() {
            bail!("Cound't find the test suite");
        }

        let tests = fs::read_to_string(dir.as_ref().join("index"))
            .with_context(|| format!("Couldn't find the test suite's index"))?
            .lines()
            .map(|line| line.into())
            .map(|path: PathBuf| {
                Test::load(&path)
                    .with_context(|| format!("Couldn't load test: {}", path.display(),))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { tests })
    }

    pub fn expand(&self) -> TokenStream {
        TestsMod::build(&self.tests).expand()
    }
}
