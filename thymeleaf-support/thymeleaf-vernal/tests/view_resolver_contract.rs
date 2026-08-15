//! ThymeleafViewResolver 对 Spring 语义的集成契约测试。
//! 覆盖 webmvc-view-integration-notes 的 5 项桥接缺口。
//!
//! StringTemplateResolver 语义下"模板文本即模板名"——含 th:text 表达式的
//! 用例通过置空前缀/后缀、以模板文本作为视图名驱动（真实资源装载由宿主
//! 换用 File/Class 解析器，ViewResolver 只负责映射与渲染编排）。

use std::any::Any;
use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf_vernal::ThymeleafViewResolver;
use vernal_web::{Model, ViewResolver};

/// 直渲解析器：前缀/后缀置空，视图名 = 模板文本。
fn direct_resolver() -> ThymeleafViewResolver {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(thymeleaf::TemplateMode::HTML);
    let mut resolver = ThymeleafViewResolver::new(Arc::new(TemplateEngine::new()), r);
    resolver.set_prefix("");
    resolver.set_suffix("");
    resolver
}

fn model_with(pairs: &[(&str, Arc<dyn Any + Send + Sync>)]) -> Model {
    let mut model = Model::new();
    for (name, value) in pairs {
        model.add_attribute(*name, Some(Arc::clone(value)));
    }
    model
}

/// 缺口 5：前缀/后缀——Spring 默认值把视图名映射为模板资源名。
#[test]
fn view_name_maps_through_spring_default_prefix_and_suffix() {
    let resolver = direct_resolver();
    // 默认值在 new() 时已设置；此处验证映射函数（默认前缀在直渲构造时被覆盖，
    // 映射语义用 template_name_for 直接断言）。
    assert_eq!(resolver.template_name_for("a.html"), "a.html");
    let mut spring_default = {
        let r = StringTemplateResolver::new();
        ThymeleafViewResolver::new(Arc::new(TemplateEngine::new()), r)
    };
    assert_eq!(
        spring_default.template_name_for("home"),
        "templates/home.html",
        "classpath:/templates/ + .html"
    );
    assert_eq!(
        spring_default.template_name_for("shop/cart"),
        "templates/shop/cart.html"
    );
    spring_default.set_prefix("views/");
    spring_default.set_suffix(".htm");
    assert_eq!(spring_default.template_name_for("page"), "views/page.htm");
}

/// 缺口 1+2：ViewResolver 解析 + Model 桥（String/i64 变量进模板）。
#[test]
fn resolver_renders_view_name_with_model_variables() {
    let resolver = direct_resolver();
    let view = resolver
        .resolve_view_name("<p th:text=\"${name} + ' #' + ${count}\">x</p>", None)
        .expect("resolve");
    let model = model_with(&[
        (
            "name",
            Arc::new("vernal".to_owned()) as Arc<dyn Any + Send + Sync>,
        ),
        ("count", Arc::new(42_i64) as Arc<dyn Any + Send + Sync>),
    ]);
    let rendered = view.render(&model, None).expect("render");
    assert_eq!(rendered.status(), 200);
    assert_eq!(rendered.headers()["content-type"], "text/html");
    let text = String::from_utf8_lossy(rendered.body());
    assert!(text.contains("vernal"), "string 变量: {text}");
    assert!(text.contains("42"), "i64 变量: {text}");
}

/// 缺口 3：Locale 协商——locale 注入渲染上下文。
#[test]
fn locale_is_injected_into_render_context() {
    let resolver = direct_resolver();
    let view = resolver
        .resolve_view_name("<p th:text=\"${#locale.getCountry()}\">x</p>", None)
        .expect("resolve");
    let rendered = view.render(&Model::new(), Some("zh-CN")).expect("render");
    assert_eq!(rendered.body(), "<p>CN</p>");
}

/// 缺口 4：缓存开关——cacheable=false 每次渲染走缓存失效路径，结果幂等。
#[test]
fn cacheable_false_invalidates_template_cache_per_render() {
    let mut resolver = direct_resolver();
    resolver.set_cacheable(false);
    let view = resolver
        .resolve_view_name("<p>stable</p>", None)
        .expect("resolve");
    assert_eq!(
        view.render(&Model::new(), None).expect("v1").body(),
        "<p>stable</p>"
    );
    // 失效路径（clear_template_cache_for）不报错且重渲染结果一致
    let again = view.render(&Model::new(), None).expect("re-render");
    assert_eq!(again.body(), "<p>stable</p>");
}

/// 渲染失败的诊断错误（模板语法错误）。
#[test]
fn render_failure_reports_diagnostic_view_error() {
    let resolver = direct_resolver();
    let view = resolver
        .resolve_view_name("<p th:text=\"${unclosed\">x</p>", None)
        .expect("resolve struct");
    let error = view
        .render(&Model::new(), None)
        .err()
        .expect("syntax error");
    assert!(!error.message().is_empty());
}
