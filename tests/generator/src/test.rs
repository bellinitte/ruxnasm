use crate::utils::escape_name;
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Test {
    name: String,
    dirs: Vec<String>,
    path: PathBuf,
}

impl Test {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut components = human_readable_components(path).collect::<Vec<_>>();
        let name = components.pop().unwrap();
        let relative_path = Path::new("tests/suite").join(path);

        if !relative_path.join("input.tal").exists() {
            bail!("Test is missing the `input.tal` file");
        }

        Ok(Self {
            name,
            dirs: components,
            path: relative_path,
        })
    }

    pub fn dirs(&self) -> impl Iterator<Item = String> + '_ {
        self.dirs.iter().cloned()
    }

    pub fn expand(&self) -> TokenStream {
        let name = escape_name(&self.name);
        let input_path_string = self.path.join("input.tal").display().to_string();
        let output_path_string = self.path.join("output.rom").display().to_string();

        quote! {
            #[test]
            fn #name() {
                let input_path = ::std::path::Path::new(#input_path_string);
                let output_path = ::std::path::Path::new(#output_path_string);

                let input = ::std::fs::read(input_path).unwrap();
                let expected_output = if output_path.exists() {
                    Some(::std::fs::read(output_path).unwrap())
                } else {
                    None
                };

                match (assemble(&input), expected_output) {
                    (Ok((actual_binary, _)), Some(expected_binary)) => {
                        if actual_binary != expected_binary {
                            let actual_hex_dump = pretty_hex::pretty_hex(&actual_binary);
                            let expected_hex_dump = pretty_hex::pretty_hex(&expected_binary);
                            print_diff(&expected_hex_dump, &actual_hex_dump);
                            panic!("Found differences in outputs");
                        }
                    },
                    (Ok((actual_binary, _)), None) => {
                        panic!("Expected no output but received some");
                    }
                    (Err((errors, _)), Some(_)) => {
                        panic!("Expected some output but received none");
                    }
                    (Err(_), None) => (),
                }
            }
        }
    }
}

fn human_readable_components<'a>(path: &'a Path) -> impl Iterator<Item = String> + 'a {
    path.components().flat_map(|component| {
        if let Component::Normal(dir) = component {
            Some(dir.to_string_lossy().into_owned())
        } else {
            None
        }
    })
}
