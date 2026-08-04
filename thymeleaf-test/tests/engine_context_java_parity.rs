//! EngineContext 与 WebEngineContext 的固定 Java 上游差分测试。

#[allow(dead_code, unused_imports)]
mod support;

use std::collections::BTreeMap;
use std::sync::Arc;

use indexmap::IndexMap;
use support::CorpusWebExchange;
use thymeleaf::cache::AlwaysValidCacheEntryValidity;
use thymeleaf::context::{
    EngineContext, IContext, IEngineContext, IExpressionContext, ITemplateContext, WebEngineContext,
};
use thymeleaf::engine::TemplateData;
use thymeleaf::expression::TemplateValue;
use thymeleaf::inline::StandardTextInliner;
use thymeleaf::templateresource::StringTemplateResource;
use thymeleaf::util::{Locale, Utf16String};
use thymeleaf::{ITemplateEngine, TemplateEngine, TemplateMode};

fn golden() -> BTreeMap<String, String> {
    include_str!("../../thymeleaf/tests/fixtures/engine_context_golden.txt")
        .lines()
        .map(|line| {
            let (key, value) = line.split_once('=').expect("golden key/value");
            (key.to_owned(), value.to_owned())
        })
        .collect()
}

fn utf16_string(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn value(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::string(utf16_string(value)))
}

fn locale(language: &str, country: &str) -> Locale {
    Locale::new(utf16_string(language), utf16_string(country))
}

fn template_data(name: &str) -> TemplateData {
    TemplateData::new(
        Some(utf16_string(name)),
        None,
        Some(Arc::new(
            StringTemplateResource::new(Some(name)).expect("resource"),
        )),
        Some(TemplateMode::HTML),
        Some(Arc::new(AlwaysValidCacheEntryValidity::new())),
    )
}

fn variable_text(context: &dyn IContext, name: &str) -> String {
    context
        .get_variable(Some(&utf16_string(name)))
        .and_then(|value| value.to_utf16_string())
        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy())
}

