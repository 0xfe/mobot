/// Test that markdown text is properly escaped.

#[test]
fn escape_code() {
    let code = "hello `world` \\foo";

    assert_eq!(mobot::api::escape_code(code), "hello \\`world\\` \\\\foo");
}

#[test]
fn escape_markdown() {
    let md = "hello *world* [foo](bar) _baz_";

    assert_eq!(
        mobot::api::escape_md(md),
        "hello \\*world\\* \\[foo\\]\\(bar\\) \\_baz\\_"
    );
}
