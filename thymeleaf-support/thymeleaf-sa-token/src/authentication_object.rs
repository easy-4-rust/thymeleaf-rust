//! 把 [`SaTokenAuthentication`] 接入 Thymeleaf 动态值模型的 `TemplateObject` 适配器。
//!
//! 该对象支持模板表达式访问：
//! - `${#authentication.name}` / `${#authentication.loginId}`
//! - `${#authentication.roles}`（字符串序列）
//! - `${#authentication.permissions}`（字符串序列）
//! - `${#authentication.authenticated}`（布尔）
//! - `${#authentication.hasRole('ADMIN')}` / `${#authentication.hasAnyRole('A','B')}`
//! - `${#authentication.hasPermission('orders:write')}` 等
//!
//! 属性与方法的精确集合见 [`SaTokenAuthenticationObject::PROPERTY_NAME`] 注释，
//! 对齐 Java `thymeleaf-extras-springsecurity` 的 `Authentication` 模板可见子集
//! （`name`/`principal`/`authorities`；sa-token 无 `credentials`/`details` 等价物，
//! 故不提供，见 crate 文档的 NA 登记）。

use std::any::Any;
use std::cmp::Ordering;
use std::sync::Arc;

use thymeleaf::TemplateValue;
use thymeleaf::expression::{
    TemplateObject, TemplateObjectMethodError, TemplateObjectPropertyError,
};
use thymeleaf::util::Utf16String;

use crate::authentication::SaTokenAuthentication;

/// `TemplateObject` 适配器 —— 共享同一 `SaTokenAuthentication` 引用身份。
#[derive(Clone, Debug)]
pub struct SaTokenAuthenticationObject {
    authentication: Arc<SaTokenAuthentication>,
}

impl SaTokenAuthenticationObject {
    /// 包装安全快照。
    #[must_use]
    pub const fn new(authentication: Arc<SaTokenAuthentication>) -> Self {
        Self { authentication }
    }

    /// 返回被包装的快照引用。
    #[must_use]
    pub const fn authentication(&self) -> &Arc<SaTokenAuthentication> {
        &self.authentication
    }

    /// 把快照包装为模板上下文可直接注入的 `TemplateValue`。
    #[must_use]
    pub fn to_template_value(authentication: Arc<SaTokenAuthentication>) -> TemplateValue {
        TemplateValue::Object(Arc::new(Self::new(authentication)))
    }
}

impl TemplateObject for SaTokenAuthenticationObject {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.extras.springsecurity6.auth.Authorization"
    }

    fn to_utf16_string(&self) -> Utf16String {
        self.authentication.login_id().map_or_else(
            || Utf16String::from_rust_str("anonymous"),
            Utf16String::from_rust_str,
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_equals(&self, other: &dyn TemplateObject) -> bool {
        other
            .as_any()
            .downcast_ref::<SaTokenAuthenticationObject>()
            .is_some_and(|other| other.authentication == self.authentication)
    }

    fn java_compare_to(
        &self,
        _other: &dyn TemplateObject,
    ) -> Option<Result<Ordering, thymeleaf::expression::TemplateObjectComparisonError>> {
        None
    }

    fn java_iterable_values(&self) -> Option<Vec<Arc<TemplateValue>>> {
        None
    }

    fn java_serializable_properties(
        &self,
    ) -> Option<Vec<(Utf16String, Option<Arc<TemplateValue>>)>> {
        Some(vec![
            (
                Utf16String::from_rust_str("name"),
                Some(Arc::new(TemplateValue::string(self.to_utf16_string()))),
            ),
            (
                Utf16String::from_rust_str("roles"),
                Some(Arc::new(sequence_value(self.authentication.roles().iter()))),
            ),
            (
                Utf16String::from_rust_str("permissions"),
                Some(Arc::new(sequence_value(
                    self.authentication.permissions().iter(),
                ))),
            ),
        ])
    }

    fn java_get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        let value = match property_name.to_string_lossy().as_str() {
            "name" | "loginId" | "login_id" => {
                Some(Arc::new(TemplateValue::string(self.to_utf16_string())))
            }
            "roles" => Some(Arc::new(sequence_value(self.authentication.roles().iter()))),
            "permissions" => Some(Arc::new(sequence_value(
                self.authentication.permissions().iter(),
            ))),
            "authenticated" | "isAuthenticated" => Some(Arc::new(TemplateValue::Boolean(
                self.authentication.is_authenticated(),
            ))),
            _ => return None,
        };
        Some(Ok(value))
    }

    fn java_invoke_method(
        &self,
        method_name: &Utf16String,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        let result = match method_name.to_string_lossy().as_str() {
            "hasRole" | "has_role" => {
                let role = string_argument(arguments, 0)?;
                self.authentication.has_role(&role)
            }
            "hasAnyRole" | "has_any_role" => self
                .authentication
                .has_any_role(&string_slice(&string_arguments(arguments)?)),
            "hasAllRoles" | "has_all_roles" => self
                .authentication
                .has_all_roles(&string_slice(&string_arguments(arguments)?)),
            "hasPermission" | "has_permission" => {
                let permission = string_argument(arguments, 0)?;
                self.authentication.has_permission(&permission)
            }
            "hasAnyPermission" | "has_any_permission" => self
                .authentication
                .has_any_permission(&string_slice(&string_arguments(arguments)?)),
            "hasAllPermissions" | "has_all_permissions" => self
                .authentication
                .has_all_permissions(&string_slice(&string_arguments(arguments)?)),
            "isAuthenticated" | "is_authenticated" => self.authentication.is_authenticated(),
            _ => return None,
        };
        Some(Ok(Some(Arc::new(TemplateValue::Boolean(result)))))
    }
}

/// 把字符串列表转换为临时 `&[&str]` 切片（供 `has_any_*`/`has_all_*` 使用）。
fn string_slice(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

/// 把字符串迭代器构建为 `TemplateValue::List`。
fn sequence_value<'a, I>(items: I) -> TemplateValue
where
    I: IntoIterator<Item = &'a Arc<str>>,
{
    TemplateValue::List(Arc::new(
        items
            .into_iter()
            .map(|item| Arc::new(TemplateValue::string(Utf16String::from_rust_str(item))))
            .collect(),
    ))
}

/// 读取位置参数并转换为字符串；缺失或非字符串返回 `None`。
fn string_argument(arguments: &[Option<Arc<TemplateValue>>], index: usize) -> Option<String> {
    let value = arguments.get(index)?.as_ref()?;
    match value.as_ref() {
        TemplateValue::String(text) => Some(text.to_string_lossy()),
        _ => None,
    }
}

/// 把全部位置参数读取为字符串；任一缺失/非字符串返回 `None`。
fn string_arguments(arguments: &[Option<Arc<TemplateValue>>]) -> Option<Vec<String>> {
    let mut result = Vec::with_capacity(arguments.len());
    for argument in arguments {
        let value = argument.as_ref()?;
        match value.as_ref() {
            TemplateValue::String(text) => result.push(text.to_string_lossy()),
            _ => return None,
        }
    }
    Some(result)
}
