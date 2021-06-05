use ruxnasm::{assemble, Error};
use test_case::test_case;

#[test_case("(" => vec![Error::NoMatchingClosingParenthesis { span: 0..1 }])]
fn scanner_error_tests(source: &str) -> Vec<Error> {
    match assemble(source) {
        Ok(_) => Vec::new(),
        Err((errors, _)) => errors,
    }
}
