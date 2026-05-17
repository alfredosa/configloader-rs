#[test]
fn malformed_string_attributes_fail_to_compile() {
    let t = trybuild::TestCases::new();

    t.compile_fail("tests/ui/malformed_default_attr.rs");
    t.compile_fail("tests/ui/malformed_env_attr.rs");
    t.compile_fail("tests/ui/malformed_prefix_attr.rs");
}
