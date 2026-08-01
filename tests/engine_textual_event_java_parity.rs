//! 引擎文本类事件 Java 1:1 差分测试。
//!
//! 逐一对应上游 `thymeleaf-tests-core` 的
//! `org.thymeleaf.engine` 包单元测试：
//! - TextTest（构造/位置/subSequence/内容标志）
//! - CommentTest（构造/边界/subSequence/内容标志）
//! - CDATASectionTest（构造/边界/subSequence/内容标志）
//! - DocTypeTest（全部构造器组合与字段分解）
//! - XmlDeclarationTest（全部构造器组合与字段分解）
//! - ProcessingInstructionTest（target/content 组合与字段分解）

use std::sync::Arc;

use thymeleaf::engine::{
    CDATASection, Comment, DocType, EngineEventUtils, ProcessingInstruction, Text, XMLDeclaration,
};
use thymeleaf::model::{
    ICDATASection, IComment, IDocType, IProcessingInstruction, ITemplateEvent, IText,
    IXMLDeclaration,
};
use thymeleaf::util::{JavaCharSequence, JavaString};

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn sequence(value: &str) -> Arc<dyn JavaCharSequence> {
    Arc::new(JavaString::from_rust_str(value))
}

/// 逐字符提取（对应上游 extractText 辅助方法）。
fn extract_text(sequence: &dyn JavaCharSequence) -> String {
    let mut result = String::new();
    let length = sequence.java_length().expect("sequence length").max(0) as usize;
    for i in 0..length {
        let c = sequence.java_char_at(i as i32).expect("char at index");
        result.push(char::from_u32(c as u32).expect("valid char"));
    }
    result
}

// ===========================================================================
// 1. TextTest
// ===========================================================================

#[test]
fn text_construction_and_location() {
    // 对应 Java TextTest#test 第一部分
    let c1 = Text::with_location(Some(sequence("hello")), Some(js("template")), 10, 3);
    assert_eq!(extract_text(&c1), "hello");
    assert_eq!(
        c1.get_text()
            .expect("text")
            .expect("content")
            .to_string_lossy(),
        "hello"
    );
    assert_eq!(
        c1.get_template_name().unwrap().to_string_lossy(),
        "template"
    );
    assert_eq!(c1.get_line(), 10);
    assert_eq!(c1.get_col(), 3);

    // 对应 Java TextTest#test 第二部分（无位置）
    let c1 = Text::new(Some(sequence(" something\nhere ")));
    assert_eq!(
        c1.get_text()
            .expect("text")
            .expect("content")
            .to_string_lossy(),
        " something\nhere "
    );
    assert!(c1.get_template_name().is_none());
    assert_eq!(c1.get_line(), -1);
    assert_eq!(c1.get_col(), -1);
}

#[test]
fn text_subsection() {
    // 对应 Java TextTest#testSubsection
    let c1 = Text::new(Some(sequence("something")));
    assert_eq!(
        c1.java_sub_sequence(4, 9)
            .expect("subsequence")
            .to_string_lossy(),
        "thing"
    );
    assert_eq!(
        c1.java_sub_sequence(0, 4)
            .expect("subsequence")
            .to_string_lossy(),
        "some"
    );

    let c1 = Text::with_location(Some(sequence("something")), Some(js("test")), 1, 1);
    assert_eq!(
        c1.java_sub_sequence(4, 9)
            .expect("subsequence")
            .to_string_lossy(),
        "thing"
    );
    assert_eq!(
        c1.java_sub_sequence(0, 4)
            .expect("subsequence")
            .to_string_lossy(),
        "some"
    );
}

