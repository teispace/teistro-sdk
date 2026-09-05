//! The type-safety guarantees that exist only because these programs do
//! not compile (ADR-0023): a swapped latitude and longitude, a UT1 day
//! passed as TT, a bare number where a quantity is expected.

#[test]
fn illegal_states_do_not_compile() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}
