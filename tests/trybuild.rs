

#[test] #[ignore]
fn test_trybuild() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild/*.rs");
}