/// 上游 TextTest/CommentTest/CDATASectionTest 共用的内容标志表
/// （text, whitespace, inlineable）——逐字转录。
const CONTENT_FLAGS: &[(&str, bool, bool)] = &[
    ("", false, false),
    (" ", true, false),
    ("   ", true, false),
    ("\n", true, false),
    ("\n  \t", true, false),
    ("\n  [asd]", false, false),
    ("\n  asdasdasd 23123 [ [asd ]]", false, false),
    ("\n  asdasdasd 23123 [[asd ]]", false, true),
    ("\n  asdasdasd 23123 [[asd ]]    [[asd]]", false, true),
    ("\n  asdasdasd 23123  [ [asd ]]    [[asd] ]", false, false),
    ("[[asd]]", false, true),
    ("[[asd]", false, false),
    ("[asd]]", false, false),
    ("]]", false, false),
    ("[[", false, false),
    ("[[asd]]asd", false, true),
    ("asd[[asd]]", false, true),
    ("asd[[asd]]asd", false, true),
    ("\n  (asd)", false, false),
    ("\n  asdasdasd 23123 [ (asd )]", false, false),
    ("\n  asdasdasd 23123 [(asd )]", false, true),
    ("\n  asdasdasd 23123 [(asd )]    [(asd)]", false, true),
    ("\n  asdasdasd 23123  [ (asd )]    [(asd) ]", false, false),
    ("[(asd)]", false, true),
    ("[(asd)", false, false),
    ("[asd)]", false, false),
    (")]", false, false),
    ("[(", false, false),
    ("[(asd)]asd", false, true),
    ("asd[(asd)]", false, true),
    ("asd[(asd)]asd", false, true),
    ("\n  (asd)", false, false),
    ("\n  asdasdasd 23123 [ (asd ]]", false, false),
    ("\n  asdasdasd 23123 [[asd )]", false, false),
    ("\n  asdasdasd 23123 [(asd ]]    [[asd)]", false, false),
    ("\n  asdasdasd 23123  [ (asd ]]    [(asd) ]", false, false),
    ("[(asd]]", false, false),
    ("[(asd]", false, false),
    ("(asd)]", false, false),
    ("[(asd]]asd", false, false),
    ("asd[[asd)]", false, false),
    ("asd[(asd]])asd", false, false),
];

#[test]
fn text_content_flags() {
    // 对应 Java TextTest#testContentFlags（两种构造器各跑一遍）
    for (text, whitespace, inlineable) in CONTENT_FLAGS {
        let t1 = Text::new(Some(sequence(text)));
        assert_eq!(
            t1.is_whitespace().expect("whitespace"),
            *whitespace,
            "Text::new({text:?}) whitespace"
        );
        assert_eq!(
            t1.is_inlineable().expect("inlineable"),
            *inlineable,
            "Text::new({text:?}) inlineable"
        );

        let t1 = Text::with_location(Some(sequence(text)), Some(js("test")), 1, 1);
        assert_eq!(
            t1.is_whitespace().expect("whitespace"),
            *whitespace,
            "Text+loc({text:?}) whitespace"
        );
        assert_eq!(
            t1.is_inlineable().expect("inlineable"),
            *inlineable,
            "Text+loc({text:?}) inlineable"
        );
    }
}

// ===========================================================================
// 2. CommentTest
// ===========================================================================

#[test]
fn comment_construction_and_boundaries() {
    // 对应 Java CommentTest#test 第一部分
    let c1 = Comment::with_location(Some(sequence("hello")), Some(js("template")), 10, 3);
    assert_eq!(extract_text(&c1), "<!--hello-->");
    assert_eq!(
        c1.get_comment()
            .expect("comment")
            .expect("content")
            .to_string_lossy(),
        "<!--hello-->"
    );
    assert_eq!(
        c1.get_content()
            .expect("content")
            .expect("content")
            .to_string_lossy(),
        "hello"
    );
    assert_eq!(
        c1.get_template_name().unwrap().to_string_lossy(),
        "template"
    );
    assert_eq!(c1.get_line(), 10);
    assert_eq!(c1.get_col(), 3);

    // 对应 Java CommentTest#test 第二部分（无位置）
    let c1 = Comment::new(Some(sequence(" something\nhere ")));
    assert_eq!(
        c1.get_content()
            .expect("content")
            .expect("content")
            .to_string_lossy(),
        " something\nhere "
    );
    assert_eq!(
        c1.get_comment()
            .expect("comment")
            .expect("content")
            .to_string_lossy(),
        "<!-- something\nhere -->"
    );
    assert!(c1.get_template_name().is_none());
    assert_eq!(c1.get_line(), -1);
    assert_eq!(c1.get_col(), -1);
}

#[test]
fn comment_subsection() {
    // 对应 Java CommentTest#testSubsection（全文含 <!-- --> 的切片）
    let c1 = Comment::new(Some(sequence("something")));
    assert_eq!(
        c1.java_sub_sequence(1, 5)
            .expect("subsequence")
            .to_string_lossy(),
        "!--s"
    );
    assert_eq!(
        c1.java_sub_sequence(4, 8)
            .expect("subsequence")
            .to_string_lossy(),
        "some"
    );

    let c1 = Comment::with_location(Some(sequence("something")), Some(js("test")), 1, 1);
    assert_eq!(
        c1.java_sub_sequence(1, 5)
            .expect("subsequence")
            .to_string_lossy(),
        "!--s"
    );
    assert_eq!(
        c1.java_sub_sequence(4, 8)
            .expect("subsequence")
            .to_string_lossy(),
        "some"
    );
}

