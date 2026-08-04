//! sec 方言属性处理器 —— `sec:authorize` 与 `sec:authentication`。
//!
//! 对应 Java `thymeleaf-extras-springsecurity6` 的 `SecAuthorizeAttrProcessor` 与
//! `SecAuthenticationAttrProcessor`：
//! - `sec:authorize`：条件可见性。属性值为布尔表达式（如 `${hasRole('ADMIN')}`、
//!   `${isAuthenticated()}`、`${#authorization.hasPermission('orders:write')}`），
//!   求值为 `false` 时移除元素；表达式对象 `#authentication`/`#authorization`
//!   由本方言的 `IExpressionObjectDialect` 提供。
//! - `sec:authentication`：输出当前认证身份的属性（如 `name`），对齐 Java
//!   `sec:authentication="name"` 语义（Java 版本在此处只接受字面属性名）。

use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{IElementProcessor, IElementTagProcessor, IElementTagStructureHandler};
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::expression::{StandardExpressions, TemplateValue};
use thymeleaf::model::IProcessableElementTag;
use thymeleaf::processor::{
    AbstractStandardConditionalVisibilityTagProcessor,
    AbstractStandardExpressionAttributeTagProcessor, IProcessor,
};
use thymeleaf::util::{EvaluationUtils, EvaluationValue, Utf16String};

use crate::expression_object::read_authentication;

/// `sec:authorize` 属性名。
pub const AUTHORIZE_ATTR_NAME: &str = "authorize";
/// `sec:authentication` 属性名。
pub const AUTHENTICATION_ATTR_NAME: &str = "authentication";

/// `sec:authorize` 处理器 precedence（Java `SecAuthorizeAttrProcessor`：500）。
pub const AUTHORIZE_PRECEDENCE: i32 = 500;
/// `sec:authentication` 处理器 precedence（Java `SecAuthenticationAttrProcessor`：510）。
pub const AUTHENTICATION_PRECEDENCE: i32 = 510;

/// `sec:authorize` 条件可见性处理器。
pub struct SecAuthorizeTagProcessor {
    processor: AbstractStandardConditionalVisibilityTagProcessor,
}

impl SecAuthorizeTagProcessor {
    /// 创建指定模板模式和方言前缀的 `sec:authorize` 处理器。
    ///
    /// # Errors
    ///
    /// 处理器配置非法时返回模板处理异常。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardConditionalVisibilityTagProcessor::new(
                template_mode,
                dialect_prefix,
                Utf16String::from_rust_str(AUTHORIZE_ATTR_NAME),
                AUTHORIZE_PRECEDENCE,
                |context, _tag, _attribute_name, attribute_value| {
                    evaluate_authorize_expression(context, attribute_value)
                },
                "org.thymeleaf.extras.springsecurity6.processor.SecAuthorizeAttrProcessor",
            )?,
        })
    }
}

/// `sec:authentication` 属性处理器。
pub struct SecAuthenticationTagProcessor {
    processor: AbstractStandardExpressionAttributeTagProcessor,
}

impl SecAuthenticationTagProcessor {
    /// 创建指定模板模式和方言前缀的 `sec:authentication` 处理器。
    ///
    /// # Errors
    ///
    /// 处理器配置非法时返回模板处理异常。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardExpressionAttributeTagProcessor::new(
                template_mode,
                dialect_prefix,
                Utf16String::from_rust_str(AUTHENTICATION_ATTR_NAME),
                AUTHENTICATION_PRECEDENCE,
                true,
                thymeleaf::expression::StandardExpressionExecutionContext::NORMAL,
                |context,
                 _tag,
                 _attribute_name,
                 attribute_value,
                 _expression_result,
                 structure_handler| {
                    // Java 只接受字面属性名（"name"）；若值是标准表达式则求值字符串
                    let property = attribute_value
                        .map_or_else(|| Utf16String::from_rust_str("name"), Utf16String::clone);
                    let value = read_authentication(context);
                    let text = match property.to_string_lossy().as_str() {
                        "name" | "loginId" | "login_id" => value.login_id().map_or_else(
                            || Utf16String::from_rust_str(""),
                            Utf16String::from_rust_str,
                        ),
                        "roles" => Utf16String::from_rust_str(&value.roles().join(",")),
                        _ => {
                            return Err(Box::new(TemplateProcessingException::new(Some(format!(
                                "Unknown authentication property: {}",
                                property.to_string_lossy()
                            )))));
                        }
                    };
                    structure_handler.set_body_sequence(Arc::new(text), true);
                    Ok(())
                },
                "org.thymeleaf.extras.springsecurity6.processor.SecAuthenticationAttrProcessor",
            )?,
        })
    }
}

