use oxc_span::SourceType;
use react_compiler_rust::debug_hir;
use std::fs;

#[test]
fn test_fixtures() {
    insta::glob!("../fixtures", "*.js", |path| {
        let input = fs::read_to_string(path).unwrap();
        let source_type = SourceType::from_path(path).unwrap_or_default();
        let output = debug_hir(&input, source_type).unwrap();
        insta::assert_snapshot!(output);
    });
}
