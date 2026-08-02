//! `WebEngineContextTest` 直接差分（Java 21 实测）——多层遮蔽、表示串、
//! selection target、template stack 与 exchange 直写可见性。
//!
//! 与 `engine_context_java_parity.rs`（Golden 锚点）互补：本文件逐方法 1:1
//! 复刻上游 `org.thymeleaf.engine.WebEngineContextTest` 的
//! test08（变量表示串 / removeVariable null 占位 / isVariableLocal 翻转）、
//! test11（多级 selection target 表示）、test13（templateData/templateStack
//! 多级 push/pop）、test04（exchange 直写实时可见、降层不回滚）的关键断言。
//! 期望字符串逐字取自 Java 测试（Locale.US、模板 "test01"）。

#![allow(dead_code, unused_imports)]

mod support;

use std::sync::Arc;

use support::CorpusWebExchange;
use thymeleaf::cache::AlwaysValidCacheEntryValidity;
use thymeleaf::context::{
    IContext, IEngineContext, IExpressionContext, ITemplateContext, WebEngineContext,
};
use thymeleaf::engine::TemplateData;
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresource::StringTemplateResource;
use thymeleaf::util::{JavaLocale, JavaString};
use thymeleaf::web::IWebExchange;
use thymeleaf::{ITemplateEngine, TemplateEngine, TemplateMode};

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn value(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::string(js(value)))
}

fn locale_en() -> JavaLocale {
    JavaLocale::new(js("en"), js("US"))
}

fn template_data(name: &str, mode: TemplateMode) -> TemplateData {
    TemplateData::new(
        Some(js(name)),
        None,
        Some(Arc::new(
            StringTemplateResource::new(Some(name)).expect("resource"),
        )),
        Some(mode),
        Some(Arc::new(AlwaysValidCacheEntryValidity::new())),
    )
}

/// 构造与 Java `new WebEngineContext(config, templateData, null, exchange, LOCALE, null)`
/// 等价的 Web 引擎上下文（无初始变量）。返回具体类型以访问
/// `get_string_representation_by_level` 与 Display（表示串不在 trait 上）。
fn web_context(name: &str, mode: TemplateMode) -> (Arc<WebEngineContext>, Arc<dyn IWebExchange>) {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("engine configuration");
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let context = WebEngineContext::new(
        configuration,
        template_data(name, mode),
        None,
        Arc::clone(&exchange),
        locale_en(),
        None,
    );
    (context, exchange)
}

fn variable(context: &dyn IContext, name: &str) -> String {
    context
        .get_variable(Some(&js(name)))
        .and_then(|value| value.to_java_string())
        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy())
}

fn names(context: &dyn IEngineContext) -> Vec<String> {
    let mut snapshot: Vec<String> = context
        .get_variable_names()
        .snapshot()
        .into_iter()
        .flatten()
        .map(|name| name.to_string_lossy())
        .collect();
    snapshot.sort();
    snapshot
}

// ===========================================================================
// test08：变量表示串 / removeVariable null 占位 / isVariableLocal 翻转
// ===========================================================================

