mod test;
mod tests;
mod tests_mod;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use test::Test;
use tests::Tests;
use tests_mod::TestsMod;

#[proc_macro]
pub fn generate_tests(_: TokenStream) -> TokenStream {
    let tests = match Tests::discover("tests/suite") {
        Ok(tests) => tests,
        Err(err) => {
            panic!("\n{:?}", err);
        }
    };

    let tests = tests.expand();

    (quote! {
        #tests
    })
    .into()
}
