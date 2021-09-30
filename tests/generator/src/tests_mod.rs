use crate::utils::escape_name;
use crate::Test;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default, Debug)]
pub struct TestsMod<'a> {
    tests: BTreeSet<&'a Test>,
    children: BTreeMap<String, Self>,
}

impl<'a> TestsMod<'a> {
    pub fn build(tests: &'a [Test]) -> Self {
        tests.iter().fold(Self::default(), |mut this, test| {
            this.add(test);
            this
        })
    }

    fn add(&mut self, test: &'a Test) {
        let mut this = self;

        for test_dir in test.dirs() {
            this = this.children.entry(test_dir).or_default();
        }

        this.tests.insert(test);
    }

    pub fn expand(&self) -> TokenStream {
        let tests = self.expand_tests();
        let children = self.expand_children();

        quote! {
            #(#tests)*
            #(#children)*
        }
    }

    fn expand_tests(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.tests.iter().map(|test| test.expand())
    }

    fn expand_children(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.children.iter().map(|(name, children)| {
            let name = escape_name(name);
            let children = children.expand();

            quote! {
                mod #name {
                    use super::*;

                    #children
                }
            }
        })
    }
}