#[test]
fn comment_content_flags() {
    // 对应 Java CommentTest#testContentFlags。
    // 上游同时用标准边界与自定义边界构造验证标志仅与内容相关。
    for (text, whitespace, inlineable) in CONTENT_FLAGS {
        let t1 = Comment::new(Some(sequence(text)));
        assert_eq!(
            EngineEventUtils::is_whitespace_comment(Some(&t1)).expect("whitespace"),
            *whitespace,
            "Comment::new({text:?}) whitespace"
        );
        assert_eq!(
            EngineEventUtils::is_inlineable_comment(Some(&t1)).expect("inlineable"),
            *inlineable,
            "Comment::new({text:?}) inlineable"
        );

        // 上游: new Comment("<!--  ", text, "  -->")
        let t1 = Comment::with_boundaries(js("<!--  "), Some(sequence(text)), js("  -->"));
        assert_eq!(
            EngineEventUtils::is_whitespace_comment(Some(&t1)).expect("whitespace"),
            *whitespace,
            "Comment+boundaries({text:?}) whitespace"
        );
        assert_eq!(
            EngineEventUtils::is_inlineable_comment(Some(&t1)).expect("inlineable"),
            *inlineable,
            "Comment+boundaries({text:?}) inlineable"
        );
    }
}

// ===========================================================================
// 3. CDATASectionTest
// ===========================================================================

#[test]
fn cdata_section_construction_and_boundaries() {
    // 对应 Java CDATASectionTest#test 第一、三部分
    let c1 = CDATASection::with_location(Some(sequence("hello")), Some(js("testtemplate")), 10, 3);
    assert_eq!(extract_text(&c1), "<![CDATA[hello]]>");
    assert_eq!(
        c1.get_cdata_section()
            .expect("cdata")
            .expect("content")
            .to_string_lossy(),
        "<![CDATA[hello]]>"
    );
    assert_eq!(
        c1.get_content()
            .expect("content")
            .expect("content")
            .to_string_lossy(),
        "hello"
    );
    assert_eq!(
        c1.get_template_name().unwrap().to_string_lossy(),
        "testtemplate"
    );
    assert_eq!(c1.get_line(), 10);
    assert_eq!(c1.get_col(), 3);

    // 对应 Java CDATASectionTest#test 第二部分（无位置）
    let c1 = CDATASection::new(Some(sequence(" something\nhere ")));
    assert_eq!(
        c1.get_content()
            .expect("content")
            .expect("content")
            .to_string_lossy(),
        " something\nhere "
    );
    assert_eq!(
        c1.get_cdata_section()
            .expect("cdata")
            .expect("content")
            .to_string_lossy(),
        "<![CDATA[ something\nhere ]]>"
    );
    assert!(c1.get_template_name().is_none());
    assert_eq!(c1.get_line(), -1);
    assert_eq!(c1.get_col(), -1);

    // 对应 Java: new CDATASection("<![cdata[", content, CDATA_SUFFIX, "testtemplate", 11, 4)
    let c1 = CDATASection::with_boundaries_and_location(
        js("<![cdata["),
        Some(sequence(" something\nhere ")),
        js("]]>"),
        Some(js("testtemplate")),
        11,
        4,
    );
    assert_eq!(
        c1.get_cdata_section()
            .expect("cdata")
            .expect("content")
            .to_string_lossy(),
        "<![cdata[ something\nhere ]]>"
    );
    assert_eq!(
        c1.get_content()
            .expect("content")
            .expect("content")
            .to_string_lossy(),
        " something\nhere "
    );
    assert_eq!(
        c1.get_template_name().unwrap().to_string_lossy(),
        "testtemplate"
    );
    assert_eq!(c1.get_line(), 11);
    assert_eq!(c1.get_col(), 4);
}

#[test]
fn cdata_section_subsection() {
    // 对应 Java CDATASectionTest#testSubsection（全文含 <![CDATA[ ]]> 的切片）
    let c1 = CDATASection::new(Some(sequence("something")));
    assert_eq!(
        c1.java_sub_sequence(3, 8)
            .expect("subsequence")
            .to_string_lossy(),
        "CDATA"
    );
    assert_eq!(
        c1.java_sub_sequence(9, 13)
            .expect("subsequence")
            .to_string_lossy(),
        "some"
    );

    let c1 = CDATASection::with_location(Some(sequence("something")), Some(js("test")), 1, 1);
    assert_eq!(
        c1.java_sub_sequence(3, 8)
            .expect("subsequence")
            .to_string_lossy(),
        "CDATA"
    );
    assert_eq!(
        c1.java_sub_sequence(9, 13)
            .expect("subsequence")
            .to_string_lossy(),
        "some"
    );
}

