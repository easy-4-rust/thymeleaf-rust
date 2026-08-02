//! `org.thymeleaf.standard.expression` 对象族 Java 1:1 差分测试。
//!
//! 转写上游 `thymeleaf-tests-core`：
//!
//! 1. `LiteralSubstitutionUtilTest#testLiteralSubstitution` —— `|...|`
//!    字面量替换断言。Rust 侧 `LiteralSubstitutionUtil` 为包内
//!    API（Java 对应类为 public），差分走公开入口
//!    `StandardExpressionParser#parseExpression` 的字符串表示；
//!    多操作数连接的 Java 工具输出在解析后呈左结合括号化
//!    （`x + y + z` → `(x + y) + z`，与 Java AST `toString` 的括号化
//!    规则一致），此类用例断言括号化后的精确形式；
//!    两个含转义反斜杠的用例（`|a 'one' b|`、`|a \\'one\\' b|`）为
//!    工具级转义语义（Java 直测工具函数），Rust 公开解析路径会先经过
//!    Standard 预处理，如实记录为 RUST_OBLIGATION，不伪称 MATCH；
//! 2. `FragmentSignatureTest#testFragmentSignature` —— 5 个断言，
//!    经 `FragmentSignatureUtils#parseFragmentSignature` 公开入口。
//!
//! 覆盖对象（对象表编号）：`LiteralSubstitutionUtil`（对应 Java 类）、
//! `FragmentSignatureUtils`、`StandardExpressionParser`、
//! `ExpressionParsingUtil`（经解析路径）、`ExpressionSequence`、
//! `Each`（经 `parseEach`）。

use std::sync::Arc;

use thymeleaf::context::ExpressionContext;
use thymeleaf::expression::{
    FragmentSignatureUtils, IStandardExpressionParser, StandardExpressionParser,
};
use thymeleaf::util::JavaString;
use thymeleaf::{ITemplateEngine, ITemplateResolver, TemplateEngine};

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn expression_context() -> Arc<dyn thymeleaf::context::IExpressionContext> {
    let engine = TemplateEngine::new();
    let resolver = thymeleaf::templateresolver::StringTemplateResolver::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("set resolver");
    let configuration = engine.get_configuration().expect("configuration");
    ExpressionContext::new(Some(configuration)).expect("expression context")
}

/// 解析表达式并返回其字符串表示（对应 Java `Expression#getStringRepresentation`）。
fn parse_string(input: &str) -> String {
    let context = expression_context();
    let parser = StandardExpressionParser::new();
    let expression = parser
        .parse_expression(context.as_ref(), Some(&js(input)))
        .unwrap_or_else(|error| panic!("{input:?} 解析失败: {error}"));
    expression
        .get_string_representation()
        .expect("string representation")
        .to_string_lossy()
}

// ===========================================================================
// 1. LiteralSubstitutionUtilTest#testLiteralSubstitution
// ===========================================================================

/// 直接匹配 Java 工具输出（解析后字符串表示与替换文本一致）。
#[test]
fn literal_substitution_plain_matches_java() {
    for (input, expected) in [
        ("|${one}      |", "${one} + '      '"),
        ("|     ${one}|", "'     ' + ${one}"),
        ("${one}", "${one}"),
        ("'lalala'", "'lalala'"),
        ("null", "null"),
        ("null and token", "null and token"),
        ("4123.4l and token", "4123.4l and token"),
        ("'Sum: ' + (10 + 2)", "'Sum: ' + (10 + 2)"),
        ("'Sum: ' + |10 + 2|", "'Sum: ' + '10 + 2'"),
        ("'Sum: ' + |10 + 'aaa 2|", "'Sum: ' + '10 + \\'aaa 2'"),
        ("|Sum: | + |10 + 2|", "'Sum: ' + '10 + 2'"),
        ("${one}${two}", "${one}${two}"),
    ] {
        assert_eq!(
            expected,
            parse_string(input),
            "LiteralSubstitution({input:?}) 与 Java 输出不一致"
        );
    }
}

/// 多操作数连接：Java 工具输出 `x + y + z`，解析后按 AST 括号化规则
/// 呈 `(x + y) + z`（Java `AdditionExpression#toString` 同样完整括号化）。
#[test]
fn literal_substitution_left_associative_bracketing_matches_java_ast() {
    for (input, java_substitution, bracketed) in [
        (
            "|${one} ${two}|",
            "${one} + ' ' + ${two}",
            "(${one} + ' ') + ${two}",
        ),
        (
            "|     ${one}      |",
            "'     ' + ${one} + '      '",
            "('     ' + ${one}) + '      '",
        ),
        (
            "|${one} et ${two}|",
            "${one} + ' et ' + ${two}",
            "(${one} + ' et ') + ${two}",
        ),
        (
            "|Welcome, ${one} to application with name #{two}|",
            "'Welcome, ' + ${one} + ' to application with name ' + #{two}",
            "(('Welcome, ' + ${one}) + ' to application with name ') + #{two}",
        ),
        (
            "|Welcome, | + |${one}| + | to application| + ' with' + | name #{two}|",
            "'Welcome, ' + ${one} + ' to application' + ' with' + ' name ' + #{two}",
            "(((('Welcome, ' + ${one}) + ' to application') + ' with') + ' name ') + #{two}",
        ),
        (
            "|${one}${two}|",
            "${one} + '' + ${two}",
            "(${one} + '') + ${two}",
        ),
    ] {
        let actual = parse_string(input);
        assert_eq!(
            bracketed, actual,
            "LiteralSubstitution({input:?}) 括号化形式不一致；Java 替换文本: {java_substitution}"
        );
    }
}

// ===========================================================================
// 2. FragmentSignatureTest#testFragmentSignature
// ===========================================================================

/// 对应 Java FragmentSignatureTest：签名名称与形参名解析。
#[test]
fn fragment_signature_matches_java() {
    let signature = |spec: &str| {
        FragmentSignatureUtils::parse_fragment_signature(None, Some(&js(spec)))
            .expect("fragment signature")
    };

    let check = |spec: &str, name: &str, parameters: Option<&[&str]>| {
        let parsed = signature(spec);
        assert_eq!(
            name,
            parsed.get_fragment_name().to_string_lossy(),
            "签名 {spec:?} 的名称不匹配"
        );
        match parameters {
            None => assert!(
                parsed.get_parameter_names().is_none(),
                "签名 {spec:?} 不应有形参"
            ),
            Some(expected) => {
                let actual = parsed
                    .get_parameter_names()
                    .expect("parameter names")
                    .iter()
                    .map(|name| {
                        name.as_ref()
                            .map_or_else(|| "null".to_owned(), |n| n.to_string_lossy())
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    expected
                        .iter()
                        .map(|name| (*name).to_owned())
                        .collect::<Vec<_>>(),
                    actual,
                    "签名 {spec:?} 的形参不匹配"
                );
            }
        }
    };

    check("frag", "frag", None);
    check("     frag ", "frag", None);
    check("     frag ()", "frag", None);
    check("     frag (as)", "frag", Some(&["as"]));
    check(
        "     frag ( asd , 231fa., asaad    )",
        "frag",
        Some(&["asd", "231fa.", "asaad"]),
    );
}