#[test]
fn web_engine_context_representation_matches_java_test08() {
    let (vm, _) = web_context("test01", TemplateMode::HTML);

    vm.set_variable(Some(js("one")), Some(value("a value")));
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{0:{one=a value}(test01)}[0]"
    );
    assert_eq!(vm.to_string(), "{one=a value}(test01)");
    assert_eq!(names(vm.as_ref()), vec!["one"]);
    assert!(!vm.is_variable_local(Some(&js("one"))));

    vm.set_variable(Some(js("one")), Some(value("two values")));
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{0:{one=two values}(test01)}[0]"
    );
    assert_eq!(vm.to_string(), "{one=two values}(test01)");

    vm.remove_variable(Some(&js("one")));
    assert_eq!(vm.get_string_representation_by_level(), "{0:{}(test01)}[0]");
    assert_eq!(vm.to_string(), "{}(test01)");
    assert!(names(vm.as_ref()).is_empty());

    vm.set_variable(Some(js("one")), Some(value("two values")));
    assert!(!vm.is_variable_local(Some(&js("one"))));

    // 层级遮蔽与 isVariableLocal 翻转
    vm.increase_level();
    assert!(!vm.is_variable_local(Some(&js("one"))));
    vm.set_variable(Some(js("one")), Some(value("hello")));
    assert!(vm.is_variable_local(Some(&js("one"))));
    vm.decrease_level();
    assert!(!vm.is_variable_local(Some(&js("one"))));

    vm.increase_level();
    vm.set_variable(Some(js("one")), Some(value("hello")));
    assert!(vm.is_variable_local(Some(&js("one"))));
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{1:{one=hello},0:{one=two values}(test01)}[1]"
    );
    assert_eq!(vm.to_string(), "{one=hello}(test01)");
    assert_eq!(names(vm.as_ref()), vec!["one"]);

    // removeVariable -> 该层 null 占位，有效视图清空
    vm.remove_variable(Some(&js("one")));
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{1:{one=null},0:{one=two values}(test01)}[1]"
    );
    assert_eq!(vm.to_string(), "{}(test01)");
    assert!(names(vm.as_ref()).is_empty());

    vm.set_variable(Some(js("one")), Some(value("hello")));
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{1:{one=hello},0:{one=two values}(test01)}[1]"
    );
    assert_eq!(vm.to_string(), "{one=hello}(test01)");

    // 删除不存在的变量无副作用
    vm.remove_variable(Some(&js("two")));
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{1:{one=hello},0:{one=two values}(test01)}[1]"
    );

    vm.set_variable(Some(js("two")), Some(value("twello")));
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{1:{one=hello, two=twello},0:{one=two values}(test01)}[1]"
    );
    assert_eq!(vm.to_string(), "{one=hello, two=twello}(test01)");
    assert_eq!(names(vm.as_ref()), vec!["one", "two"]);

    vm.remove_variable(Some(&js("two")));
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{1:{one=hello},0:{one=two values}(test01)}[1]"
    );
    assert_eq!(names(vm.as_ref()), vec!["one"]);
}

// ===========================================================================
// test11：多级 selection target 表示（BIGFORM / SMALLFORM 链）
// ===========================================================================

#[test]
fn web_engine_context_selection_target_representation_matches_java_test11() {
    let (vm, _) = web_context("test01", TemplateMode::HTML);

    vm.set_variable(Some(js("one")), Some(value("a value")));
    assert!(!vm.has_selection_target());
    assert!(vm.get_selection_target().is_none());
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{0:{one=a value}(test01)}[0]"
    );

    vm.increase_level();
    vm.set_variable(Some(js("one")), Some(value("hello")));
    vm.remove_variable(Some(&js("one")));
    vm.set_variable(Some(js("one")), Some(value("hello")));
    vm.remove_variable(Some(&js("two")));
    vm.set_variable(Some(js("two")), Some(value("twello")));
    vm.set_variable(Some(js("two")), Some(value("twellor")));
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{1:{one=hello, two=twellor},0:{one=a value}(test01)}[1]"
    );

    vm.increase_level();
    vm.set_variable(Some(js("three")), Some(value("twelloree")));
    vm.set_variable(Some(js("one")), Some(value("atwe")));
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{2:{three=twelloree, one=atwe},1:{one=hello, two=twellor},0:{one=a value}(test01)}[2]"
    );
    assert_eq!(
        vm.to_string(),
        "{one=atwe, two=twellor, three=twelloree}(test01)"
    );

    vm.set_selection_target(Some(value("BIGFORM")));
    assert!(vm.has_selection_target());
    assert_eq!(
        variable(vm.as_ref(), "one"),
        "atwe",
        "selection target 不改变变量读取"
    );
    assert_eq!(
        vm.to_string(),
        "{one=atwe, two=twellor, three=twelloree}<BIGFORM>(test01)"
    );
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{2:{three=twelloree, one=atwe}<BIGFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[2]"
    );

    vm.increase_level();
    vm.remove_variable(Some(&js("two")));
    assert!(!vm.contains_variable(Some(&js("two"))));
    assert_eq!(variable(vm.as_ref(), "one"), "atwe");
    assert_eq!(variable(vm.as_ref(), "two"), "null");
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{3:{two=null},2:{three=twelloree, one=atwe}<BIGFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[3]"
    );
    assert_eq!(
        vm.to_string(),
        "{one=atwe, three=twelloree}<BIGFORM>(test01)"
    );

    vm.increase_level();
    vm.remove_variable(Some(&js("two")));
    vm.set_selection_target(Some(value("SMALLFORM")));
    assert_eq!(
        vm.get_string_representation_by_level(),
        "{4:<SMALLFORM>,3:{two=null},2:{three=twelloree, one=atwe}<BIGFORM>,1:{one=hello, two=twellor},0:{one=a value}(test01)}[4]"
    );
    assert_eq!(
        vm.to_string(),
        "{one=atwe, three=twelloree}<SMALLFORM>(test01)"
    );

    // 降层恢复父级 selection target
    vm.decrease_level();
    assert_eq!(
        variable(vm.as_ref(), "two"),
        "null",
        "降层后 two 仍被该层 null 占位遮蔽"
    );
    assert_eq!(
        vm.to_string(),
        "{one=atwe, three=twelloree}<BIGFORM>(test01)"
    );

    // 继续降层直至根层：selection 与层级变量全部恢复
    vm.decrease_level();
    vm.decrease_level();
    vm.decrease_level();
    assert!(!vm.has_selection_target());
    assert_eq!(variable(vm.as_ref(), "two"), "null");
    assert_eq!(vm.to_string(), "{one=a value}(test01)");
}

