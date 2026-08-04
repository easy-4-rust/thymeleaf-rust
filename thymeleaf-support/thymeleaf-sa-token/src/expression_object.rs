//! `#authentication` / `#authorization` 表达式对象工厂。
//!
//! 对应 Java `thymeleaf-extras-springsecurity6` 的
//! `AuthenticationExpressionObjectFactory`/`AuthorizationExpressionObjectFactory`：
//! 模板内以 `${#authentication.name}`、`${#authorization.hasRole('ADMIN')}` 等
//! 形式访问当前请求的认证/授权状态。
//!
//! 对象从模板上下文变量 [`AUTHENTICATION_VARIABLE`] 读取预取快照；变量缺失或为
//! 匿名快照时，`#authentication` 求值为 null、`#authorization` 的所有授权查询
//! 恒为 `false`（fail-closed，与 sa-token 匿名拒绝语义一致）。

use std::sync::Arc;

use thymeleaf::context::IExpressionContext;
use thymeleaf::expression::{
    ExpressionObjectNames, IExpressionObjectFactory, StandardExpressionResult, TemplateValue,
};
use thymeleaf::util::Utf16String;

use crate::authentication::{AUTHENTICATION_VARIABLE, SaTokenAuthentication};
use crate::authentication_object::SaTokenAuthenticationObject;

/// `#authentication` 表达式对象名。
pub const AUTHENTICATION_OBJECT_NAME: &str = "authentication";
/// `#authorization` 表达式对象名。
pub const AUTHORIZATION_OBJECT_NAME: &str = "authorization";

/// 从模板上下文读取安全快照变量。
///
/// 变量缺失、为 null 或不是安全快照对象时返回匿名快照（不报错）。
#[must_use]
pub fn read_authentication(context: &dyn IExpressionContext) -> Arc<SaTokenAuthentication> {
    let name = Utf16String::from_rust_str(AUTHENTICATION_VARIABLE);
    match context.get_variable(Some(&name)) {
        Some(value) => match value.as_ref() {
            TemplateValue::Object(object) => object
                .as_any()
                .downcast_ref::<SaTokenAuthenticationObject>()
                .map(|object| Arc::clone(object.authentication()))
                .unwrap_or_else(|| Arc::new(SaTokenAuthentication::anonymous())),
            _ => Arc::new(SaTokenAuthentication::anonymous()),
        },
        None => Arc::new(SaTokenAuthentication::anonymous()),
    }
}

/// 构建 `#authentication` 或 `#authorization` 表达式对象。
///
/// `#authentication` 返回安全快照的 `TemplateObject` 包装（匿名时也可安全访问，
/// 授权查询恒为 `false`）；`#authorization` 与 `#authentication` 共享同一快照，
/// 仅语义上强调"仅授权判断"入口，因此返回相同的对象。
fn build_object(
    context: Arc<dyn IExpressionContext>,
    name: Option<&Utf16String>,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let Some(name) = name else {
        return Ok(None);
    };
    if name.to_string_lossy() != AUTHENTICATION_OBJECT_NAME
        && name.to_string_lossy() != AUTHORIZATION_OBJECT_NAME
    {
        return Ok(None);
    }
    let authentication = read_authentication(context.as_ref());
    Ok(Some(Arc::new(
        SaTokenAuthenticationObject::to_template_value(authentication),
    )))
}

/// `#authentication` / `#authorization` 表达式对象工厂。
#[derive(Clone, Copy, Debug, Default)]
pub struct SaTokenExpressionObjectFactory;

impl SaTokenExpressionObjectFactory {
    /// 创建空工厂。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl IExpressionObjectFactory for SaTokenExpressionObjectFactory {
    fn get_all_expression_object_names(&self) -> Option<ExpressionObjectNames> {
        Some(Arc::from(vec![
            Some(Utf16String::from_rust_str(AUTHENTICATION_OBJECT_NAME)),
            Some(Utf16String::from_rust_str(AUTHORIZATION_OBJECT_NAME)),
        ]))
    }

    fn build_object(
        &self,
        context: Arc<dyn IExpressionContext>,
        expression_object_name: Option<&Utf16String>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        build_object(context, expression_object_name)
    }

    fn is_cacheable(&self, expression_object_name: Option<&Utf16String>) -> bool {
        expression_object_name.is_some_and(|name| {
            name.to_string_lossy() == AUTHENTICATION_OBJECT_NAME
                || name.to_string_lossy() == AUTHORIZATION_OBJECT_NAME
        })
    }
}