fn stack(context: &dyn ITemplateContext) -> String {
    context
        .get_template_stack()
        .into_iter()
        .map(|template| template.get_template().expect("template").to_string_lossy())
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn engine_and_web_contexts_match_java_golden_for_scope_and_selection() {
    let fixture = golden();
    assert_eq!(
        fixture["baseline"],
        "10f9dd2eb8cbd98515ce14b149d115e0287d0add"
    );
    assert!(fixture["shape.abstract"].contains("getExpressionObjects()"));
    assert!(fixture["shape.engine"].contains("getStringRepresentationByLevel()"));
    assert!(fixture["shape.web"].contains("getExchange()"));
    assert!(fixture["shape.template.interface"].contains("getIdentifierSequences()"));
    assert!(
        fixture["shape.engine.interface"]
            .contains("setVariable(java.lang.String,java.lang.Object)")
    );

    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("configuration");
    let mut variables = IndexMap::new();
    variables.insert(Some(utf16_string("root")), Some(value("one")));
    variables.insert(
        Some(utf16_string("nullable")),
        Some(Arc::new(TemplateValue::Null)),
    );
    let context = EngineContext::new(
        Arc::clone(&configuration),
        template_data("root-template"),
        None,
        locale("en-US", "US"),
        Some(&variables),
    );
    assert_eq!(
        variable_text(context.as_ref(), "root"),
        fixture["plain.root.value"]
    );
    assert_eq!(
        variable_text(context.as_ref(), "nullable"),
        fixture["plain.root.nullable"]
    );
    assert_eq!(stack(context.as_ref()), fixture["plain.root.stack"]);
    assert_eq!(
        context.has_selection_target().to_string(),
        fixture["plain.root.selection.present"]
    );

    context.set_selection_target(Some(value("root-target")));
    context.increase_level();
    context.set_variable(Some(utf16_string("root")), Some(value("local")));
    context.set_variable(Some(utf16_string("local")), Some(value("yes")));
    context.set_selection_target(None);
    context.set_template_data(Arc::new(template_data("nested-template")));
    assert_eq!(context.level().to_string(), fixture["plain.nested.level"]);
    assert_eq!(
        variable_text(context.as_ref(), "root"),
        fixture["plain.nested.root"]
    );
    assert_eq!(
        variable_text(context.as_ref(), "local"),
        fixture["plain.nested.local"]
    );
    assert_eq!(
        context
            .is_variable_local(Some(&utf16_string("root")))
            .to_string(),
        fixture["plain.nested.root.local"]
    );
    assert_eq!(
        context.has_selection_target().to_string(),
        fixture["plain.nested.selection.present"]
    );
    // 显式 null 必须遮蔽父级 target，不能把 `root-target` 错误地回退出来。
    assert!(context.get_selection_target().is_none());
    assert_eq!(stack(context.as_ref()), fixture["plain.nested.stack"]);
    assert_eq!(
        context.get_string_representation_by_level(),
        fixture["plain.nested.representation"]
    );
    context.decrease_level();
    assert_eq!(context.level().to_string(), fixture["plain.restored.level"]);
    assert_eq!(
        variable_text(context.as_ref(), "root"),
        fixture["plain.restored.root"]
    );
    assert_eq!(
        variable_text(context.as_ref(), "local"),
        fixture["plain.restored.local"]
    );
    assert_eq!(
        context
            .get_selection_target()
            .and_then(|value| value.to_utf16_string())
            .expect("selection")
            .to_string_lossy(),
        fixture["plain.restored.selection"]
    );
    assert_eq!(stack(context.as_ref()), fixture["plain.restored.stack"]);

    let exchange: Arc<dyn thymeleaf::web::IWebExchange> = Arc::new(CorpusWebExchange::new());
    let web = WebEngineContext::new(
        configuration,
        template_data("web-root"),
        None,
        Arc::clone(&exchange),
        locale("en-CA", "CA"),
        None,
    );
    web.set_variable(Some(utf16_string("value")), Some(value("root")));
    web.set_selection_target(Some(value("root-target")));
    web.increase_level();
    web.set_variable(Some(utf16_string("value")), Some(value("local")));
    web.set_variable(Some(utf16_string("local")), Some(value("yes")));
    web.set_selection_target(None);
    assert_eq!(
        variable_text(web.as_ref(), "value"),
        fixture["web.nested.value"]
    );
    assert_eq!(
        variable_text(web.as_ref(), "local"),
        fixture["web.nested.local"]
    );
    assert_eq!(
        web.is_variable_local(Some(&utf16_string("value")))
            .to_string(),
        fixture["web.nested.value.local"]
    );
    assert_eq!(
        web.has_selection_target().to_string(),
        fixture["web.nested.selection.present"]
    );
    assert!(web.get_selection_target().is_none());
    assert_eq!(
        web.get_string_representation_by_level(),
        fixture["web.nested.representation"]
    );
    web.decrease_level();
    assert_eq!(
        variable_text(web.as_ref(), "value"),
        fixture["web.restored.value"]
    );
    assert_eq!(
        variable_text(web.as_ref(), "local"),
        fixture["web.restored.local"]
    );
    assert_eq!(
        web.get_selection_target()
            .and_then(|value| value.to_utf16_string())
            .expect("selection")
            .to_string_lossy(),
        fixture["web.restored.selection"]
    );
}

#[test]
fn abstract_engine_context_defers_expression_object_construction() {
    let fixture = golden();
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("configuration");
    let context = EngineContext::new(
        configuration,
        template_data("lazy-expression"),
        None,
        locale("en-US", "US"),
        None,
    );
    let first =
        context.get_expression_objects() as *const dyn thymeleaf::expression::IExpressionObjects;
    let second =
        context.get_expression_objects() as *const dyn thymeleaf::expression::IExpressionObjects;
    assert_eq!(first, second);
    assert_eq!(fixture["expression.factory.before"], "0");
    assert_eq!(fixture["expression.factory.after.first"], "1");
    assert_eq!(fixture["expression.factory.same"], "true");
    assert_eq!(fixture["expression.factory.after.second"], "1");
}

// ===========================================================================
// EngineContextTest test03/test05 关键序列（Java 21 逐字）：
//   (*removed*) 占位标记 与 [StandardTextInliner] 表示
// ===========================================================================

#[test]
fn engine_context_removed_marker_and_inliner_representation_match_java() {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("configuration");
    let mut variables = IndexMap::new();
    variables.insert(Some(utf16_string("one")), Some(value("two values")));
    let context = EngineContext::new(
        Arc::clone(&configuration),
        template_data("test01"),
        None,
        locale("en-US", "US"),
        Some(&variables),
    );

    // 初始：{0:{one=two values}(test01)}[0]
    assert_eq!(
        context.get_string_representation_by_level(),
        "{0:{one=two values}(test01)}[0]"
    );
    assert_eq!(context.to_string(), "{one=two values}(test01)");

    context.increase_level();
    context.set_variable(Some(utf16_string("one")), Some(value("hello")));
    assert_eq!(
        context.get_string_representation_by_level(),
        "{1:{one=hello},0:{one=two values}(test01)}[1]"
    );

    // removeVariable -> EngineContext 用 (*removed*) 占位（WebEngineContext 为 null）
    context.remove_variable(Some(&utf16_string("one")));
    assert_eq!(
        context.get_string_representation_by_level(),
        "{1:{one=(*removed*)},0:{one=two values}(test01)}[1]"
    );
    assert_eq!(context.to_string(), "{}(test01)");

    context.set_variable(Some(utf16_string("one")), Some(value("hello")));
    context.remove_variable(Some(&utf16_string("two")));
    assert_eq!(
        context.get_string_representation_by_level(),
        "{1:{one=hello},0:{one=two values}(test01)}[1]",
        "删除不存在的变量无副作用"
    );

    // setInliner(StandardTextInliner) -> 表示串与 toString 带 [StandardTextInliner]
    context.set_variable(Some(utf16_string("two")), Some(value("twello")));
    context.set_inliner(Some(Arc::new(StandardTextInliner::new(
        configuration.as_ref(),
    ))));
    assert_eq!(
        context.get_string_representation_by_level(),
        "{1:{one=hello, two=twello}[StandardTextInliner],0:{one=two values}(test01)}[1]"
    );
    assert_eq!(
        context.to_string(),
        "{one=hello, two=twello}[StandardTextInliner](test01)"
    );

    context.remove_variable(Some(&utf16_string("two")));
    assert_eq!(
        context.get_string_representation_by_level(),
        "{1:{one=hello}[StandardTextInliner],0:{one=two values}(test01)}[1]"
    );
    context.remove_variable(Some(&utf16_string("one")));
    assert_eq!(
        context.get_string_representation_by_level(),
        "{1:{one=(*removed*)}[StandardTextInliner],0:{one=two values}(test01)}[1]"
    );
    assert_eq!(context.to_string(), "{}[StandardTextInliner](test01)");

    // 降层恢复
    context.decrease_level();
    assert_eq!(
        context.get_string_representation_by_level(),
        "{0:{one=two values}(test01)}[0]"
    );
    assert_eq!(context.to_string(), "{one=two values}(test01)");
}

// ===========================================================================
// EngineContextTest test01：多层嵌套遮蔽完整断言序列（Java 21 逐字）
// ===========================================================================

/// 断言 containsVariable + getVariable 与 Java 一致。
fn assert_variable(context: &dyn IContext, name: &str, present: bool, expected: &str) {
    let key = utf16_string(name);
    assert_eq!(
        context.contains_variable(Some(&key)),
        present,
        "containsVariable({name})"
    );
    assert_eq!(
        &variable_text(context, name),
        expected,
        "getVariable({name})"
    );
}

fn selection_text(context: &dyn ITemplateContext) -> String {
    context
        .get_selection_target()
        .and_then(|value| value.to_utf16_string())
        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy())
}

