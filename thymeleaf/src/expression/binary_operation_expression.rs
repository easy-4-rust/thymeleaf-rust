#![expect(
    clippy::type_complexity,
    reason = "二元结果显式保留左右 Java null 与对象共享身份"
)]

use std::cmp::Ordering;
use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::{BigDecimalValue, EvaluationUtils, Utf16String, ValidateError};

use super::{
    IStandardExpression, StandardExpressionExecutionContext, StandardExpressionResult,
    TemplateValue,
};

/// Standard Expression 二元运算的共享不可变状态。
///
/// 对应 Java: `org.thymeleaf.standard.expression.BinaryOperationExpression`。
///
/// Java 抽象类通过继承保存左右操作数；Rust 具体二元表达式组合本对象，并把解析组合
/// 算法放入后续 `ExpressionParsingUtil`，避免用反射构造具体类型。
pub struct BinaryOperationExpression {
    left: Arc<dyn IStandardExpression>,
    right: Arc<dyn IStandardExpression>,
}

/// 对应 Java 语义：`BinaryOperationExpression` 的 `execute_operands` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn execute_operands(
    expression: &BinaryOperationExpression,
    context: &dyn IExpressionContext,
    execution_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<(Option<Arc<TemplateValue>>, Option<Arc<TemplateValue>>)> {
    Ok((
        expression
            .left_arc()
            .execute_with_context(context, execution_context)?,
        expression
            .right_arc()
            .execute_with_context(context, execution_context)?,
    ))
}

/// 对应 Java 语义：`BinaryOperationExpression` 的 `execute_raw_operands` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn execute_raw_operands(
    expression: &BinaryOperationExpression,
    context: &dyn IExpressionContext,
    execution_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<(Option<Arc<TemplateValue>>, Option<Arc<TemplateValue>>)> {
    Ok((
        expression
            .left_arc()
            .execute_raw(context, execution_context)?,
        expression
            .right_arc()
            .execute_raw(context, execution_context)?,
    ))
}

/// 对应 Java 语义：Java 接口/超类方法 `evaluateAsNumber()` 的 Rust 移植（`BinaryOperationExpression` 继承路径）。
pub(crate) fn evaluate_as_number(
    value: Option<&Arc<TemplateValue>>,
) -> StandardExpressionResult<Option<BigDecimalValue>> {
    let Some(value) = value else {
        return Ok(None);
    };
    Ok(
        EvaluationUtils::evaluate_as_number(&value.to_evaluation_value())?
            .map(|result| result.as_decimal().clone()),
    )
}

/// 对应 Java 语义：Java 接口/超类方法 `evaluateAsBoolean()` 的 Rust 移植（`BinaryOperationExpression` 继承路径）。
pub(crate) fn evaluate_as_boolean(
    value: Option<&Arc<TemplateValue>>,
) -> StandardExpressionResult<bool> {
    let evaluation_value = value
        .map(|value| value.to_evaluation_value())
        .unwrap_or(crate::util::EvaluationValue::Null);
    Ok(EvaluationUtils::evaluate_as_boolean(&evaluation_value)?)
}

/// 对应 Java 语义：`BinaryOperationExpression` 的 `normalized_null_value` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn normalized_null_value(value: Option<Arc<TemplateValue>>) -> Arc<TemplateValue> {
    value.unwrap_or_else(|| Arc::new(TemplateValue::string(Utf16String::from_rust_str("null"))))
}

/// 对应 Java 语义：`BinaryOperationExpression` 的 `literal_unwrapped_string` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn literal_unwrapped_string(value: &TemplateValue) -> Option<Utf16String> {
    match value {
        TemplateValue::Literal(literal) => literal.get_value().cloned(),
        _ => value.to_utf16_string(),
    }
}

/// 对应 Java 语义：`BinaryOperationExpression` 的 `unwrap_literal_result` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn unwrap_literal_result(
    value: Option<Arc<TemplateValue>>,
) -> Option<Arc<TemplateValue>> {
    match value.as_deref() {
        Some(TemplateValue::Literal(literal)) => literal
            .get_value()
            .cloned()
            .map(TemplateValue::string)
            .map(Arc::new),
        _ => value,
    }
}

/// 对应 Java 语义：`BinaryOperationExpression` 的 `collapse_java_null` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn collapse_java_null(value: Option<Arc<TemplateValue>>) -> Option<Arc<TemplateValue>> {
    match value.as_deref() {
        Some(TemplateValue::Null) => None,
        _ => value,
    }
}