#[test]
fn cdata_section_content_flags() {
    // 对应 Java CDATASectionTest#testContentFlags
    for (text, whitespace, inlineable) in CONTENT_FLAGS {
        let t1 = CDATASection::new(Some(sequence(text)));
        assert_eq!(
            EngineEventUtils::is_whitespace_cdata(Some(&t1)).expect("whitespace"),
            *whitespace,
            "CDATASection::new({text:?}) whitespace"
        );
        assert_eq!(
            EngineEventUtils::is_inlineable_cdata(Some(&t1)).expect("inlineable"),
            *inlineable,
            "CDATASection::new({text:?}) inlineable"
        );

        let t1 = CDATASection::with_location(Some(sequence(text)), Some(js("test")), 1, 1);
        assert_eq!(
            EngineEventUtils::is_whitespace_cdata(Some(&t1)).expect("whitespace"),
            *whitespace,
            "CDATASection+loc({text:?}) whitespace"
        );
        assert_eq!(
            EngineEventUtils::is_inlineable_cdata(Some(&t1)).expect("inlineable"),
            *inlineable,
            "CDATASection+loc({text:?}) inlineable"
        );
    }
}

// ===========================================================================
// 4. DocTypeTest
// ===========================================================================

const DOCTYPE_XHTML_TRANSITIONAL: &str = "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\">";
const DOCTYPE_XHTML_TRANSITIONAL_WS: &str = "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\" [\n <!-- an internal subset can be embedded here -->\n ]>";
const DOCTYPE_XHTML_TRANSITIONAL_KLC: &str = "<!doctype html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\">";
const KEYWORD_UC: &str = "DOCTYPE";
const KEYWORD_LC: &str = "doctype";
const ELEMENT_NAME_HTML: &str = "html";
const TYPE_PUBLIC_UC: &str = "PUBLIC";
const TYPE_SYSTEM_UC: &str = "SYSTEM";
const PUBLIC_ID_XHTML_TRANSITIONAL: &str = "-//W3C//DTD XHTML 1.0 Transitional//EN";
const SYSTEM_ID_XHTML_TRANSITIONAL: &str =
    "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd";
const INTERNAL_SUBSET_WS_XHTML_TRANSITIONAL: &str =
    "\n <!-- an internal subset can be embedded here -->\n ";

#[test]
fn doc_type_full_constructor_with_location() {
    // 对应 Java DocTypeTest#test 前两段（9 参构造器）
    let d1 = DocType::with_location(
        Some(js(DOCTYPE_XHTML_TRANSITIONAL_WS)),
        Some(js(KEYWORD_UC)),
        Some(js(ELEMENT_NAME_HTML)),
        Some(js(PUBLIC_ID_XHTML_TRANSITIONAL)),
        Some(js(SYSTEM_ID_XHTML_TRANSITIONAL)),
        Some(js(INTERNAL_SUBSET_WS_XHTML_TRANSITIONAL)),
        Some(js("template")),
        11,
        4,
    )
    .expect("valid doctype");
    assert_eq!(
        d1.get_doc_type().unwrap().to_string_lossy(),
        DOCTYPE_XHTML_TRANSITIONAL_WS
    );
    assert_eq!(d1.get_keyword().unwrap().to_string_lossy(), KEYWORD_UC);
    assert_eq!(
        d1.get_element_name().unwrap().to_string_lossy(),
        ELEMENT_NAME_HTML
    );
    assert_eq!(d1.get_type().unwrap().to_string_lossy(), TYPE_PUBLIC_UC);
    assert_eq!(
        d1.get_public_id().unwrap().to_string_lossy(),
        PUBLIC_ID_XHTML_TRANSITIONAL
    );
    assert_eq!(
        d1.get_system_id().unwrap().to_string_lossy(),
        SYSTEM_ID_XHTML_TRANSITIONAL
    );
    assert_eq!(
        d1.get_internal_subset().unwrap().to_string_lossy(),
        INTERNAL_SUBSET_WS_XHTML_TRANSITIONAL
    );
    assert_eq!(
        d1.get_template_name().unwrap().to_string_lossy(),
        "template"
    );
    assert_eq!(d1.get_line(), 11);
    assert_eq!(d1.get_col(), 4);

    let d1 = DocType::with_location(
        Some(js(DOCTYPE_XHTML_TRANSITIONAL_WS)),
        Some(js(KEYWORD_UC)),
        Some(js(ELEMENT_NAME_HTML)),
        Some(js(PUBLIC_ID_XHTML_TRANSITIONAL)),
        Some(js(SYSTEM_ID_XHTML_TRANSITIONAL)),
        Some(js(INTERNAL_SUBSET_WS_XHTML_TRANSITIONAL)),
        Some(js("template")),
        10,
        3,
    )
    .expect("valid doctype");
    assert_eq!(d1.get_line(), 10);
    assert_eq!(d1.get_col(), 3);
}