// ===========================================================================
// test13：templateData / templateStack 多级 push/pop
// ===========================================================================

#[test]
fn web_engine_context_template_stack_matches_java_test13() {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("engine configuration");
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let template1 = template_data("test01", TemplateMode::HTML);
    let template2 = template_data("test02", TemplateMode::HTML);
    let template3 = template_data("test03", TemplateMode::XML);
    let vm = WebEngineContext::new(
        configuration,
        template1.clone(),
        None,
        exchange,
        locale_en(),
        None,
    );

    let stack = || -> Vec<String> {
        vm.get_template_stack()
            .into_iter()
            .map(|t| t.get_template().expect("template").to_string_lossy())
            .collect()
    };

    assert_eq!(stack(), vec!["test01"]);
    assert_eq!(
        vm.get_template_data()
            .get_template()
            .expect("template")
            .to_string_lossy(),
        "test01"
    );

    vm.increase_level();
    vm.set_template_data(Arc::new(template2.clone()));
    assert_eq!(stack(), vec!["test01", "test02"]);
    assert_eq!(
        vm.get_template_data()
            .get_template()
            .expect("template")
            .to_string_lossy(),
        "test02"
    );

    vm.increase_level();
    vm.set_template_data(Arc::new(template3.clone()));
    assert_eq!(stack(), vec!["test01", "test02", "test03"]);
    assert_eq!(
        vm.get_template_data().get_template_mode(),
        Some(TemplateMode::XML)
    );

    vm.set_variable(Some(js("three")), Some(value("twelloree")));
    vm.set_variable(Some(js("one")), Some(value("atwe")));
    assert_eq!(variable(vm.as_ref(), "one"), "atwe");
    assert_eq!(variable(vm.as_ref(), "two"), "null");
    assert_eq!(variable(vm.as_ref(), "three"), "twelloree");

    // 降层恢复
    vm.decrease_level();
    assert_eq!(stack(), vec!["test01", "test02"]);
    assert_eq!(variable(vm.as_ref(), "two"), "null");
    vm.decrease_level();
    assert_eq!(stack(), vec!["test01"]);
    assert_eq!(
        vm.get_template_data()
            .get_template()
            .expect("template")
            .to_string_lossy(),
        "test01"
    );
}

// ===========================================================================
// test04：exchange 直写实时可见、降层不回滚（Java test04 核心语义）
// ===========================================================================

#[test]
fn web_engine_context_exchange_direct_write_visible_without_rollback() {
    let (vm, exchange) = web_context("test01", TemplateMode::HTML);

    vm.set_variable(Some(js("one")), Some(value("a value")));
    vm.set_variable(Some(js("ten")), Some(value("tieen")));
    assert_eq!(variable(vm.as_ref(), "one"), "a value");
    assert_eq!(variable(vm.as_ref(), "ten"), "tieen");

    // 直写 exchange 属性（Java `mockRequest.setAttribute`）-> 实时可见
    exchange.set_attribute_value(Some(js("one")), Some(value("outer1")));
    exchange.set_attribute_value(Some(js("six")), Some(value("outer6")));
    assert_eq!(variable(vm.as_ref(), "one"), "outer1");
    assert_eq!(variable(vm.as_ref(), "six"), "outer6");

    // 层级变更不回滚 exchange 直写（直写不在 local_changes 内）
    vm.increase_level();
    assert_eq!(variable(vm.as_ref(), "one"), "outer1");
    assert_eq!(variable(vm.as_ref(), "six"), "outer6");
    vm.set_variable(Some(js("one")), Some(value("hello")));
    assert_eq!(variable(vm.as_ref(), "one"), "hello");
    vm.decrease_level();
    assert_eq!(
        variable(vm.as_ref(), "one"),
        "outer1",
        "降层只回滚本层 set_variable，exchange 直写保留"
    );
    assert_eq!(variable(vm.as_ref(), "six"), "outer6");
}
