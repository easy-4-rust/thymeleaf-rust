//! DTD 验证集成测试（`dtd-validation` feature）。
//!
//! 覆盖三部分：
//! 1. `DtdValidator` 单元：SYSTEM 标识符识别、未知标识符降级；
//! 2. `TemplateEngine` XML 模式三策略（Disabled / Warn / Strict）行为；
//! 3. DOCTYPE 缺失 / 未知 SYSTEM 标识符的边界语义。
#![cfg(feature = "dtd-validation")]

use std::sync::Arc;

use thymeleaf::context::Context;
use thymeleaf::dtd::ValidationPolicy;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::{TemplateEngine, TemplateMode};

const XHTML1_STRICT: &str = "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd";
const XHTML1_TRANSITIONAL: &str = "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd";
const XHTML1_FRAMESET: &str = "http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd";

/// 以给定 SYSTEM 标识符组装 `<!DOCTYPE html SYSTEM ...>` 前缀。
fn doctype(system_id: &str) -> String {
    format!("<!DOCTYPE html SYSTEM \"{system_id}\">\n")
}

/// XHTML 1.0 Strict 下合法的最小文档。
fn valid_strict() -> String {
    format!(
        "{}<html><head><title>t</title></head><body><p>x</p></body></html>",
        doctype(XHTML1_STRICT)
    )
}

/// 含未声明元素 `<bogus/>`（违反 Element Valid）。
fn invalid_undeclared_element() -> String {
    format!(
        "{}<html><head><title>t</title></head><body><p>x</p><bogus/></body></html>",
        doctype(XHTML1_STRICT)
    )
}

/// `html` 的内容模型为 `(head, body)`，缺 `head` 即违反。
fn invalid_missing_head() -> String {
    format!("{}<html><body/></html>", doctype(XHTML1_STRICT))
}

/// `p` 未声明 `bogusattr` 属性。
fn invalid_undeclared_attribute() -> String {
    format!(
        "{}<html><head><title>t</title></head><body><p bogusattr=\"1\">x</p></body></html>",
        doctype(XHTML1_STRICT)
    )
}

fn xml_engine(policy: ValidationPolicy) -> TemplateEngine {
    let engine = TemplateEngine::new();
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::XML);
    engine
        .set_template_resolver(Arc::new(resolver))
        .expect("engine not initialized");
    engine
        .set_dtd_validation_policy(policy)
        .expect("engine not initialized");
    engine
}

fn process(engine: &TemplateEngine, template: &str) -> Result<String, String> {
    let context = Context::new();
    engine
        .process_template(template, &context)
        .map(|output| output.to_string_lossy())
        .map_err(|error| {
            // 异常 Display 只含顶层消息，沿原因链拼接底层细节（含 DTD 违反描述）。
            let mut message = format!("{error}");
            let mut source = std::error::Error::source(&*error);
            while let Some(current) = source {
                message.push_str(" | ");
                message.push_str(&current.to_string());
                source = current.source();
            }
            message
        })
}

#[test]
fn disabled_policy_is_default_and_never_rejects() {
    assert_eq!(ValidationPolicy::default(), ValidationPolicy::Disabled);
    // 默认引擎（未显式设置策略）对无效文档放行——零验证开销语义。
    let engine = xml_engine(ValidationPolicy::Disabled);
    for template in [
        invalid_undeclared_element(),
        invalid_missing_head(),
        invalid_undeclared_attribute(),
    ] {
        let result = process(&engine, &template);
        assert!(result.is_ok(), "Disabled must not reject: {result:?}");
    }
}

#[test]
fn strict_accepts_valid_document() {
    let engine = xml_engine(ValidationPolicy::Strict);
    let output = process(&engine, &valid_strict()).expect("valid document must pass");
    assert!(output.contains("<p>x</p>"), "unexpected output: {output}");
}

#[test]
fn strict_accepts_mixed_content_nodes() {
    let engine = xml_engine(ValidationPolicy::Strict);
    // 注释/PI 按 markup() 处理（非 EMPTY 内容即合法）；实体引用与 CDATA
    // 按 reference_data 处理（p 为混合内容模型，均放行）。
    let template = format!(
        concat!(
            "{}<html><head><title>t<!-- note --><?proc instr?></title></head>",
            "<body><p>a&amp;b<![CDATA[<c>]]></p></body></html>"
        ),
        doctype(XHTML1_STRICT)
    );
    let result = process(&engine, &template);
    assert!(result.is_ok(), "mixed content must pass Strict: {result:?}");
}

