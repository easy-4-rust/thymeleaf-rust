//! VALUE_ADD：`ThymeleafRenderer` 覆盖缺口测试（2026-09-02）——风险：web 渲染路径
//! （render_data_stream、default_content_type 各模板模式分支、metadata 错误处理）。
//!
//! 缺失行 124-176+：render_data_stream 数据驱动渲染、default_content_type 模板模式
//! 分支（XML/TEXT/JAVASCRIPT/CSS/RAW）。Java 侧 Spring View/Reactive 测试迁移已覆盖
//! render_full/render_stream；以下补 VALUE_ADD 分支。

use std::sync::Arc;

use thymeleaf::context::{Context, IContext};
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::web::{RenderedTemplateBody, ThymeleafRenderer};
use thymeleaf::{ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode, TemplateSpec};

fn engine() -> Arc<TemplateEngine> {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let e = Arc::new(TemplateEngine::new());
    e.set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver");
    e
}

fn renderer() -> ThymeleafRenderer {
    ThymeleafRenderer::new(engine() as Arc<dyn ITemplateEngine>)
}

// ===========================================================================
// default_content_type: XML template mode
// ===========================================================================

#[test]
fn render_xml_template_has_xml_content_type() {
    let template = "<root><item>data</item></root>";
    let spec = TemplateSpec::with_template_mode(Some(template), Some(TemplateMode::XML))
        .expect("template spec");
    let rendered = renderer()
        .render_full(&spec, &Context::new())
        .expect("render");
    let (_, headers, _) = rendered.into_parts();
    let ct = headers
        .get(http::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("application/xml"),
        "XML mode must produce application/xml: {ct}"
    );
}

// ===========================================================================
// default_content_type: TEXT template mode
// ===========================================================================

#[test]
fn render_text_template_has_plain_content_type() {
    let template = "plain text content";
    let spec = TemplateSpec::with_template_mode(Some(template), Some(TemplateMode::TEXT))
        .expect("template spec");
    let rendered = renderer()
        .render_full(&spec, &Context::new())
        .expect("render");
    let (_, headers, _) = rendered.into_parts();
    let ct = headers
        .get(http::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("text/plain"),
        "TEXT mode must produce text/plain: {ct}"
    );
}

// ===========================================================================
// default_content_type: JAVASCRIPT template mode
// ===========================================================================

#[test]
fn render_javascript_template_has_js_content_type() {
    let template = "var x = 1;";
    let spec = TemplateSpec::with_template_mode(Some(template), Some(TemplateMode::JAVASCRIPT))
        .expect("template spec");
    let rendered = renderer()
        .render_full(&spec, &Context::new())
        .expect("render");
    let (_, headers, _) = rendered.into_parts();
    let ct = headers
        .get(http::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("application/javascript"), "JS mode: {ct}");
}

// ===========================================================================
// default_content_type: CSS template mode
// ===========================================================================

#[test]
fn render_css_template_has_css_content_type() {
    let template = "body { color: red; }";
    let spec = TemplateSpec::with_template_mode(Some(template), Some(TemplateMode::CSS))
        .expect("template spec");
    let rendered = renderer()
        .render_full(&spec, &Context::new())
        .expect("render");
    let (_, headers, _) = rendered.into_parts();
    let ct = headers
        .get(http::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("text/css"), "CSS mode: {ct}");
}

// ===========================================================================
// default_content_type: RAW template mode
// ===========================================================================

#[test]
fn render_raw_template_has_octet_stream_content_type() {
    let template = "binary data";
    let spec = TemplateSpec::with_template_mode(Some(template), Some(TemplateMode::RAW))
        .expect("template spec");
    let rendered = renderer()
        .render_full(&spec, &Context::new())
        .expect("render");
    let (_, headers, _) = rendered.into_parts();
    let ct = headers
        .get(http::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("application/octet-stream"), "RAW mode: {ct}");
}

// ===========================================================================
// with_chunk_size: clamps to minimum 1
// ===========================================================================

#[test]
fn with_chunk_size_clamps_below_one() {
    let r = renderer().with_chunk_size(0);
    // chunk_size should be clamped to 1 (field is private, but we can verify render still works)
    let template = "<p>test</p>";
    let spec = TemplateSpec::with_template_mode(Some(template), Some(TemplateMode::HTML))
        .expect("template spec");
    let context: Arc<dyn IContext> = Arc::new(Context::new());
    let rendered = r.render_stream(spec, context).expect("render");
    let (_, _, body) = rendered.into_parts();
    assert!(matches!(body, RenderedTemplateBody::Stream(_)));
}

// ===========================================================================
// render_stream: produces stream body
// ===========================================================================

#[test]
fn render_stream_produces_stream_body() {
    let template = "<p>streamed</p>";
    let spec = TemplateSpec::with_template_mode(Some(template), Some(TemplateMode::HTML))
        .expect("template spec");
    let context: Arc<dyn IContext> = Arc::new(Context::new());
    let rendered = renderer().render_stream(spec, context).expect("render");
    let (_, headers, body) = rendered.into_parts();
    // Stream body has no Content-Length
    assert!(headers.get(http::header::CONTENT_LENGTH).is_none());
    assert!(matches!(body, RenderedTemplateBody::Stream(_)));
}

// ===========================================================================
// render_full: produces full body with Content-Length
// ===========================================================================

#[test]
fn render_full_produces_content_length() {
    let template = "<p>full</p>";
    let spec = TemplateSpec::with_template_mode(Some(template), Some(TemplateMode::HTML))
        .expect("template spec");
    let rendered = renderer()
        .render_full(&spec, &Context::new())
        .expect("render");
    let (_, headers, body) = rendered.into_parts();
    assert!(headers.get(http::header::CONTENT_LENGTH).is_some());
    if let RenderedTemplateBody::Full(bytes) = body {
        assert_eq!(bytes.as_ref(), template.as_bytes());
    } else {
        panic!("expected full body");
    }
}

// ===========================================================================
// render_full: invalid charset produces error
// ===========================================================================

#[test]
fn render_full_invalid_charset_produces_error() {
    let spec = TemplateSpec::with_output_content_type(
        Some("<p>test</p>"),
        Some("text/html;charset=INVALID-CHARSET"),
    )
    .expect("template spec");
    match renderer().render_full(&spec, &Context::new()) {
        Ok(_) => panic!("invalid charset must produce error"),
        Err(error) => assert!(
            error.to_string().to_ascii_lowercase().contains("charset"),
            "error must mention charset: {error}"
        ),
    }
}
