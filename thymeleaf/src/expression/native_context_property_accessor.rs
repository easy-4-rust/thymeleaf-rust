use std::sync::Arc;

use thiserror::Error;

use crate::context::IContext;
use crate::util::Utf16String;

use super::TemplateValue;

/// 允许 OGNL 按 Map 属性语义读取 `IContext` 变量。
///
/// 对应 Java: `org.thymeleaf.standard.expression.OGNLContextPropertyAccessor`。
pub struct NativeContextPropertyAccessor;

impl NativeContextPropertyAccessor {
    /// OGNL 上下文中启用受限表达式对象访问的标记键。
    pub const RESTRICT_EXPRESSION_OBJECTS: &'static str = "%RESTRICT_EXPRESSION_OBJECTS%";
    /// 受限执行上下文禁止直接读取的请求参数变量名。
    pub const REQUEST_PARAMETERS_RESTRICTED_VARIABLE_NAME: &'static str = "param";

    /// 创建无状态 Context 属性访问器。
    pub const fn new() -> Self {
        Self
    }

    /// 读取 Context 变量并执行 `param` 访问限制。
    /// 对应 Java: `OGNLContextPropertyAccessor#getProperty()`。
    pub fn get_property(
        &self,
        restrict_expression_objects: bool,
        target: &dyn IContext,
        name: Option<&Utf16String>,
    ) -> Result<Option<Arc<TemplateValue>>, NativeContextPropertyError> {
        if restrict_expression_objects
            && name.is_some_and(|name| {
                name == &Utf16String::from_rust_str(
                    Self::REQUEST_PARAMETERS_RESTRICTED_VARIABLE_NAME,
                )
            })
        {
            return Err(NativeContextPropertyError::RestrictedVariable {
                name: Self::REQUEST_PARAMETERS_RESTRICTED_VARIABLE_NAME.to_owned(),
            });
        }
        Ok(target.get_variable(name))
    }

    /// Context 在 OGNL 中只读，写操作始终失败。
    /// 对应 Java: `OGNLContextPropertyAccessor#setProperty()`。
    pub fn set_property(
        &self,
        _target: &dyn IContext,
        _name: Option<&Utf16String>,
        _value: Option<Arc<TemplateValue>>,
    ) -> Result<(), NativeContextPropertyError> {
        Err(NativeContextPropertyError::ReadOnly)
    }
}

impl Default for NativeContextPropertyAccessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Context 属性访问失败类别。
#[derive(Debug, Error, Eq, PartialEq)]
/// 对应 Java 语义：`OGNLContextPropertyAccessor` 的 Rust 侧类型 `NativeContextPropertyError`。
pub enum NativeContextPropertyError {
    /// 受限上下文禁止访问 `param`。
    #[error(
        "Access to variable \"{name}\" is forbidden in this context. Note some restrictions apply to variable access."
    )]
    RestrictedVariable {
        /// 被拒绝的名称。
        name: String,
    },
    /// OGNL 不能修改 Context 变量。
    #[error("Cannot set values into VariablesMap instances from OGNL Expressions")]
    ReadOnly,
}
