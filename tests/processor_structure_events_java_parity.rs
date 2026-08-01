//! ProcessorTemplateHandler 结构事件处理 Java Golden 差分测试。
//!
//! 覆盖：comment、CDATA、DOCTYPE、processing instruction、XML 声明、
//! 属性名事件、文本事件在 HTML/XML 模式下的保留与处理。

use std::sync::Arc;

use thymeleaf::context::Context;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

fn engine(mode: TemplateMode) -> TemplateEngine {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(mode);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

fn render_html(tmpl: &str) -> String {
    engine(TemplateMode::HTML)
        .process_template(tmpl, &Context::new())
        .unwrap()
        .to_string_lossy()
}

fn render_xml(tmpl: &str) -> String {
    engine(TemplateMode::XML)
        .process_template(tmpl, &Context::new())
        .unwrap()
        .to_string_lossy()
}

// ===========================================================================
// 1. Comment 事件
// ===========================================================================

#[test]
fn html_comment_preserved() {
    let input = "<!-- a comment --><p>text</p>";
    assert_eq!(render_html(input), input);
}

#[test]
fn html_comment_contains_tags() {
    let input = "<!-- <div>inside comment</div> --><p>after</p>";
    assert_eq!(render_html(input), input);
}

#[test]
fn xml_comment_preserved() {
    let input = "<?xml version=\"1.0\"?>\n<!-- xml comment --><root/>";
    assert_eq!(render_xml(input), input);
}

// ===========================================================================
// 2. CDATA 事件
// ===========================================================================

#[test]
fn xml_cdata_preserved() {
    let input = "<?xml version=\"1.0\"?>\n<root><![CDATA[<b>raw</b>]]></root>";
    assert_eq!(render_xml(input), input);
}

#[test]
fn html_cdata_preserved() {
    let input = "<![CDATA[raw <content>]]>";
    assert_eq!(render_html(input), input);
}

// ===========================================================================
// 3. DOCTYPE 事件
// ===========================================================================

#[test]
fn html_doctype_preserved() {
    let input = "<!DOCTYPE html>\n<html><body>hi</body></html>";
    assert_eq!(render_html(input), input);
}

#[test]
fn html_doctype_with_public_id() {
    let input = "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Strict//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd\">\n<html/>";
    assert_eq!(render_html(input), input);
}

#[test]
fn html_doctype_with_system_id() {
    let input = "<!DOCTYPE html SYSTEM \"about:legacy-compat\">\n<html/>";
    assert_eq!(render_html(input), input);
}

// ===========================================================================
// 4. Processing Instruction 事件
// ===========================================================================

#[test]
fn xml_processing_instruction_preserved() {
    let input = "<?xml version=\"1.0\"?>\n<?target data?><root/>";
    assert_eq!(render_xml(input), input);
}

// ===========================================================================
// 5. XML 声明事件
// ===========================================================================

#[test]
fn xml_declaration_preserved() {
    let input = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root/>";
    assert_eq!(render_xml(input), input);
}

#[test]
fn xml_declaration_standalone() {
    let input = "<?xml version=\"1.0\" standalone=\"yes\"?>\n<root/>";
    assert_eq!(render_xml(input), input);
}

// ===========================================================================
// 6. 文本与元素混合
// ===========================================================================

#[test]
fn mixed_text_and_elements() {
    let input = "<p>line1<br/>line2<span>inline</span></p>";
    assert_eq!(render_html(input), input);
}

#[test]
fn nested_elements_deep() {
    let input = "<div><section><article><p>deep</p></article></section></div>";
    assert_eq!(render_html(input), input);
}

// ===========================================================================
// 7. 属性事件
// ===========================================================================

#[test]
fn attributes_with_special_chars() {
    let input = "<div class=\"a b\" data-x=\"1\" title='single'>text</div>";
    assert_eq!(render_html(input), input);
}

#[test]
fn attributes_boolean_style() {
    let input = "<input disabled checked/>";
    assert_eq!(render_html(input), input);
}

#[test]
fn attributes_dynamic_values() {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let ctx = Context::new();
    let s = e
        .process_template(
            "<a th:href=\"'http://example.com'\" th:title=\"'My Title'\">link</a>",
            &ctx,
        )
        .unwrap()
        .to_string_lossy();
    assert!(s.contains("http://example.com"));
    assert!(s.contains("My Title"));
}

// ===========================================================================
// 8. 自闭合与空元素
// ===========================================================================

#[test]
fn self_closing_elements() {
    let input = "<br/><hr/><img src=\"x.png\"/><input type=\"text\"/>";
    assert_eq!(render_html(input), input);
}

#[test]
fn void_elements_without_slash() {
    let input = "<br><hr><img src=\"x.png\">";
    assert_eq!(render_html(input), input);
}

// ===========================================================================
// 9. 特殊字符与 Unicode
// ===========================================================================

#[test]
fn text_with_special_chars() {
    let input = "<p>&amp; &lt; &gt; &quot; &#39;</p>";
    assert_eq!(render_html(input), input);
}

#[test]
fn unicode_text() {
    let input = "<p>日本語 🚀 émoji</p>";
    assert_eq!(render_html(input), input);
}

#[test]
fn html_entities_preserved() {
    let input = "<p>&copy; &nbsp; &euro;</p>";
    assert_eq!(render_html(input), input);
}

// ===========================================================================
// 10. 多模板结构组合
// ===========================================================================

#[test]
fn full_document_structure() {
    let input = "<!DOCTYPE html>\n<html>\n<head>\n<title>Title</title>\n</head>\n<body>\n<!-- comment -->\n<p>text</p>\n</body>\n</html>";
    assert_eq!(render_html(input), input);
}

#[test]
fn whitespace_preserved_in_text() {
    let input = "<p>  spaced   out  </p>";
    assert_eq!(render_html(input), input);
}
