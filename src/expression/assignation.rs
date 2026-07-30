use std::sync::Arc;

use crate::util::{JavaString, ValidateError};

use super::{IStandardExpression, StandardExpressionResult};

/// Standard Expression 中的单个赋值对。
///
/// 对应 Java: `org.thymeleaf.standard.expression.Assignation`。
pub struct Assignation {
    left: Arc<dyn IStandardExpression>,
    right: Option<Arc<dyn IStandardExpression>>,
}

impl Assignation {
    /// 创建赋值；左侧为 null 时复现 Java 参数校验错误，右侧允许缺失。
    pub(crate) fn new(
        left: Option<Arc<dyn IStandardExpression>>,
        right: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<Self, ValidateError> {
        let left = left.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Assignation left side cannot be null".to_owned()),
        })?;
        Ok(Self { left, right })
    }
    /// 返回左侧表达式。
    pub fn get_left(&self) -> &dyn IStandardExpression {
        self.left.as_ref()
    }
    /// 返回可空右侧表达式。
    pub fn get_right(&self) -> Option<&dyn IStandardExpression> {
        self.right.as_deref()
    }
    /// 返回规范字符串表示。
    pub fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        let mut units = self.left.get_string_representation()?.as_utf16().to_vec();
        if let Some(right) = &self.right {
            units.push(b'=' as u16);
            if right.is_complex() {
                units.push(b'(' as u16);
            }
            units.extend_from_slice(right.get_string_representation()?.as_utf16());
            if right.is_complex() {
                units.push(b')' as u16);
            }
        }
        Ok(JavaString::from_utf16(units))
    }
}
