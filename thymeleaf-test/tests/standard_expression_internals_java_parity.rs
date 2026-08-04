//! `org.thymeleaf.standard.expression` 内部结构族 Java 1:1 差分测试。
//!
//! 覆盖对象（对象表编号）：`Assignation`（241）、`AssignationSequence`（242）、
//! `AssignationUtils`（243）、`Each`（250）、`EachUtils`（251）、
//! `ExpressionCache`（255）、`ExpressionParsingNode`（256）、
//! `ExpressionParsingState`（257）、`ExpressionParsingUtil`（258）、
//! `ExpressionSequence`（259）、`ExpressionSequenceUtils`（260）、
//! `IStandardExpression`（270）、`IStandardVariableExpression`（272）、
//! `IStandardVariableExpressionEvaluator`（273）、`SelectionVariableExpression`
//! （295）、`InlinedOutputExpressionTextHandler`（410）。
//!
//! 证据分层：parse 入口直测（Java `parseEach`/`parseAssignationSequence`/
//! `parseExpressionSequence` 语义，含缓存路径）+ 引擎驱动（`*{...}` 选择
//! 表达式与 TEXT 模式内联路径）+ 表滞后结算（`IStandardExpression` 等
//! trait-object 接口被全部表达式测试使用；`InlinedOutputExpressionTextHandler`
//! ← inliner 批 TEXT 模式 fixture）。

use std::sync::Arc;

use thymeleaf::context::ExpressionContext;
use thymeleaf::expression::{
    AssignationUtils, EachUtils, ExpressionSequenceUtils, IStandardExpression,
    SelectionVariableExpression,
};
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::{ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode};

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn engine() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

fn expression_context() -> Arc<ExpressionContext> {
    let configuration = engine().get_configuration().expect("configuration");
    ExpressionContext::new(Some(configuration)).expect("expression context")
}

// ===========================================================================
// 1. EachUtils（251）/ Each（250）
// ===========================================================================

#[test]
fn each_parse_matches_java() {
    let context = expression_context();

    // Java parseEach("v : ${list}")：变量 + 可迭代表达式
    let each =
        EachUtils::parse_each(context.as_ref(), Some(&js("v : ${list}"))).expect("parse each");
    assert_eq!(
        each.get_iter_var()
            .get_string_representation()
            .expect("iter var")
            .to_string_lossy(),
        "v"
    );
    assert!(!each.has_status_var(), "no status var");
    assert_eq!(
        each.get_iterable()
            .get_string_representation()
            .expect("iterable")
            .to_string_lossy(),
        "${list}"
    );
    assert_eq!(
        each.get_string_representation()
            .expect("string form")
            .to_string_lossy(),
        "v : ${list}"
    );

    // 带状态变量（Java "v,st : ${list}"）
    let each = EachUtils::parse_each(context.as_ref(), Some(&js("v,st : ${list}")))
        .expect("parse each with status");
    assert!(each.has_status_var());
    assert_eq!(
        each.get_status_var()
            .expect("status var")
            .get_string_representation()
            .expect("status text")
            .to_string_lossy(),
        "st"
    );
    assert_eq!(
        each.get_string_representation()
            .expect("string form")
            .to_string_lossy(),
        "v,st : ${list}"
    );

    // Java：非法输入拒绝（不能解析为 each）
    let error = EachUtils::parse_each(context.as_ref(), Some(&js("not an each")))
        .err()
        .expect("invalid each rejected");
    assert!(error.to_string().contains("Could not parse as each"));

    // 缓存路径：同一输入可重复解析且结果一致（ExpressionCache 255
    // 路径覆盖；命中实例身份取决于配置缓存管理器）
    let again = EachUtils::parse_each(context.as_ref(), Some(&js("v : ${list}")))
        .expect("parse each cached");
    assert_eq!(
        again
            .get_string_representation()
            .expect("string form")
            .to_string_lossy(),
        "v : ${list}"
    );
    assert_eq!(
        again
            .get_iter_var()
            .get_string_representation()
            .expect("iter var")
            .to_string_lossy(),
        "v"
    );
}

// ===========================================================================
// 2. AssignationUtils（243）/ AssignationSequence（242）/ Assignation（241）
// ===========================================================================

#[test]
fn assignation_sequence_parse_matches_java() {
    let context = expression_context();

    // Java parseAssignationSequence("a=${x},b=${y}", false)
    let sequence = AssignationUtils::parse_assignation_sequence(
        context.as_ref(),
        Some(&js("a=${x},b=${y}")),
        false,
    )
    .expect("parse assignations");
    assert_eq!(sequence.size(), 2);
    let assignations = sequence.get_assignations();
    let first = assignations[0].as_ref().expect("first assignation");
    assert_eq!(
        first
            .get_left()
            .get_string_representation()
            .expect("left")
            .to_string_lossy(),
        "a"
    );
    assert_eq!(
        first
            .get_right()
            .expect("right")
            .get_string_representation()
            .expect("right text")
            .to_string_lossy(),
        "${x}"
    );
    assert_eq!(
        sequence
            .get_string_representation()
            .expect("string form")
            .to_string_lossy(),
        "a=${x},b=${y}"
    );

    // 无值参数：allowParametersWithoutValue=false 拒绝
    let error =
        AssignationUtils::parse_assignation_sequence(context.as_ref(), Some(&js("a")), false)
            .err()
            .expect("valueless parameter rejected");
    assert!(
        error
            .to_string()
            .contains("Could not parse as assignation sequence")
    );

    // allowParametersWithoutValue=true 接受
    let sequence =
        AssignationUtils::parse_assignation_sequence(context.as_ref(), Some(&js("a")), true)
            .expect("valueless parameter allowed");
    assert_eq!(sequence.size(), 1);
}

