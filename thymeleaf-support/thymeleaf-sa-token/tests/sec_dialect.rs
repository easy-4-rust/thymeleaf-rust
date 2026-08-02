//! `thymeleaf-sa-token` sec 方言端到端测试。
//!
//! 验证：方言注册后 `sec:authorize`/`sec:authentication` 处理器、`#authentication`/
//! `#authorization` 表达式对象、以及匿名 fail-closed 行为。快照通过
//! [`SaTokenAuthenticationObject`] 注入模板上下文变量 [`AUTHENTICATION_VARIABLE`]。

use std::sync::Arc;

use thymeleaf::context::{Context, IContext};
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::JavaString;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

use thymeleaf_sa_token::{
    AUTHENTICATION_VARIABLE, SaTokenAuthentication, SaTokenAuthenticationObject, SaTokenDialect,
};

fn js(s: &str) -> JavaString {
    JavaString::from_rust_str(s)
}

fn engine() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    engine.add_dialect(Arc::new(SaTokenDialect::new())).unwrap();
    engine
}

fn render(template: &str, context: &dyn IContext) -> String {
    engine()
        .process_template(template, context)
        .unwrap()
        .to_string_lossy()
}

/// 创建注入认证快照的上下文。
fn context_with(authentication: SaTokenAuthentication) -> Context {
    let context = Context::new();
    context.set_variable(
        Some(js(AUTHENTICATION_VARIABLE)),
        Some(Arc::new(SaTokenAuthenticationObject::to_template_value(
            Arc::new(authentication),
        ))),
    );
    context
}

/// 无安全变量的匿名上下文（变量缺失 → fail-closed）。
fn anonymous_context() -> Context {
    Context::new()
}

fn admin_authentication() -> SaTokenAuthentication {
    SaTokenAuthentication::new(
        "admin-1".to_owned(),
        Arc::from([Arc::from("ROLE_ADMIN"), Arc::from("ROLE_USER")]),
        Arc::from([Arc::from("orders:write"), Arc::from("orders:read")]),
    )
}

// ===========================================================================
// 1. sec:authorize 条件可见性
// ===========================================================================

#[test]
fn sec_authorize_renders_when_expression_is_true() {
    let context = context_with(admin_authentication());
    // Java SecAuthorizeAttrProcessor 接受裸 Spring Security 授权表达式
    let output = render(
        "<div sec:authorize=\"hasRole('ROLE_ADMIN')\">admin panel</div>",
        &context,
    );
    assert!(output.contains("admin panel"), "output: {output}");
    assert!(
        !output.contains("sec:authorize"),
        "attribute removed: {output}"
    );
}

#[test]
fn sec_authorize_removes_element_when_expression_is_false() {
    let context = context_with(admin_authentication());
    let output = render(
        "<div sec:authorize=\"hasRole('ROLE_SUPERUSER')\">superuser only</div>",
        &context,
    );
    assert!(!output.contains("superuser only"), "output: {output}");
}

#[test]
fn sec_authorize_is_fail_closed_for_anonymous() {
    let context = anonymous_context();
    let output = render(
        "<div sec:authorize=\"isAuthenticated()\">private</div>",
        &context,
    );
    assert!(!output.contains("private"), "output: {output}");
}

#[test]
fn sec_authorize_supports_authorization_object_expression() {
    let context = context_with(admin_authentication());
    let output = render(
        "<div sec:authorize=\"${#authorization.hasPermission('orders:write')}\">can write</div>",
        &context,
    );
    assert!(output.contains("can write"), "output: {output}");
}

// ===========================================================================
// 2. sec:authentication 身份输出
// ===========================================================================

#[test]
fn sec_authentication_renders_name() {
    let context = context_with(admin_authentication());
    let output = render(
        "<span sec:authentication=\"name\">placeholder</span>",
        &context,
    );
    assert!(output.contains("admin-1"), "output: {output}");
    assert!(!output.contains("placeholder"), "output: {output}");
    assert!(!output.contains("sec:authentication"), "output: {output}");
}

