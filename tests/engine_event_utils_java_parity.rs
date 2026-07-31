//! EngineEventUtils 三类文本事件的固定 Java 差分测试。

use std::collections::BTreeMap;
use std::sync::Arc;

use thymeleaf::engine::{CDATASection, Comment, EngineEventUtils, Text};
use thymeleaf::util::{JavaCharSequence, JavaString};

fn golden() -> BTreeMap<String, String> {
    include_str!("fixtures/engine_event_utils_golden.txt")
        .lines()
        .map(|line| {
            let (key, value) = line.split_once('=').expect("golden key/value");
            (key.to_owned(), value.to_owned())
        })
        .collect()
}

fn sequence(value: &str) -> Arc<dyn JavaCharSequence> {
    Arc::new(JavaString::from_rust_str(value))
}

#[test]
fn engine_event_utils_matches_java_text_cdata_and_comment_rules() {
    let fixture = golden();
    assert_eq!(
        fixture["baseline"],
        "10f9dd2eb8cbd98515ce14b149d115e0287d0add"
    );
    assert!(fixture["shape"].contains("computeAttributeExpression("));
    assert!(fixture["shape"].contains("isWhitespace(org.thymeleaf.model.IText)"));

    assert_eq!(
        EngineEventUtils::is_whitespace_text(None)
            .expect("null text")
            .to_string(),
        fixture["text.null.whitespace"]
    );
    for (key, text) in [
        ("text.empty.whitespace", ""),
        ("text.space.whitespace", " \t\n\u{3000}"),
        ("text.figure-space.whitespace", "\u{2007}"),
        ("text.word.whitespace", " a "),
    ] {
        let event = Text::new(Some(sequence(text)));
        assert_eq!(
            EngineEventUtils::is_whitespace_text(Some(&event))
                .expect("text whitespace")
                .to_string(),
            fixture[key]
        );
    }
    for (key, text) in [
        ("text.bracket.inlineable", "x [[${name}]] y"),
        ("text.paren.inlineable", "x [(${name})] y"),
        ("text.malformed.inlineable", "[[$ {name}]"),
    ] {
        let event = Text::new(Some(sequence(text)));
        assert_eq!(
            EngineEventUtils::is_inlineable_text(Some(&event))
                .expect("text inlineable")
                .to_string(),
            fixture[key]
        );
    }

    let cdata_space = CDATASection::new(Some(sequence("\t\u{200A}")));
    let cdata_inline = CDATASection::new(Some(sequence("[[${name}]]")));
    assert_eq!(
        EngineEventUtils::is_whitespace_cdata(Some(&cdata_space))
            .expect("cdata whitespace")
            .to_string(),
        fixture["cdata.space.whitespace"]
    );
    assert_eq!(
        EngineEventUtils::is_inlineable_cdata(Some(&cdata_inline))
            .expect("cdata inlineable")
            .to_string(),
        fixture["cdata.inlineable"]
    );

    let comment_space = Comment::new(Some(sequence("\r\n")));
    let comment_inline = Comment::new(Some(sequence("[(${name})]")));
    assert_eq!(
        EngineEventUtils::is_whitespace_comment(Some(&comment_space))
            .expect("comment whitespace")
            .to_string(),
        fixture["comment.space.whitespace"]
    );
    assert_eq!(
        EngineEventUtils::is_inlineable_comment(Some(&comment_inline))
            .expect("comment inlineable")
            .to_string(),
        fixture["comment.inlineable"]
    );
}