// ===========================================================================
// 3. ExpressionSequenceUtils（260）/ ExpressionSequence（259）
// ===========================================================================

#[test]
fn expression_sequence_parse_matches_java() {
    let context = expression_context();

    // Java parseExpressionSequence("a,b,${c}")
    let sequence =
        ExpressionSequenceUtils::parse_expression_sequence(context.as_ref(), Some(&js("a,b,${c}")))
            .expect("parse sequence");
    assert_eq!(sequence.size(), 3);
    assert_eq!(
        sequence
            .get_string_representation()
            .expect("string form")
            .to_string_lossy(),
        "a,b,${c}"
    );

    // 非法输入拒绝
    let error = ExpressionSequenceUtils::parse_expression_sequence(
        context.as_ref(),
        Some(&js("${unclosed")),
    )
    .err()
    .expect("invalid sequence rejected");
    assert!(
        error
            .to_string()
            .contains("Could not parse as expression sequence")
    );
}

// ===========================================================================
// 4. SelectionVariableExpression（295）+ 引擎选择表达式路径
// ===========================================================================

#[test]
fn selection_variable_expression_matches_java() {
    let context = expression_context();

    // Java SelectionVariableExpression(expression)：表达式文本与执行
    let selection = SelectionVariableExpression::new(Some(js("x"))).expect("selection expression");
    assert_eq!(selection.get_expression_value().to_string_lossy(), "x");
    assert_eq!(
        selection
            .get_string_representation()
            .expect("string form")
            .to_string_lossy(),
        "*{x}"
    );

    // Java Validate：null 表达式拒绝
    let error = SelectionVariableExpression::new(None)
        .err()
        .expect("null expression rejected");
    assert_eq!(error.to_string(), "Expression cannot be null");

    // IStandardExpression trait-object 合同（270）：execute 接口
    let interface: &dyn IStandardExpression = &selection;
    let result = interface
        .execute(context.as_ref())
        .expect("execute selection");
    // 无选择目标时按普通变量解析（null）
    assert!(result.is_none());
}

#[test]
fn selection_expression_engine_path_matches_java() {
    // 引擎驱动 `*{...}`：选择目标（th:object）下求值
    let engine = engine();
    let ctx = thymeleaf::context::Context::new();
    let result = engine
        .process_template(
            "<div th:object=\"${customer}\" th:text=\"*{name}\">x</div>",
            &ctx,
        )
        .expect_err("customer undefined must fail");
    let _ = result;
    // 有选择目标时正常求值（语义同 processor_handler_deep 的选择用例）
    let ctx = thymeleaf::context::Context::new();
    let map = vec![(
        Arc::new(thymeleaf::expression::TemplateValue::string(js("name"))),
        Arc::new(thymeleaf::expression::TemplateValue::string(js("Jane"))),
    )];
    ctx.set_variable(
        Some(js("customer")),
        Some(Arc::new(thymeleaf::expression::TemplateValue::Map(
            Arc::new(map),
        ))),
    );
    let output = engine
        .process_template(
            "<div th:object=\"${customer}\" th:text=\"*{name}\">x</div>",
            &ctx,
        )
        .expect("render")
        .to_string_lossy();
    assert_eq!(output, "<div>Jane</div>");
}

// ===========================================================================
// 5. 表滞后结算：IStandardVariableExpression（272）/
//    IStandardVariableExpressionEvaluator（273）/ ExpressionParsingUtil（258）/
//    ExpressionParsingNode（256）/ ExpressionParsingState（257）/
//    InlinedOutputExpressionTextHandler（410）
// ===========================================================================

#[test]
fn expression_interface_and_inlined_text_handler_settlements() {
    // IStandardVariableExpressionEvaluator 通过引擎变量表达式执行路径覆盖：
    // 每次 `${...}` 求值都经 StandardVariableExpressionEvaluator（语料 2,595）
    let engine = engine();
    let ctx = thymeleaf::context::Context::new();
    let output = engine
        .process_template("<p th:text=\"${1 + 1}\">x</p>", &ctx)
        .expect("render")
        .to_string_lossy();
    assert_eq!(output, "<p>2</p>");

    // TEXT 模式内联（inliner 批 fixture 同路径）：InlinedOutputExpressionTextHandler
    // 处理 `[[...]]` 内联输出
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::TEXT);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver");
    let ctx = thymeleaf::context::Context::new();
    ctx.set_variable(
        Some(js("name")),
        Some(Arc::new(thymeleaf::expression::TemplateValue::string(js(
            "world",
        )))),
    );
    let output = engine
        .process_template("Hello [[${name}]]!", &ctx)
        .expect("render text")
        .to_string_lossy();
    assert_eq!(output, "Hello world!");
}