#[test]
fn doc_type_computed_forms() {
    // 对应 Java DocTypeTest#test 第三至六段（5 参构造器计算）
    let d1 = DocType::with_components(
        Some(js(KEYWORD_UC)),
        Some(js(ELEMENT_NAME_HTML)),
        Some(js(PUBLIC_ID_XHTML_TRANSITIONAL)),
        Some(js(SYSTEM_ID_XHTML_TRANSITIONAL)),
        None,
    )
    .expect("valid doctype");
    assert_eq!(
        d1.get_doc_type().unwrap().to_string_lossy(),
        DOCTYPE_XHTML_TRANSITIONAL
    );
    assert_eq!(d1.get_keyword().unwrap().to_string_lossy(), KEYWORD_UC);
    assert_eq!(
        d1.get_element_name().unwrap().to_string_lossy(),
        ELEMENT_NAME_HTML
    );
    assert_eq!(d1.get_type().unwrap().to_string_lossy(), TYPE_PUBLIC_UC);
    assert!(d1.get_public_id().is_some());
    assert!(d1.get_system_id().is_some());
    assert!(d1.get_internal_subset().is_none());
    assert!(d1.get_template_name().is_none());
    assert_eq!(d1.get_line(), -1);
    assert_eq!(d1.get_col(), -1);

    // 小写 keyword 原样保留
    let d1 = DocType::with_components(
        Some(js(KEYWORD_LC)),
        Some(js(ELEMENT_NAME_HTML)),
        Some(js(PUBLIC_ID_XHTML_TRANSITIONAL)),
        Some(js(SYSTEM_ID_XHTML_TRANSITIONAL)),
        None,
    )
    .expect("valid doctype");
    assert_eq!(
        d1.get_doc_type().unwrap().to_string_lossy(),
        DOCTYPE_XHTML_TRANSITIONAL_KLC
    );
    assert_eq!(d1.get_keyword().unwrap().to_string_lossy(), KEYWORD_LC);
    assert_eq!(d1.get_type().unwrap().to_string_lossy(), TYPE_PUBLIC_UC);
    assert!(d1.get_internal_subset().is_none());

    // 任意 PUBLIC/SYSTEM 值
    let d1 = DocType::with_components(
        Some(js(KEYWORD_LC)),
        Some(js(ELEMENT_NAME_HTML)),
        Some(js("something")),
        Some(js("someother")),
        None,
    )
    .expect("valid doctype");
    assert_eq!(
        d1.get_doc_type().unwrap().to_string_lossy(),
        "<!doctype html PUBLIC \"something\" \"someother\">"
    );
    assert_eq!(d1.get_keyword().unwrap().to_string_lossy(), KEYWORD_LC);
    assert_eq!(d1.get_public_id().unwrap().to_string_lossy(), "something");
    assert_eq!(d1.get_system_id().unwrap().to_string_lossy(), "someother");

    // SYSTEM-only（无 PUBLIC）
    let d1 = DocType::with_components(
        Some(js(KEYWORD_LC)),
        Some(js(ELEMENT_NAME_HTML)),
        None,
        Some(js("someother")),
        None,
    )
    .expect("valid doctype");
    assert_eq!(
        d1.get_doc_type().unwrap().to_string_lossy(),
        "<!doctype html SYSTEM \"someother\">"
    );
    assert_eq!(d1.get_type().unwrap().to_string_lossy(), TYPE_SYSTEM_UC);
    assert!(d1.get_public_id().is_none());
    assert_eq!(d1.get_system_id().unwrap().to_string_lossy(), "someother");

    // PUBLIC 无 SYSTEM → Java IllegalArgumentException 等价
    let result = DocType::with_components(
        Some(js(KEYWORD_UC)),
        Some(js(ELEMENT_NAME_HTML)),
        Some(js("something")),
        None,
        None,
    );
    assert!(
        result.is_err(),
        "PUBLIC without SYSTEM must be rejected (Java IllegalArgumentException)"
    );
}

