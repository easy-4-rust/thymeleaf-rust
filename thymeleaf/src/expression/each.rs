use std::sync::Arc;

use crate::util::{JavaString, ValidateError};

use super::{IStandardExpression, StandardExpressionResult};

/// `th:each` 解析后的迭代声明。
///
/// 对应 Java: `org.thymeleaf.standard.expression.Each`。
pub struct Each {
    iter_var: Arc<dyn IStandardExpression>,
    status_var: Option<Arc<dyn IStandardExpression>>,
    iterable: Arc<dyn IStandardExpression>,
}

impl Each {
    /// 创建迭代声明，并按 Java 顺序校验迭代变量和 iterable。
    pub fn new(
        iter_var: Option<Arc<dyn IStandardExpression>>,
        status_var: Option<Arc<dyn IStandardExpression>>,
        iterable: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<Self, ValidateError> {
        let iter_var = iter_var.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Iteration variable cannot be null".to_owned()),
        })?;
        let iterable = iterable.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Iterable cannot be null".to_owned()),
        })?;
        Ok(Self {
            iter_var,
            status_var,
            iterable,
        })
    }
    /// 返回迭代变量表达式。
    pub fn get_iter_var(&self) -> &dyn IStandardExpression {
        self.iter_var.as_ref()
    }
    /// 判断是否声明状态变量。
    pub fn has_status_var(&self) -> bool {
        self.status_var.is_some()
    }
    /// 返回可空状态变量表达式。
    pub fn get_status_var(&self) -> Option<&dyn IStandardExpression> {
        self.status_var.as_deref()
    }
    /// 返回 iterable 表达式。
    pub fn get_iterable(&self) -> &dyn IStandardExpression {
        self.iterable.as_ref()
    }
    /// 返回 `iter[,status] : iterable` 规范文本。
    pub fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        let mut units = self
            .iter_var
            .get_string_representation()?
            .as_utf16()
            .to_vec();
        if let Some(status_var) = &self.status_var {
            units.push(b',' as u16);
            units.extend_from_slice(status_var.get_string_representation()?.as_utf16());
        }
        units.extend_from_slice(&[b' ' as u16, b':' as u16, b' ' as u16]);
        units.extend_from_slice(self.iterable.get_string_representation()?.as_utf16());
        Ok(JavaString::from_utf16(units))
    }
}