#[test]
fn sec_authentication_renders_roles() {
    let context = context_with(admin_authentication());
    let output = render(
        "<span sec:authentication=\"roles\">placeholder</span>",
        &context,
    );
    assert!(output.contains("ROLE_ADMIN,ROLE_USER"), "output: {output}");
}

// ===========================================================================
// 3. #authentication / #authorization 表达式对象
// ===========================================================================

// 注意：HTML 模式内联插值默认关闭（与 Java Thymeleaf 一致，需 th:inline 开启），
// 表达式对象访问通过 th:text 等标准属性处理器承载。

#[test]
fn authentication_object_exposes_name() {
    let context = context_with(admin_authentication());
    let output = render("<p th:text=\"${#authentication.name}\">x</p>", &context);
    assert!(output.contains("admin-1"), "output: {output}");
}

#[test]
fn authentication_object_exposes_authenticated_flag() {
    let context = context_with(admin_authentication());
    let output = render(
        "<p th:text=\"${#authentication.isAuthenticated()}\">x</p>",
        &context,
    );
    assert!(output.contains("true"), "output: {output}");

    let anonymous = anonymous_context();
    let output = render(
        "<p th:text=\"${#authentication.isAuthenticated()}\">x</p>",
        &anonymous,
    );
    assert!(output.contains("false"), "output: {output}");
}

#[test]
fn authentication_object_has_role_and_permission_methods() {
    let context = context_with(admin_authentication());
    assert_eq!(
        render(
            "<p th:text=\"${#authentication.hasRole('ROLE_ADMIN')}\">x</p>",
            &context
        ),
        "<p>true</p>"
    );
    assert_eq!(
        render(
            "<p th:text=\"${#authentication.hasRole('ROLE_NOPE')}\">x</p>",
            &context
        ),
        "<p>false</p>"
    );
    assert_eq!(
        render(
            "<p th:text=\"${#authentication.hasPermission('orders:write')}\">x</p>",
            &context
        ),
        "<p>true</p>"
    );
    assert_eq!(
        render(
            "<p th:text=\"${#authentication.hasPermission('billing:read')}\">x</p>",
            &context
        ),
        "<p>false</p>"
    );
}

#[test]
fn authorization_object_matches_permission_wildcards() {
    // 权限存储含 "orders:*" 形式的前缀通配
    let authentication = SaTokenAuthentication::new(
        "op-1".to_owned(),
        Arc::from([]),
        Arc::from([Arc::from("orders:*")]),
    );
    let context = context_with(authentication);
    assert_eq!(
        render(
            "<p th:text=\"${#authorization.hasPermission('orders:create')}\">x</p>",
            &context
        ),
        "<p>true</p>"
    );
    assert_eq!(
        render(
            "<p th:text=\"${#authorization.hasPermission('billing:read')}\">x</p>",
            &context
        ),
        "<p>false</p>"
    );
}

#[test]
fn authorization_object_is_fail_closed_for_anonymous() {
    let context = anonymous_context();
    assert_eq!(
        render(
            "<p th:text=\"${#authorization.hasRole('ROLE_ADMIN')}\">x</p>",
            &context
        ),
        "<p>false</p>"
    );
    assert_eq!(
        render(
            "<p th:text=\"${#authorization.hasPermission('orders:write')}\">x</p>",
            &context
        ),
        "<p>false</p>"
    );
}

// ===========================================================================
// 4. 方言注册后 sec: 属性被处理（无 sec 方言时原样保留）
// ===========================================================================

#[test]
fn sec_attributes_are_processed_only_with_dialect() {
    let context = context_with(admin_authentication());
    let engine = engine();

    let with_dialect = engine
        .process_template(
            "<div sec:authorize=\"hasRole('ROLE_ADMIN')\">x</div>",
            &context,
        )
        .unwrap()
        .to_string_lossy();
    assert!(!with_dialect.contains("sec:authorize"), "{with_dialect}");

    // 无方言的引擎保留 sec: 属性
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let plain_engine = TemplateEngine::new();
    plain_engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let plain = plain_engine
        .process_template(
            "<div sec:authorize=\"hasRole('ROLE_ADMIN')\">x</div>",
            &context,
        )
        .unwrap()
        .to_string_lossy();
    assert!(plain.contains("sec:authorize"), "{plain}");
}