/// 对应 Java 语义：`BinaryOperationExpression` 的 `java_values_equal` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn java_values_equal(
    left: Option<&Arc<TemplateValue>>,
    right: Option<&Arc<TemplateValue>>,
) -> StandardExpressionResult<bool> {
    let Some(left) = left else {
        return Ok(right.is_none());
    };
    let Some(right) = right else {
        return Ok(false);
    };
    if let (Some(left_number), Some(right_number)) = (
        evaluate_as_number(Some(left))?,
        evaluate_as_number(Some(right))?,
    ) {
        return Ok(left_number.compare_java(&right_number) == Ordering::Equal);
    }
    let left = character_as_string(left);
    let right = character_as_string(right);
    if left.java_class_name() == right.java_class_name()
        && let Some(comparison) = left.java_compare_to(right.as_ref())
    {
        return Ok(comparison? == Ordering::Equal);
    }
    Ok(left.java_equals(right.as_ref()))
}

/// 对应 Java 语义：`BinaryOperationExpression` 的 `compare_java_values` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn compare_java_values(
    left: &Arc<TemplateValue>,
    right: &Arc<TemplateValue>,
) -> StandardExpressionResult<Option<Ordering>> {
    if let (Some(left_number), Some(right_number)) = (
        evaluate_as_number(Some(left))?,
        evaluate_as_number(Some(right))?,
    ) {
        return Ok(Some(left_number.compare_java(&right_number)));
    }
    if left.java_class_name() != right.java_class_name() {
        return Ok(None);
    }
    match left.java_compare_to(right.as_ref()) {
        Some(comparison) => Ok(Some(comparison?)),
        None => Ok(None),
    }
}

fn character_as_string(value: &Arc<TemplateValue>) -> Arc<TemplateValue> {
    match value.as_ref() {
        TemplateValue::Character(unit) => {
            Arc::new(TemplateValue::string(Utf16String::from_utf16(vec![*unit])))
        }
        _ => Arc::clone(value),
    }
}

impl BinaryOperationExpression {
    /// 创建二元运算状态，并按 Java 顺序校验左右操作数。
    /// 对应 Java 语义：`BinaryOperationExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        left: Option<Arc<dyn IStandardExpression>>,
        right: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<Self, ValidateError> {
        let left = left.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Left-side expression cannot be null".to_owned()),
        })?;
        let right = right.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Right-side expression cannot be null".to_owned()),
        })?;
        Ok(Self { left, right })
    }

    /// 返回左操作数的同一动态对象。
    /// 对应 Java: `BinaryOperationExpression#getLeft()`。
    pub fn get_left(&self) -> &dyn IStandardExpression {
        self.left.as_ref()
    }

    /// 返回左操作数共享引用，供具体运算执行。
    /// 对应 Java 语义：`BinaryOperationExpression` 的 `left_arc` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn left_arc(&self) -> &Arc<dyn IStandardExpression> {
        &self.left
    }

    /// 返回右操作数的同一动态对象。
    /// 对应 Java: `BinaryOperationExpression#getRight()`。
    pub fn get_right(&self) -> &dyn IStandardExpression {
        self.right.as_ref()
    }

    /// 返回右操作数共享引用，供具体运算执行。
    /// 对应 Java 语义：`BinaryOperationExpression` 的 `right_arc` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn right_arc(&self) -> &Arc<dyn IStandardExpression> {
        &self.right
    }

    /// 返回 `left operator right`，复杂子表达式自动添加圆括号。
    /// 对应 Java: `BinaryOperationExpression#getStringRepresentation()`。
    pub fn get_string_representation(
        &self,
        operator: Option<&Utf16String>,
    ) -> StandardExpressionResult<Utf16String> {
        let mut units = Vec::new();
        append_operand(&mut units, self.left.as_ref())?;
        units.push(b' ' as u16);
        match operator {
            Some(operator) => units.extend_from_slice(operator.as_utf16()),
            None => units.extend("null".encode_utf16()),
        }
        units.push(b' ' as u16);
        append_operand(&mut units, self.right.as_ref())?;
        Ok(Utf16String::from_utf16(units))
    }
}

fn append_operand(
    target: &mut Vec<u16>,
    expression: &dyn IStandardExpression,
) -> StandardExpressionResult<()> {
    if expression.is_complex() {
        target.push(b'(' as u16);
    }
    target.extend_from_slice(expression.get_string_representation()?.as_utf16());
    if expression.is_complex() {
        target.push(b')' as u16);
    }
    Ok(())
}