/// 把 Standard Expression 错误保留为模板处理异常的 cause。
fn expression_processing_error(
    message: &'static str,
    error: thymeleaf::expression::StandardExpressionError,
) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some(message.to_owned()),
        StandardExpressionCause(error),
    ))
}

struct StandardExpressionCause(Box<dyn std::error::Error + Send + Sync>);

impl std::fmt::Display for StandardExpressionCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}

impl std::fmt::Debug for StandardExpressionCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("StandardExpressionCause")
            .field(&self.0.to_string())
            .finish()
    }
}

impl std::error::Error for StandardExpressionCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

/// 解析并执行属性表达式，按 Thymeleaf 真值规则求布尔值。
///
/// 对应 `standard_processor_utils::evaluate_standard_expression_as_boolean`（该函数
/// 为 crate 内部，这里在外部 integration crate 复刻同一逻辑）。
fn evaluate_expression_as_boolean(
    context: &dyn ITemplateContext,
    input: Option<&Utf16String>,
) -> Result<bool, Box<dyn TemplateEngineException>> {
    let parser = StandardExpressions::get_expression_parser(context.get_configuration()).map_err(
        |error| expression_processing_error("Could not obtain Standard Expression parser", error),
    )?;
    let expression = parser.parse_expression(context, input).map_err(|error| {
        expression_processing_error("Could not parse Standard Expression", error)
    })?;
    let result = expression.execute(context).map_err(|error| {
        expression_processing_error("Could not execute Standard Expression", error)
    })?;
    let evaluation_value = result
        .as_deref()
        .map_or(EvaluationValue::Null, TemplateValue::to_evaluation_value);
    EvaluationUtils::evaluate_as_boolean(&evaluation_value).map_err(|error| {
        Box::new(TemplateProcessingException::with_cause(
            Some("Could not evaluate Standard Expression as boolean".to_owned()),
            error,
        )) as Box<dyn TemplateEngineException>
    })
}

/// 求值 `sec:authorize` 属性值。
///
/// 属性值可以是：
/// - 裸 Spring Security 授权表达式（`hasRole('ADMIN')`、`isAuthenticated()`、
///   `permitAll`、`denyAll` 等）—— 对齐 Java `SecAuthorizeAttrProcessor` 的
///   Spring Security 授权表达式语义；
/// - 标准表达式（`${#authorization.hasPermission('orders:write')}` 等）——
///   走 `evaluate_expression_as_boolean`。
///
/// 裸表达式仅在属性值不以 `${` 开头时尝试解析；否则交给标准表达式求值。
fn evaluate_authorize_expression(
    context: &dyn ITemplateContext,
    input: Option<&Utf16String>,
) -> Result<bool, Box<dyn TemplateEngineException>> {
    let Some(input) = input else {
        return Ok(false);
    };
    let text = input.to_string_lossy();
    if text.trim_start().starts_with("${") {
        return evaluate_expression_as_boolean(context, Some(input));
    }
    Ok(evaluate_authorization_expression(&text, context))
}

/// 解析并求值裸 Spring Security 授权表达式。
///
/// 支持子集（对齐 `thymeleaf-extras-springsecurity6` 常见用法，基于当前安全快照）：
/// - `permitAll` / `permit_all` / `permitAll()` → `true`
/// - `denyAll` / `deny_all` / `denyAll()` → `false`
/// - `isAuthenticated()` / `is_authenticated()` / `isFullyAuthenticated()` → 已认证
/// - `isAnonymous()` → 未认证
/// - `hasRole('X')`、`hasAnyRole('A','B')`、`hasAllRoles('A','B')`
/// - `hasAuthority('X')`（= hasRole 语义）、`hasPermission('X')`、
///   `hasAnyPermission('A','B')`、`hasAllPermissions('A','B')`
///
/// 未知表达式与解析失败按 fail-closed 返回 `false`（与 sa-token 匿名拒绝一致）。
fn evaluate_authorization_expression(expression: &str, context: &dyn ITemplateContext) -> bool {
    let authentication = read_authentication(context);
    let expression = expression.trim();
    let without_parentheses = strip_call_parens(expression);

    match without_parentheses {
        Some(("permitAll" | "permit_all", "")) => true,
        Some(("denyAll" | "deny_all", "")) => false,
        Some(("isAuthenticated" | "is_authenticated" | "isFullyAuthenticated", "")) => {
            authentication.is_authenticated()
        }
        Some(("isAnonymous", "")) => !authentication.is_authenticated(),
        Some(("hasRole" | "hasAuthority" | "has_role", args)) => {
            single_string_arg(args).is_some_and(|role| authentication.has_role(&role))
        }
        Some(("hasAnyRole" | "has_any_role", args)) => string_args(args)
            .is_some_and(|roles| authentication.has_any_role(&string_slice(&roles))),
        Some(("hasAllRoles" | "has_all_roles", args)) => string_args(args)
            .is_some_and(|roles| authentication.has_all_roles(&string_slice(&roles))),
        Some(("hasPermission" | "has_permission", args)) => single_string_arg(args)
            .is_some_and(|permission| authentication.has_permission(&permission)),
        Some(("hasAnyPermission" | "has_any_permission", args)) => {
            string_args(args).is_some_and(|permissions| {
                authentication.has_any_permission(&string_slice(&permissions))
            })
        }
        Some(("hasAllPermissions" | "has_all_permissions", args)) => {
            string_args(args).is_some_and(|permissions| {
                authentication.has_all_permissions(&string_slice(&permissions))
            })
        }
        _ => false,
    }
}