#[test]
fn engine_context_test01_multi_level_shadowing_matches_java() {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("configuration");
    let context = EngineContext::new(
        configuration,
        template_data("test01"),
        None,
        locale("en-US", "US"),
        None,
    );

    context.set_variable(Some(utf16_string("one")), Some(value("a value")));
    assert_variable(context.as_ref(), "one", true, "a value");

    context.set_variable(Some(utf16_string("one")), Some(value("two values")));
    assert_variable(context.as_ref(), "one", true, "two values");

    context.increase_level();
    context.set_variable(Some(utf16_string("one")), Some(value("hello")));
    assert_variable(context.as_ref(), "one", true, "hello");

    context.set_variable(Some(utf16_string("two")), Some(value("twello")));
    assert_variable(context.as_ref(), "one", true, "hello");
    assert_variable(context.as_ref(), "two", true, "twello");

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "two values");
    assert_variable(context.as_ref(), "two", false, "null");

    context.increase_level();
    context.set_variable(Some(utf16_string("two")), Some(value("twellor")));
    assert_variable(context.as_ref(), "one", true, "two values");
    assert_variable(context.as_ref(), "two", true, "twellor");

    context.increase_level();
    context.set_variable(Some(utf16_string("three")), Some(value("twelloree")));
    assert_variable(context.as_ref(), "one", true, "two values");
    assert_variable(context.as_ref(), "two", true, "twellor");
    assert_variable(context.as_ref(), "three", true, "twelloree");

    context.set_variable(Some(utf16_string("one")), Some(value("atwe")));
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", true, "twellor");
    assert_variable(context.as_ref(), "three", true, "twelloree");

    context.increase_level();
    context.increase_level();
    context.increase_level();
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", true, "twellor");
    assert_variable(context.as_ref(), "three", true, "twelloree");

    context.set_variable(Some(utf16_string("four")), Some(value("lotwss")));
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", true, "twellor");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert_variable(context.as_ref(), "four", true, "lotwss");

    context.set_variable(Some(utf16_string("two")), Some(value("itwiii")));
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", true, "itwiii");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert_variable(context.as_ref(), "four", true, "lotwss");
    assert_variable(context.as_ref(), "five", false, "null");

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", true, "twellor");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert_variable(context.as_ref(), "four", false, "null");

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", true, "twellor");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert_variable(context.as_ref(), "four", false, "null");

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", true, "twellor");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert_variable(context.as_ref(), "four", false, "null");

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "two values");
    assert_variable(context.as_ref(), "two", true, "twellor");
    assert_variable(context.as_ref(), "three", false, "null");
    assert_variable(context.as_ref(), "four", false, "null");

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "two values");
    assert_variable(context.as_ref(), "two", false, "null");
    assert_variable(context.as_ref(), "three", false, "null");
    assert_variable(context.as_ref(), "four", false, "null");
}