#[test]
fn strict_rejects_text_in_empty_element() {
    let engine = xml_engine(ValidationPolicy::Strict);
    // XHTML 1.0 Strict 中 <base> 声明为 EMPTY：任何字符数据即违反。
    let template = format!(
        concat!(
            "{}<html><head><title>t</title><base href=\"/\">x</base></head>",
            "<body><p>x</p></body></html>"
        ),
        doctype(XHTML1_STRICT)
    );
    let error = process(&engine, &template).expect_err("text in EMPTY element must fail");
    assert!(
        error.contains("DTD validation failed"),
        "error must identify DTD validation: {error}"
    );
}

#[test]
fn strict_rejects_undeclared_element() {
    let engine = xml_engine(ValidationPolicy::Strict);
    let error = process(&engine, &invalid_undeclared_element())
        .expect_err("undeclared element must fail in Strict");
    assert!(
        error.contains("DTD validation failed"),
        "error must identify DTD validation: {error}"
    );
    assert!(
        error.to_lowercase().contains("bogus"),
        "error must name the offending element: {error}"
    );
}

#[test]
fn strict_rejects_missing_head_content_model() {
    let engine = xml_engine(ValidationPolicy::Strict);
    let error =
        process(&engine, &invalid_missing_head()).expect_err("missing head must fail in Strict");
    assert!(
        error.contains("DTD validation failed"),
        "error must identify DTD validation: {error}"
    );
}

#[test]
fn strict_rejects_undeclared_attribute() {
    let engine = xml_engine(ValidationPolicy::Strict);
    let error = process(&engine, &invalid_undeclared_attribute())
        .expect_err("undeclared attribute must fail in Strict");
    assert!(
        error.contains("DTD validation failed"),
        "error must identify DTD validation: {error}"
    );
}

#[test]
fn warn_validates_but_never_rejects() {
    let engine = xml_engine(ValidationPolicy::Warn);
    for template in [
        invalid_undeclared_element(),
        invalid_missing_head(),
        invalid_undeclared_attribute(),
    ] {
        let result = process(&engine, &template);
        assert!(result.is_ok(), "Warn must not reject: {result:?}");
    }
}

#[test]
fn strict_without_doctype_skips_validation() {
    let engine = xml_engine(ValidationPolicy::Strict);
    // 无 DOCTYPE 即无验证义务：未声明元素不构成违反。
    let result = process(&engine, "<html><body><bogus/></body></html>");
    assert!(result.is_ok(), "no DOCTYPE means no validation: {result:?}");
}

#[test]
fn strict_with_unknown_system_id_fails_closed() {
    let engine = xml_engine(ValidationPolicy::Strict);
    let template = format!(
        "{}<html><body><bogus/></body></html>",
        doctype("http://example.com/unknown.dtd")
    );
    let error = process(&engine, &template).expect_err("unresolvable DTD must fail in Strict");
    assert!(
        error.contains("Cannot resolve DTD"),
        "error must identify unresolvable DTD: {error}"
    );
}

#[test]
fn warn_with_unknown_system_id_degrades_to_no_validation() {
    let engine = xml_engine(ValidationPolicy::Warn);
    let template = format!(
        "{}<html><body><bogus/></body></html>",
        doctype("http://example.com/unknown.dtd")
    );
    let result = process(&engine, &template);
    assert!(
        result.is_ok(),
        "Warn must degrade to no validation: {result:?}"
    );
}

#[test]
fn validator_builds_for_w3c_system_ids_and_rejects_unknown() {
    use thymeleaf::dtd::DtdValidator;
    for system_id in [XHTML1_STRICT, XHTML1_TRANSITIONAL, XHTML1_FRAMESET] {
        let declaration = format!("html SYSTEM \"{system_id}\"");
        assert!(
            DtdValidator::new(&declaration).is_some(),
            "known system id must resolve: {system_id}"
        );
    }
    let unknown = "html SYSTEM \"http://example.com/x.dtd\"";
    assert!(DtdValidator::new(unknown).is_none());
    // 未声明外部标识符的声明主体按内部子集 DTD 解析，仍可构建。
    assert!(DtdValidator::new("html").is_some());
}

#[test]
fn strict_validates_transitional_doctype() {
    let engine = xml_engine(ValidationPolicy::Strict);
    // Transitional 允许 Strict 禁止的元素（如 <center>），对 Transitional 应放行。
    let template = format!(
        "{}<html><head><title>t</title></head><body><center>x</center></body></html>",
        doctype(XHTML1_TRANSITIONAL)
    );
    let result = process(&engine, &template);
    assert!(result.is_ok(), "transitional allows center: {result:?}");
    // 同一文档对 Strict 必须拒绝——DOCTYPE 决定验证基准。
    let engine = xml_engine(ValidationPolicy::Strict);
    let strict_template = template.replace(XHTML1_TRANSITIONAL, XHTML1_STRICT);
    let error = process(&engine, &strict_template).expect_err("strict must reject center");
    assert!(error.contains("DTD validation failed"), "{error}");
}
