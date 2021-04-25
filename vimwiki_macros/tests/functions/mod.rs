// mod vimwiki;
// mod vimwiki_format;
//
#[test]
fn test() {
    use vimwiki_macros::*;
    let _ = vimwiki_definition_list_format!(
        r#"
            term:: {} definition
            term2 {}::
            :: def 2
            :: def {} 3
        "#,
        "first",
        "second",
        "third",
    );
}