// ===========================================================================
// EngineContextTest test02：起始变量（starting Map）构造 + 遮蔽/恢复
// ===========================================================================

#[test]
fn engine_context_test02_starting_variables_shadowing_matches_java() {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("configuration");
    let mut variables = IndexMap::new();
    variables.insert(Some(utf16_string("one")), Some(value("ha")));
    variables.insert(Some(utf16_string("ten")), Some(value("tieen")));
    let context = EngineContext::new(
        Arc::clone(&configuration),
        template_data("test01"),
        None,
        locale("en-US", "US"),
        Some(&variables),
    );

    assert_variable(context.as_ref(), "one", true, "ha");
    assert_variable(context.as_ref(), "ten", true, "tieen");

    context.set_variable(Some(utf16_string("one")), Some(value("a value")));
    assert_variable(context.as_ref(), "one", true, "a value");
    assert_variable(context.as_ref(), "ten", true, "tieen");

    context.increase_level();
    context.set_variable(Some(utf16_string("one")), Some(value("hello")));
    assert_variable(context.as_ref(), "one", true, "hello");
    assert_variable(context.as_ref(), "ten", true, "tieen");

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "a value");
    assert_variable(context.as_ref(), "ten", true, "tieen");
}

// ===========================================================================
// EngineContextTest test06：单层七变量
// ===========================================================================

#[test]
fn engine_context_test06_seven_variables_match_java() {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("configuration");
    let context = EngineContext::new(
        configuration,
        template_data("test01"),
        None,
        locale("en-US", "US"),
        None,
    );

    context.set_variable(Some(utf16_string("one")), Some(value("a value")));
    assert_variable(context.as_ref(), "one", true, "a value");

    context.increase_level();
    context.set_variable(Some(utf16_string("one")), Some(value("hello")));
    assert_variable(context.as_ref(), "one", true, "hello");

    context.set_variable(Some(utf16_string("two")), Some(value("twello")));
    assert_variable(context.as_ref(), "one", true, "hello");
    assert_variable(context.as_ref(), "two", true, "twello");

    context.set_variable(Some(utf16_string("three")), Some(value("trwello")));
    assert_variable(context.as_ref(), "one", true, "hello");
    assert_variable(context.as_ref(), "two", true, "twello");
    assert_variable(context.as_ref(), "three", true, "trwello");

    context.set_variable(Some(utf16_string("four")), Some(value("fwello")));
    assert_variable(context.as_ref(), "one", true, "hello");
    assert_variable(context.as_ref(), "two", true, "twello");
    assert_variable(context.as_ref(), "three", true, "trwello");
    assert_variable(context.as_ref(), "four", true, "fwello");

    context.set_variable(Some(utf16_string("five")), Some(value("vwello")));
    assert_variable(context.as_ref(), "one", true, "hello");
    assert_variable(context.as_ref(), "two", true, "twello");
    assert_variable(context.as_ref(), "three", true, "trwello");
    assert_variable(context.as_ref(), "four", true, "fwello");
    assert_variable(context.as_ref(), "five", true, "vwello");

    context.set_variable(Some(utf16_string("six")), Some(value("swello")));
    assert_variable(context.as_ref(), "one", true, "hello");
    assert_variable(context.as_ref(), "two", true, "twello");
    assert_variable(context.as_ref(), "three", true, "trwello");
    assert_variable(context.as_ref(), "four", true, "fwello");
    assert_variable(context.as_ref(), "five", true, "vwello");
    assert_variable(context.as_ref(), "six", true, "swello");

    context.set_variable(Some(utf16_string("seven")), Some(value("svwello")));
    assert_variable(context.as_ref(), "one", true, "hello");
    assert_variable(context.as_ref(), "two", true, "twello");
    assert_variable(context.as_ref(), "three", true, "trwello");
    assert_variable(context.as_ref(), "four", true, "fwello");
    assert_variable(context.as_ref(), "five", true, "vwello");
    assert_variable(context.as_ref(), "six", true, "swello");
    assert_variable(context.as_ref(), "seven", true, "svwello");
}