#[test]
fn doc_type_default_forms() {
    // 对应 Java DocTypeTest#test 第七段与 d2/d3/d4
    // new DocType(null, null) → HTML5
    let d1 = DocType::with_ids(None, None).expect("valid doctype");
    assert_eq!(
        d1.get_doc_type().unwrap().to_string_lossy(),
        "<!DOCTYPE html>"
    );
    assert_eq!(d1.get_keyword().unwrap().to_string_lossy(), KEYWORD_UC);
    assert_eq!(
        d1.get_element_name().unwrap().to_string_lossy(),
        ELEMENT_NAME_HTML
    );
    assert!(d1.get_type().is_none());
    assert!(d1.get_public_id().is_none());
    assert!(d1.get_system_id().is_none());
    assert!(d1.get_internal_subset().is_none());
    assert!(d1.get_template_name().is_none());
    assert_eq!(d1.get_line(), -1);
    assert_eq!(d1.get_col(), -1);

    // d2: 5 参含 internal subset
    let d2 = DocType::with_components(
        Some(js(KEYWORD_UC)),
        Some(js(ELEMENT_NAME_HTML)),
        Some(js(PUBLIC_ID_XHTML_TRANSITIONAL)),
        Some(js(SYSTEM_ID_XHTML_TRANSITIONAL)),
        Some(js(INTERNAL_SUBSET_WS_XHTML_TRANSITIONAL)),
    )
    .expect("valid doctype");
    assert_eq!(
        d2.get_doc_type().unwrap().to_string_lossy(),
        DOCTYPE_XHTML_TRANSITIONAL_WS
    );
    assert_eq!(
        d2.get_internal_subset().unwrap().to_string_lossy(),
        INTERNAL_SUBSET_WS_XHTML_TRANSITIONAL
    );
    assert!(d2.get_template_name().is_none());
    assert_eq!(d2.get_line(), -1);
    assert_eq!(d2.get_col(), -1);

    // d2 去除 internal subset 后重新计算
    let d2 = DocType::with_components(
        Some(js(KEYWORD_UC)),
        Some(js(ELEMENT_NAME_HTML)),
        Some(js(PUBLIC_ID_XHTML_TRANSITIONAL)),
        Some(js(SYSTEM_ID_XHTML_TRANSITIONAL)),
        None,
    )
    .expect("valid doctype");
    assert_eq!(
        d2.get_doc_type().unwrap().to_string_lossy(),
        DOCTYPE_XHTML_TRANSITIONAL
    );
    assert!(d2.get_internal_subset().is_none());

    // d3: new DocType(publicId, systemId) → 默认 keyword/element
    let d3 = DocType::with_ids(
        Some(js(PUBLIC_ID_XHTML_TRANSITIONAL)),
        Some(js(SYSTEM_ID_XHTML_TRANSITIONAL)),
    )
    .expect("valid doctype");
    assert_eq!(
        d3.get_doc_type().unwrap().to_string_lossy(),
        DOCTYPE_XHTML_TRANSITIONAL
    );
    assert_eq!(d3.get_keyword().unwrap().to_string_lossy(), KEYWORD_UC);
    assert_eq!(
        d3.get_element_name().unwrap().to_string_lossy(),
        ELEMENT_NAME_HTML
    );
    assert_eq!(d3.get_type().unwrap().to_string_lossy(), TYPE_PUBLIC_UC);
    assert!(d3.get_internal_subset().is_none());

    // d4: DocType() 默认构造器
    let d4 = DocType::new().expect("valid doctype");
    assert_eq!(
        d4.get_doc_type().unwrap().to_string_lossy(),
        "<!DOCTYPE html>"
    );
    assert_eq!(d4.get_keyword().unwrap().to_string_lossy(), KEYWORD_UC);
    assert!(d4.get_type().is_none());
}

// ===========================================================================
// 5. XmlDeclarationTest
// ===========================================================================

const XML_DECLAR_1_UTF_NO: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\" ?>";
const XML_DECLAR_1_UTF: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>";
const XML_DECLAR_1: &str = "<?xml version=\"1.0\"?>";
const XML_DECLAR_1_ISO_NO: &str =
    "<?xml version=\"1.0\" encoding=\"ISO-8859-1\" standalone=\"no\"?>";
const XML_DECLAR_11_ISO_YES: &str =
    "<?xml version=\"1.1\" encoding=\"ISO-8859-1\" standalone=\"yes\"?>";