/// 若表达式形如 `name(...)`，返回 `(name, 括号内参数串)`。
fn strip_call_parens(expression: &str) -> Option<(&str, &str)> {
    let open = expression.find('(')?;
    if !expression.ends_with(')') {
        return None;
    }
    let name = expression[..open].trim();
    let args = &expression[open + 1..expression.len() - 1];
    Some((name, args))
}

/// 读取单字符串参数；带引号剥除，多个参数返回 `None`。
fn single_string_arg(args: &str) -> Option<String> {
    let mut values = split_args(args);
    if values.len() != 1 {
        return None;
    }
    Some(values.remove(0))
}

/// 读取逗号分隔的字符串参数列表；每个参数去引号。
fn string_args(args: &str) -> Option<Vec<String>> {
    let values = split_args(args);
    if values.is_empty() {
        return None;
    }
    Some(values)
}

/// 按逗号切分参数并去除单/双引号。
fn split_args(args: &str) -> Vec<String> {
    args.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            let part = part.trim();
            if part.len() >= 2
                && ((part.starts_with('\'') && part.ends_with('\''))
                    || (part.starts_with('"') && part.ends_with('"')))
            {
                part[1..part.len() - 1].to_owned()
            } else {
                part.to_owned()
            }
        })
        .collect()
}

/// 把字符串列表转换为临时 `&[&str]` 切片（供 `has_any_*`/`has_all_*` 使用）。
fn string_slice(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

impl IProcessor for SecAuthorizeTagProcessor {
    fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
        Some(self)
    }
    fn java_class_name(&self) -> &'static str {
        IProcessor::java_class_name(&self.processor)
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        IProcessor::get_template_mode(&self.processor)
    }
    fn get_precedence(&self) -> i32 {
        IProcessor::get_precedence(&self.processor)
    }
}

impl IElementProcessor for SecAuthorizeTagProcessor {
    fn as_element_tag_processor(&self) -> Option<&dyn IElementTagProcessor> {
        IElementProcessor::as_element_tag_processor(&self.processor)
    }
    fn get_matching_element_name(&self) -> Option<&thymeleaf::element::MatchingElementName> {
        IElementProcessor::get_matching_element_name(&self.processor)
    }
    fn get_matching_attribute_name(&self) -> Option<&thymeleaf::element::MatchingAttributeName> {
        IElementProcessor::get_matching_attribute_name(&self.processor)
    }
}

impl IElementTagProcessor for SecAuthorizeTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}

impl IProcessor for SecAuthenticationTagProcessor {
    fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
        Some(self)
    }
    fn java_class_name(&self) -> &'static str {
        IProcessor::java_class_name(&self.processor)
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        IProcessor::get_template_mode(&self.processor)
    }
    fn get_precedence(&self) -> i32 {
        IProcessor::get_precedence(&self.processor)
    }
}

impl IElementProcessor for SecAuthenticationTagProcessor {
    fn as_element_tag_processor(&self) -> Option<&dyn IElementTagProcessor> {
        IElementProcessor::as_element_tag_processor(&self.processor)
    }
    fn get_matching_element_name(&self) -> Option<&thymeleaf::element::MatchingElementName> {
        IElementProcessor::get_matching_element_name(&self.processor)
    }
    fn get_matching_attribute_name(&self) -> Option<&thymeleaf::element::MatchingAttributeName> {
        IElementProcessor::get_matching_attribute_name(&self.processor)
    }
}

impl IElementTagProcessor for SecAuthenticationTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}
