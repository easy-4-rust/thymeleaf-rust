//! 安全模型专项测试：ThymeleafACL + 表达式外部访问封禁。
//!
//! 对应 Java：`ExpressionUtils.isTypeForbidden` / `isMemberForbidden` +
//! `StandardExpressionUtils.containsExternalAccess` 的行为合同。
//! 这些 API 是 pub，直接从 `thymeleaf::util` 调用。

use thymeleaf::util::{ExpressionUtils, StandardExpressionUtils};

// ===========================================================================
// is_type_forbidden：ACL 类白名单
// ===========================================================================

#[test]
fn allowed_java_classes_are_not_forbidden() {
    // 白名单内 ~50 个类——Java 核量类型、集合、日期时间、OgnlRuntime
    for allowed in &[
        "java.lang.String",
        "java.lang.Integer",
        "java.lang.Math",
        "java.math.BigDecimal",
        "java.util.HashMap",
        "java.util.ArrayList",
        "java.time.LocalDateTime",
    ] {
        assert!(
            !ExpressionUtils::is_type_forbidden(allowed),
            "白名单类 {allowed} 不应被禁止"
        );
    }
}

#[test]
fn blocked_package_prefixes_are_forbidden() {
    // 非白名单的 java.*/javax.*/jakarta.*/jdk.* 等包前缀下类全被禁止
    for forbidden in &[
        "java.lang.Runtime",
        "java.lang.ProcessBuilder",
        "java.lang.System",
        "java.io.FileInputStream",
        "javax.script.ScriptEngine",
        "jakarta.servlet.Servlet",
        "jdk.management.Resource",
        "org.w3c.dom.Document",
    ] {
        assert!(
            ExpressionUtils::is_type_forbidden(forbidden),
            "封禁类 {forbidden} 应被禁止"
        );
    }
}

#[test]
fn arbitrary_non_java_classes_are_forbidden() {
    // 完全不在白名单中的类——即使是非 java 包也禁止
    assert!(ExpressionUtils::is_type_forbidden("com.example.Evil"));
    assert!(ExpressionUtils::is_type_forbidden(
        "org.springframework.context.ApplicationContext"
    ));
    assert!(ExpressionUtils::is_type_forbidden(""));
}

// ===========================================================================
// is_member_forbidden：成员访问白名单
// ===========================================================================

#[test]
fn allowed_class_methods_are_not_forbidden() {
    // toString / getClass / hashCode / equals / compareTo 等始终允许
    for allowed in &["toString", "getClass", "hashCode", "equals", "compareTo"] {
        // 不提供 target（None）走通用 ALLOWED_CLASS_METHODS 路径
        assert!(
            !ExpressionUtils::is_member_forbidden(None, allowed),
            "通用允许方法 {allowed} 不应被禁止"
        );
    }
}

#[test]
fn dangerous_methods_are_forbidden_without_target() {
    // ClassLoader.loadClass / Process.waitFor 等危险方法在无 target 时禁止
    assert!(
        ExpressionUtils::is_member_forbidden(None, "loadClass"),
        "loadClass 应被禁止"
    );
    assert!(
        ExpressionUtils::is_member_forbidden(None, "exec"),
        "exec 应被禁止"
    );
}

// ===========================================================================
// contains_external_access：表达式外部访问语法检测
// ===========================================================================

#[test]
fn external_access_syntax_is_detected() {
    // new / param / @Type@ 等外部访问语法在表达式中被检测到
    assert!(StandardExpressionUtils::contains_external_access(
        "${new java.lang.ProcessBuilder('cmd')}"
    ));
    assert!(StandardExpressionUtils::contains_external_access(
        "${param.cmd}"
    ));
    assert!(StandardExpressionUtils::contains_external_access(
        "${@java.lang.Runtime@exec('cmd')}"
    ));
}

#[test]
fn normal_expressions_have_no_external_access() {
    // 普通变量引用 / 方法调用 / 内建不含外部访问
    assert!(!StandardExpressionUtils::contains_external_access(
        "${user.name}"
    ));
    assert!(!StandardExpressionUtils::contains_external_access(
        "${#dates.format(now, 'yyyy')}"
    ));
    assert!(!StandardExpressionUtils::contains_external_access(
        "${1 + 2}"
    ));
    assert!(!StandardExpressionUtils::contains_external_access(""));
}
