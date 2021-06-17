use ruxnasm::{assemble, Error, Error::*};
use test_case::test_case;

macro_rules! ev {
    [] => {{
        Vec::<Error>::new()
    }};

    [$($error: expr),+] => {{
        let mut vec = Vec::<Error>::new();
        $( vec.push($error); )+
        vec
    }}
}

#[test_case("("       => ev![NoMatchingClosingParenthesis { span: 0..1 }] ; "test 1")]
#[test_case("()"      => ev![]                                            ; "test 2")]
#[test_case("( )"     => ev![]                                            ; "test 3")]
#[test_case("(  )"    => ev![]                                            ; "test 4")]
#[test_case("( ( )"   => ev![NoMatchingClosingParenthesis { span: 0..1 }] ; "test 5")]
#[test_case("( ( ) )" => ev![]                                            ; "test 6")]
#[test_case("( () )"  => ev![]                                            ; "test 7")]
#[test_case("(())"    => ev![]                                            ; "test 8")]
#[test_case(")"       => ev![NoMatchingOpeningParenthesis { span: 0..1 }] ; "test 9")]
#[test_case("( ) )"   => ev![NoMatchingOpeningParenthesis { span: 4..5 }] ; "test 10")]
#[test_case("( ))"    => ev![NoMatchingOpeningParenthesis { span: 3..4 }] ; "test 11")]
#[test_case("() )"    => ev![NoMatchingOpeningParenthesis { span: 3..4 }] ; "test 12")]
fn scanner_error_tests(source: &str) -> Vec<Error> {
    match assemble(source.as_bytes()) {
        Ok(_) => Vec::new(),
        Err((errors, _)) => errors,
    }
}