const VERSION_1: &str = "1.0";
const VERSION_11: &str = "1.1";
const ENCODING_UTF: &str = "UTF-8";
const ENCODING_ISO: &str = "ISO-8859-1";
const STANDALONE_NO: &str = "no";
const STANDALONE_YES: &str = "yes";

#[test]
fn xml_declaration_full_constructor_with_location() {
    // 对应 Java XmlDeclarationTest#test 前两段（8 参构造器）
    let d1 = XMLDeclaration::with_location(
        Some(js(XML_DECLAR_1_UTF_NO)),
        Some(js("xml")),
        Some(js(VERSION_1)),
        Some(js(ENCODING_UTF)),
        Some(js(STANDALONE_NO)),
        Some(js("template")),
        11,
        4,
    );
    assert_eq!(
        d1.get_xml_declaration().unwrap().to_string_lossy(),
        XML_DECLAR_1_UTF_NO
    );
    assert_eq!(d1.get_keyword().unwrap().to_string_lossy(), "xml");
    assert_eq!(d1.get_version().unwrap().to_string_lossy(), VERSION_1);
    assert_eq!(d1.get_encoding().unwrap().to_string_lossy(), ENCODING_UTF);
    assert_eq!(
        d1.get_standalone().unwrap().to_string_lossy(),
        STANDALONE_NO
    );
    assert_eq!(
        d1.get_template_name().unwrap().to_string_lossy(),
        "template"
    );
    assert_eq!(d1.get_line(), 11);
    assert_eq!(d1.get_col(), 4);

    let d1 = XMLDeclaration::with_location(
        Some(js(XML_DECLAR_1_UTF_NO)),
        Some(js("xml")),
        Some(js(VERSION_1)),
        Some(js(ENCODING_UTF)),
        Some(js(STANDALONE_NO)),
        Some(js("template")),
        10,
        3,
    );
    assert_eq!(d1.get_line(), 10);
    assert_eq!(d1.get_col(), 3);
}

#[test]
fn xml_declaration_computed_forms() {
    // 对应 Java XmlDeclarationTest#test 第三至五段（4 参构造器计算）
    let d1 = XMLDeclaration::with_components(
        Some(js("xml")),
        Some(js(VERSION_1)),
        Some(js(ENCODING_UTF)),
        None,
    );
    assert_eq!(
        d1.get_xml_declaration().unwrap().to_string_lossy(),
        XML_DECLAR_1_UTF
    );
    assert_eq!(d1.get_keyword().unwrap().to_string_lossy(), "xml");
    assert_eq!(d1.get_version().unwrap().to_string_lossy(), VERSION_1);
    assert_eq!(d1.get_encoding().unwrap().to_string_lossy(), ENCODING_UTF);
    assert!(d1.get_standalone().is_none());
    assert!(d1.get_template_name().is_none());
    assert_eq!(d1.get_line(), -1);
    assert_eq!(d1.get_col(), -1);

    let d1 = XMLDeclaration::with_components(Some(js("xml")), Some(js(VERSION_1)), None, None);
    assert_eq!(
        d1.get_xml_declaration().unwrap().to_string_lossy(),
        XML_DECLAR_1
    );
    assert!(d1.get_encoding().is_none());
    assert!(d1.get_standalone().is_none());

    let d1 = XMLDeclaration::with_components(
        Some(js("xml")),
        Some(js(VERSION_11)),
        Some(js(ENCODING_ISO)),
        Some(js(STANDALONE_YES)),
    );
    assert_eq!(
        d1.get_xml_declaration().unwrap().to_string_lossy(),
        XML_DECLAR_11_ISO_YES
    );
    assert_eq!(d1.get_version().unwrap().to_string_lossy(), VERSION_11);
    assert_eq!(d1.get_encoding().unwrap().to_string_lossy(), ENCODING_ISO);
    assert_eq!(
        d1.get_standalone().unwrap().to_string_lossy(),
        STANDALONE_YES
    );

    // d2: 默认 keyword + standalone=no
    let d2 = XMLDeclaration::with_components(
        Some(js("xml")),
        Some(js(VERSION_1)),
        Some(js(ENCODING_ISO)),
        Some(js(STANDALONE_NO)),
    );
    assert_eq!(
        d2.get_xml_declaration().unwrap().to_string_lossy(),
        XML_DECLAR_1_ISO_NO
    );
    assert_eq!(d2.get_version().unwrap().to_string_lossy(), VERSION_1);
    assert_eq!(d2.get_encoding().unwrap().to_string_lossy(), ENCODING_ISO);
    assert_eq!(
        d2.get_standalone().unwrap().to_string_lossy(),
        STANDALONE_NO
    );

    // d3: 默认 keyword，无 encoding/standalone
    let d3 = XMLDeclaration::with_components(Some(js("xml")), Some(js(VERSION_1)), None, None);
    assert_eq!(
        d3.get_xml_declaration().unwrap().to_string_lossy(),
        XML_DECLAR_1
    );
    assert!(d3.get_encoding().is_none());
    assert!(d3.get_standalone().is_none());

    // Java: XMLDeclaration(String encoding) 便捷构造器
    let d4 = XMLDeclaration::new(Some(js(ENCODING_UTF)));
    assert_eq!(
        d4.get_xml_declaration().unwrap().to_string_lossy(),
        XML_DECLAR_1_UTF
    );
    assert_eq!(d4.get_keyword().unwrap().to_string_lossy(), "xml");
    assert_eq!(d4.get_version().unwrap().to_string_lossy(), VERSION_1);
}

