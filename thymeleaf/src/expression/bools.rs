use indexmap::IndexSet;

use crate::util::{EvaluationError, EvaluationUtils, JavaEvaluationValue, Validate};

/// Thymeleaf 标准表达式中的布尔工具对象。
///
/// 对应 Java: `org.thymeleaf.expression.Bools`。该无状态对象通常以 `#bools`
/// 暴露，所有真值判断均委托 [`EvaluationUtils::evaluate_as_boolean`]。
#[derive(Clone, Copy, Debug, Default)]
pub struct Bools;

impl Bools {
    /// 创建无状态布尔表达式对象。对应 Java: `Bools#Bools()`。
    ///
    /// # 返回
    /// 新的 `#bools` 表达式对象。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 判断目标为真。对应 Java: `Bools#isTrue(Object)`。
    ///
    /// # 参数
    /// - `target`：待求值 Java 对象。
    ///
    /// # 返回
    /// Thymeleaf 真值规则的结果。
    ///
    /// # 错误
    /// `LiteralValue(null)` 返回 Java `NullPointerException` 等价错误。
    pub fn is_true(&self, target: &JavaEvaluationValue) -> Result<bool, EvaluationError> {
        EvaluationUtils::evaluate_as_boolean(target)
    }

    /// 对数组逐项判断为真。对应 Java: `Bools#arrayIsTrue(Object[])`。
    pub fn array_is_true(
        &self,
        target: Option<&[JavaEvaluationValue]>,
    ) -> Result<Vec<bool>, EvaluationError> {
        map_values(validate_target(target)?, |value| self.is_true(value))
    }

    /// 对列表逐项判断为真。对应 Java: `Bools#listIsTrue(List)`。
    pub fn list_is_true(
        &self,
        target: Option<&[JavaEvaluationValue]>,
    ) -> Result<Vec<bool>, EvaluationError> {
        map_values(validate_target(target)?, |value| self.is_true(value))
    }

    /// 对 Set 逐项判断为真并按首次结果顺序去重。
    ///
    /// 对应 Java: `Bools#setIsTrue(Set)`。
    pub fn set_is_true(
        &self,
        target: Option<&[JavaEvaluationValue]>,
    ) -> Result<IndexSet<bool>, EvaluationError> {
        set_values(validate_target(target)?, |value| self.is_true(value))
    }

    /// 判断目标为假。对应 Java: `Bools#isFalse(Object)`。
    pub fn is_false(&self, target: &JavaEvaluationValue) -> Result<bool, EvaluationError> {
        self.is_true(target).map(|value| !value)
    }

    /// 对数组逐项判断为假。对应 Java: `Bools#arrayIsFalse(Object[])`。
    pub fn array_is_false(
        &self,
        target: Option<&[JavaEvaluationValue]>,
    ) -> Result<Vec<bool>, EvaluationError> {
        map_values(validate_target(target)?, |value| self.is_false(value))
    }

    /// 对列表逐项判断为假。对应 Java: `Bools#listIsFalse(List)`。
    pub fn list_is_false(
        &self,
        target: Option<&[JavaEvaluationValue]>,
    ) -> Result<Vec<bool>, EvaluationError> {
        map_values(validate_target(target)?, |value| self.is_false(value))
    }

    /// 对 Set 逐项判断为假并按首次结果顺序去重。
    ///
    /// 对应 Java: `Bools#setIsFalse(Set)`。
    pub fn set_is_false(
        &self,
        target: Option<&[JavaEvaluationValue]>,
    ) -> Result<IndexSet<bool>, EvaluationError> {
        set_values(validate_target(target)?, |value| self.is_false(value))
    }

    /// 对数组执行短路逻辑与。对应 Java: `Bools#arrayAnd(Object[])`。
    pub fn array_and(
        &self,
        target: Option<&[JavaEvaluationValue]>,
    ) -> Result<bool, EvaluationError> {
        and_values(validate_target(target)?)
    }

    /// 对列表执行短路逻辑与。对应 Java: `Bools#listAnd(List)`。
    pub fn list_and(
        &self,
        target: Option<&[JavaEvaluationValue]>,
    ) -> Result<bool, EvaluationError> {
        and_values(validate_target(target)?)
    }

    /// 对 Set 执行短路逻辑与。对应 Java: `Bools#setAnd(Set)`。
    pub fn set_and(&self, target: Option<&[JavaEvaluationValue]>) -> Result<bool, EvaluationError> {
        and_values(validate_target(target)?)
    }

    /// 对数组执行短路逻辑或。对应 Java: `Bools#arrayOr(Object[])`。
    pub fn array_or(
        &self,
        target: Option<&[JavaEvaluationValue]>,
    ) -> Result<bool, EvaluationError> {
        or_values(validate_target(target)?)
    }

    /// 对列表执行短路逻辑或。对应 Java: `Bools#listOr(List)`。
    pub fn list_or(&self, target: Option<&[JavaEvaluationValue]>) -> Result<bool, EvaluationError> {
        or_values(validate_target(target)?)
    }

    /// 对 Set 执行短路逻辑或。对应 Java: `Bools#setOr(Set)`。
    pub fn set_or(&self, target: Option<&[JavaEvaluationValue]>) -> Result<bool, EvaluationError> {
        or_values(validate_target(target)?)
    }
}

fn validate_target(
    target: Option<&[JavaEvaluationValue]>,
) -> Result<&[JavaEvaluationValue], EvaluationError> {
    Validate::not_null(target, Some("Target cannot be null"))?;
    Ok(target.expect("validated non-null target"))
}

fn map_values(
    target: &[JavaEvaluationValue],
    predicate: impl Fn(&JavaEvaluationValue) -> Result<bool, EvaluationError>,
) -> Result<Vec<bool>, EvaluationError> {
    target.iter().map(predicate).collect()
}

fn set_values(
    target: &[JavaEvaluationValue],
    predicate: impl Fn(&JavaEvaluationValue) -> Result<bool, EvaluationError>,
) -> Result<IndexSet<bool>, EvaluationError> {
    target.iter().map(predicate).collect()
}

fn and_values(target: &[JavaEvaluationValue]) -> Result<bool, EvaluationError> {
    for value in target {
        if !EvaluationUtils::evaluate_as_boolean(value)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn or_values(target: &[JavaEvaluationValue]) -> Result<bool, EvaluationError> {
    for value in target {
        if EvaluationUtils::evaluate_as_boolean(value)? {
            return Ok(true);
        }
    }
    Ok(false)
}