// ===========================================================================
// EngineContextTest test07：selection target 多级设置/清除 + 精确表示串
// ===========================================================================

#[test]
fn engine_context_test07_selection_targets_match_java() {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("configuration");
    let context = EngineContext::new(
        Arc::clone(&configuration),
        template_data("test01"),
        None,
        locale("en-US", "US"),
        None,
    );

    context.set_variable(Some(utf16_string("one")), Some(value("a value")));
    assert!(!context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "null");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{0:{one=a value}(test01)}[0]"
    );
    assert_eq!(context.to_string(), "{one=a value}(test01)");

    context.increase_level();
    context.set_variable(Some(utf16_string("one")), Some(value("hello")));
    context.remove_variable(Some(&utf16_string("one")));
    context.set_variable(Some(utf16_string("one")), Some(value("hello")));
    context.remove_variable(Some(&utf16_string("two")));
    context.set_variable(Some(utf16_string("two")), Some(value("twello")));
    context.set_variable(Some(utf16_string("two")), Some(value("twellor")));
    assert!(!context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "null");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{1:{one=hello, two=twellor},0:{one=a value}(test01)}[1]"
    );
    assert_eq!(context.to_string(), "{one=hello, two=twellor}(test01)");

    context.increase_level();
    context.set_variable(Some(utf16_string("three")), Some(value("twelloree")));
    context.set_variable(Some(utf16_string("one")), Some(value("atwe")));
    assert!(!context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "null");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{2:{one=atwe, three=twelloree},1:{one=hello, two=twellor},0:{one=a value}(test01)}[2]"
    );
    assert_eq!(
        context.to_string(),
        "{one=atwe, two=twellor, three=twelloree}(test01)"
    );

    context.set_selection_target(Some(value("BIGFORM")));
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", true, "twellor");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "BIGFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{2:{one=atwe, three=twelloree}<BIGFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[2]"
    );
    assert_eq!(
        context.to_string(),
        "{one=atwe, two=twellor, three=twelloree}<BIGFORM>(test01)"
    );

    context.increase_level();
    context.remove_variable(Some(&utf16_string("two")));
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", false, "null");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "BIGFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{3:{two=(*removed*)},2:{one=atwe, three=twelloree}<BIGFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[3]"
    );
    assert_eq!(
        context.to_string(),
        "{one=atwe, three=twelloree}<BIGFORM>(test01)"
    );

    context.increase_level();
    context.remove_variable(Some(&utf16_string("two")));
    context.set_selection_target(Some(value("SMALLFORM")));
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", false, "null");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "SMALLFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{4:<SMALLFORM>,3:{two=(*removed*)},2:{one=atwe, three=twelloree}<BIGFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[4]"
    );
    assert_eq!(
        context.to_string(),
        "{one=atwe, three=twelloree}<SMALLFORM>(test01)"
    );

    context.increase_level();
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", false, "null");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "SMALLFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{4:<SMALLFORM>,3:{two=(*removed*)},2:{one=atwe, three=twelloree}<BIGFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[5]"
    );
    assert_eq!(
        context.to_string(),
        "{one=atwe, three=twelloree}<SMALLFORM>(test01)"
    );

    context.set_variable(Some(utf16_string("four")), Some(value("lotwss")));
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", false, "null");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert_variable(context.as_ref(), "four", true, "lotwss");
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "SMALLFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{5:{four=lotwss},4:<SMALLFORM>,3:{two=(*removed*)},2:{one=atwe, three=twelloree}<BIGFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[5]"
    );
    assert_eq!(
        context.to_string(),
        "{one=atwe, three=twelloree, four=lotwss}<SMALLFORM>(test01)"
    );

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", false, "null");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert_variable(context.as_ref(), "four", false, "null");
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "SMALLFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{4:<SMALLFORM>,3:{two=(*removed*)},2:{one=atwe, three=twelloree}<BIGFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[4]"
    );
    assert_eq!(
        context.to_string(),
        "{one=atwe, three=twelloree}<SMALLFORM>(test01)"
    );

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", false, "null");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert_variable(context.as_ref(), "four", false, "null");
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "BIGFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{3:{two=(*removed*)},2:{one=atwe, three=twelloree}<BIGFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[3]"
    );
    assert_eq!(
        context.to_string(),
        "{one=atwe, three=twelloree}<BIGFORM>(test01)"
    );

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", true, "twellor");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert_variable(context.as_ref(), "four", false, "null");
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "BIGFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{2:{one=atwe, three=twelloree}<BIGFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[2]"
    );
    assert_eq!(
        context.to_string(),
        "{one=atwe, two=twellor, three=twelloree}<BIGFORM>(test01)"
    );

    context.set_selection_target(Some(value("MEDIUMFORM")));
    assert_variable(context.as_ref(), "one", true, "atwe");
    assert_variable(context.as_ref(), "two", true, "twellor");
    assert_variable(context.as_ref(), "three", true, "twelloree");
    assert_variable(context.as_ref(), "four", false, "null");
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "MEDIUMFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{2:{one=atwe, three=twelloree}<MEDIUMFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[2]"
    );
    assert_eq!(
        context.to_string(),
        "{one=atwe, two=twellor, three=twelloree}<MEDIUMFORM>(test01)"
    );

    context.decrease_level();
    assert!(!context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "null");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{1:{one=hello, two=twellor},0:{one=a value}(test01)}[1]"
    );
    assert_eq!(context.to_string(), "{one=hello, two=twellor}(test01)");

    context.decrease_level();
    assert!(!context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "null");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{0:{one=a value}(test01)}[0]"
    );
    assert_eq!(context.to_string(), "{one=a value}(test01)");

    context.set_selection_target(Some(value("TOTALFORM")));
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "TOTALFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{0:{one=a value}<TOTALFORM>(test01)}[0]"
    );
    assert_eq!(context.to_string(), "{one=a value}<TOTALFORM>(test01)");

    context.increase_level();
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "TOTALFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{0:{one=a value}<TOTALFORM>(test01)}[1]"
    );
    assert_eq!(context.to_string(), "{one=a value}<TOTALFORM>(test01)");

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "a value");
    assert_variable(context.as_ref(), "two", false, "null");
    assert_variable(context.as_ref(), "three", false, "null");
    assert_variable(context.as_ref(), "four", false, "null");
    assert!(context.has_selection_target());
    assert_eq!(selection_text(context.as_ref()), "TOTALFORM");
    assert_eq!(
        context.get_string_representation_by_level(),
        "{0:{one=a value}<TOTALFORM>(test01)}[0]"
    );
    assert_eq!(context.to_string(), "{one=a value}<TOTALFORM>(test01)");
}

// ===========================================================================
// EngineContextTest test10：setVariable(name, null) 语义
//   Java：containsVariable 仍为 true，getVariable 返回 null
// ===========================================================================

#[test]
fn engine_context_test10_set_variable_null_semantics_match_java() {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("configuration");
    let context = EngineContext::new(
        configuration,
        template_data("test01"),
        None,
        locale("en-US", "US"),
        None,
    );

    assert_variable(context.as_ref(), "one", false, "null");

    context.set_variable(Some(utf16_string("one")), None);
    assert_variable(context.as_ref(), "one", true, "null");

    context.set_variable(Some(utf16_string("one")), Some(value("a value")));
    assert_variable(context.as_ref(), "one", true, "a value");

    context.increase_level();
    assert_variable(context.as_ref(), "one", true, "a value");

    context.set_variable(Some(utf16_string("one")), None);
    assert_variable(context.as_ref(), "one", true, "null");

    context.decrease_level();
    assert_variable(context.as_ref(), "one", true, "a value");
}
