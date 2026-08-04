//! `AbstractStandardFragmentInsertionTagProcessor` Java 1:1 差分测试。
//!
//! 对应上游 `thymeleaf-tests-core` 的
//! `org.thymeleaf.standard.processor.FragmentInsertionExpressionTest`：
//! 43 个旧式片段引用表达式的 `shouldBeWrappedAsFragmentExpression`
//! 判定逐例转录。

use thymeleaf::processor::AbstractStandardFragmentInsertionTagProcessor;
use thymeleaf::util::Utf16String;

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn check(expression: &str, expected: bool) {
    let actual =
        AbstractStandardFragmentInsertionTagProcessor::should_be_wrapped_as_fragment_expression(
            &js(expression),
        );
    assert_eq!(actual, expected, "expression: {expression}");
}

#[test]
fn fragment_expression_selection() {
    // 对应 Java FragmentInsertionExpressionTest#testFragmentExpressionSelection
    check("template", true);
    check("template::f", true);
    check("template::frag", true);
    check("template :: frag", true);
    check("  template :: frag   ", true);
    check("   :: frag   ", true);
    check("::frag   ", true);
    check("::frag", true);
    check("this::frag", true);
    check(" this   ::frag", true);
    check(" this   :: frag", true);
    check(" ${lala slatr} + 'ele'   :: 'index_' + 2 * 2", true);
    check(" ${lala slatr} + 'ele'   :: ('index_' + 2 * 2)", true);
    check(
        " ${lala slatr} + 'ele'   :: ('index_' + (2 * 2)) (somePar)",
        true,
    );
    check(
        " ${lala slatr} + 'ele'   :: ('index_' + (2 * 2)) (a='something')",
        true,
    );
    check(
        " ${lala slatr} + 'ele'   :: ('index_' + (2 * 2)) (a='something',b=4123)",
        true,
    );
    check(
        " ${lala slatr} + 'ele'   :: ('index_' + (2 * 2)) (a=('something'),b=4123)",
        true,
    );
    check(
        " ${lala slatr} + ('ele')   :: ('index_' + (2 * 2)) (a=('something'),b=4123)",
        true,
    );
    check(
        " ${lala slatr} + ('ele')   :: ('index_' + (2 * 2)) (a=('something' + 23),b=4123)",
        true,
    );
    check(
        " ${lala slatr}+'ele'   :: ('index_'+(2*2)) (a=('something'+23),b=4123)",
        true,
    );
    check(
        " ${lala slatr}+'ele'   :: ('index_'+(2*2)) (${name}=('something'+23),b=4123)",
        true,
    );
    check(
        " ${lala slatr}+'ele'   :: ('index_'+(2*2)) ((${name} + 0)=('something'+23),b=4123)",
        true,
    );
    check(
        "C:\\Program Files\\apps\\templates\\WEB-INF\\temp.html",
        true,
    );
    check(
        "C:\\Program Files\\apps\\templates\\WEB-INF\\temp.html :: 'fragment number one'",
        true,
    );
    check(
        "/home/user/apps/templates/WEB-INF/temp.html :: 'fragment number one'",
        true,
    );
    check("home/user :: 'fragment number one'", true);
    check("${something}", true);
    check("${this} :: ${that}", true);
    check("~{whatever}", false);
    check("${cond} ? ~{this} : ~{that}", false);
    check("${something} :: /div", true);
    check("template :: f (~{some})", true);
    check("folder/template :: f (~{some})", true);
    check("folder/template :: f (~{some})", true);
    check("~folder/template :: f (~{some})", true);
    check("~/folder/template :: f (~{some})", true);
    check("${~{impossible}} :: f (~{some})", true);
    check("'~{impossible}' :: f (~{some})", true);
    check("folder/template (title=~{some})", true);
    check("(~{some})", false);
    check("(${cond}) ? (~{this}) : (~{that})", false);
    check("folder/template (title='one',body=~{that})", true);
    check("folder/template (title=(~{some}))", true);
    check("folder/template (title=('one'),body=(~{that}))", true);
    check("folder/template (title=('one'))", true);
    check("folder/template (body=~{(that)})", true);
    check("folder/template\n (body=~{(that)})", true);
    check("~{folder/template :: f (~{some})}", false);
    check("     ~{folder/template :: f (~{some})}   ", false);
    // 上游注释：无片段规范的模板不能带合成参数调用 → false
    check("folder/template (~{some})", false);
}