// ===========================================================================
// 6. ProcessingInstructionTest
// ===========================================================================

#[test]
fn processing_instruction_full_constructor_with_location() {
    // 对应 Java ProcessingInstructionTest#test 前两段（5 参构造器）
    let d1 = ProcessingInstruction::with_location(
        Some(js("<?something someother and other and other?>")),
        Some(js("something")),
        Some(js("someother and other and other")),
        Some(js("template")),
        11,
        4,
    );
    assert_eq!(
        d1.get_processing_instruction().unwrap().to_string_lossy(),
        "<?something someother and other and other?>"
    );
    assert_eq!(d1.get_target().unwrap().to_string_lossy(), "something");
    assert_eq!(
        d1.get_content().unwrap().to_string_lossy(),
        "someother and other and other"
    );
    assert_eq!(
        d1.get_template_name().unwrap().to_string_lossy(),
        "template"
    );
    assert_eq!(d1.get_line(), 11);
    assert_eq!(d1.get_col(), 4);

    let d1 = ProcessingInstruction::with_location(
        Some(js("<?something someother and other and other?>")),
        Some(js("something")),
        Some(js("someother and other and other")),
        Some(js("template")),
        10,
        3,
    );
    assert_eq!(d1.get_line(), 10);
    assert_eq!(d1.get_col(), 3);
}

#[test]
fn processing_instruction_computed_forms() {
    // 对应 Java ProcessingInstructionTest#test 第三至六段
    let d1 = ProcessingInstruction::new(
        Some(js("anything-else")),
        Some(js("someother and other and other")),
    );
    assert_eq!(
        d1.get_processing_instruction().unwrap().to_string_lossy(),
        "<?anything-else someother and other and other?>"
    );
    assert_eq!(d1.get_target().unwrap().to_string_lossy(), "anything-else");
    assert_eq!(
        d1.get_content().unwrap().to_string_lossy(),
        "someother and other and other"
    );
    assert!(d1.get_template_name().is_none());
    assert_eq!(d1.get_line(), -1);
    assert_eq!(d1.get_col(), -1);

    let d1 = ProcessingInstruction::new(Some(js("anything-else")), Some(js("nothing here")));
    assert_eq!(
        d1.get_processing_instruction().unwrap().to_string_lossy(),
        "<?anything-else nothing here?>"
    );
    assert_eq!(d1.get_target().unwrap().to_string_lossy(), "anything-else");
    assert_eq!(d1.get_content().unwrap().to_string_lossy(), "nothing here");

    let d1 = ProcessingInstruction::new(Some(js("anything-else")), None);
    assert_eq!(
        d1.get_processing_instruction().unwrap().to_string_lossy(),
        "<?anything-else?>"
    );
    assert_eq!(d1.get_target().unwrap().to_string_lossy(), "anything-else");
    assert!(d1.get_content().is_none());

    let d2 = ProcessingInstruction::new(
        Some(js("something")),
        Some(js("someother and other and other")),
    );
    assert_eq!(
        d2.get_processing_instruction().unwrap().to_string_lossy(),
        "<?something someother and other and other?>"
    );
    assert_eq!(d2.get_target().unwrap().to_string_lossy(), "something");

    let d3 = ProcessingInstruction::new(Some(js("anything-else")), None);
    assert_eq!(
        d3.get_processing_instruction().unwrap().to_string_lossy(),
        "<?anything-else?>"
    );
    assert!(d3.get_content().is_none());
}
