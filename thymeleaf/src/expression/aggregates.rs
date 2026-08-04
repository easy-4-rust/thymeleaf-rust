use crate::util::{
    AggregateError, AggregateUtils, BigDecimalValue, NumberIterableValue, NumberValue,
};

/// Thymeleaf `#aggregates` 表达式对象。
///
/// 对应 Java: `org.thymeleaf.expression.Aggregates`。
///
/// 本对象无状态，16 个公开入口逐一委托 [`AggregateUtils`]，保留 Java 重载的
/// 数字类型、null、空目标、异常和 `BigDecimal` scale 语义。
#[derive(Clone, Copy, Debug, Default)]
pub struct Aggregates;

impl Aggregates {
    /// 创建表达式聚合对象。
    ///
    /// 对应 Java: `Aggregates#Aggregates()`。通常由标准表达式对象工厂内部创建。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 求 iterable 数字之和。对应 Java: `Aggregates#sum(Iterable)`。
    pub fn sum_iterable(
        &self,
        target: Option<&dyn NumberIterableValue>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::sum_iterable(target)
    }

    /// 求 `Number[]` 数字之和。对应 Java: `Aggregates#sum(Number[])`。
    pub fn sum_numbers(
        &self,
        target: Option<&[Option<NumberValue>]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::sum_numbers(target)
    }

    /// 求 `byte[]` 数字之和。对应 Java: `Aggregates#sum(byte[])`。
    pub fn sum_bytes(
        &self,
        target: Option<&[i8]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::sum_bytes(target)
    }

    /// 求 `short[]` 数字之和。对应 Java: `Aggregates#sum(short[])`。
    pub fn sum_shorts(
        &self,
        target: Option<&[i16]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::sum_shorts(target)
    }

    /// 求 `int[]` 数字之和。对应 Java: `Aggregates#sum(int[])`。
    pub fn sum_ints(
        &self,
        target: Option<&[i32]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::sum_ints(target)
    }

    /// 求 `long[]` 数字之和。对应 Java: `Aggregates#sum(long[])`。
    pub fn sum_longs(
        &self,
        target: Option<&[i64]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::sum_longs(target)
    }

    /// 求 `float[]` 数字之和。对应 Java: `Aggregates#sum(float[])`。
    pub fn sum_floats(
        &self,
        target: Option<&[f32]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::sum_floats(target)
    }

    /// 求 `double[]` 数字之和。对应 Java: `Aggregates#sum(double[])`。
    pub fn sum_doubles(
        &self,
        target: Option<&[f64]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::sum_doubles(target)
    }

    /// 求 iterable 数字平均值。对应 Java: `Aggregates#avg(Iterable)`。
    pub fn avg_iterable(
        &self,
        target: Option<&dyn NumberIterableValue>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::avg_iterable(target)
    }

    /// 求 `Number[]` 数字平均值。对应 Java: `Aggregates#avg(Number[])`。
    pub fn avg_numbers(
        &self,
        target: Option<&[Option<NumberValue>]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::avg_numbers(target)
    }

    /// 求 `byte[]` 数字平均值。对应 Java: `Aggregates#avg(byte[])`。
    pub fn avg_bytes(
        &self,
        target: Option<&[i8]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::avg_bytes(target)
    }

    /// 求 `short[]` 数字平均值。对应 Java: `Aggregates#avg(short[])`。
    pub fn avg_shorts(
        &self,
        target: Option<&[i16]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::avg_shorts(target)
    }

    /// 求 `int[]` 数字平均值。对应 Java: `Aggregates#avg(int[])`。
    pub fn avg_ints(
        &self,
        target: Option<&[i32]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::avg_ints(target)
    }

    /// 求 `long[]` 数字平均值。对应 Java: `Aggregates#avg(long[])`。
    pub fn avg_longs(
        &self,
        target: Option<&[i64]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::avg_longs(target)
    }

    /// 求 `float[]` 数字平均值。对应 Java: `Aggregates#avg(float[])`。
    pub fn avg_floats(
        &self,
        target: Option<&[f32]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::avg_floats(target)
    }

    /// 求 `double[]` 数字平均值。对应 Java: `Aggregates#avg(double[])`。
    pub fn avg_doubles(
        &self,
        target: Option<&[f64]>,
    ) -> Result<Option<BigDecimalValue>, AggregateError> {
        AggregateUtils::avg_doubles(target)
    }
}

#[cfg(test)]
mod tests {
    use crate::util::{AggregateError, BigDecimalValue, NumberListValue, NumberValue};

    use super::Aggregates;

    #[test]
    fn constructs_and_delegates_every_java_overload() {
        let aggregates = Aggregates::new();
        let default = Aggregates;
        assert_eq!(format!("{aggregates:?}"), format!("{default:?}"));

        let numbers = [Some(NumberValue::Integer(1)), Some(NumberValue::Integer(2))];
        let iterable_numbers = NumberListValue::new(numbers.to_vec());
        assert_eq!(
            aggregates
                .sum_iterable(Some(&iterable_numbers))
                .expect("sum")
                .expect("value")
                .to_string(),
            "3"
        );
        assert_eq!(
            aggregates
                .sum_numbers(Some(&numbers))
                .expect("sum")
                .expect("value")
                .to_string(),
            "3"
        );
        assert_eq!(render(aggregates.sum_bytes(Some(&[1, 2]))), "3");
        assert_eq!(render(aggregates.sum_shorts(Some(&[1, 2]))), "3");
        assert_eq!(render(aggregates.sum_ints(Some(&[1, 2]))), "3");
        assert_eq!(render(aggregates.sum_longs(Some(&[1, 2]))), "3");
        assert_eq!(render(aggregates.sum_floats(Some(&[0.5, 0.25]))), "0.75");
        assert_eq!(render(aggregates.sum_doubles(Some(&[0.5, 0.25]))), "0.75");

        assert_eq!(
            aggregates
                .avg_iterable(Some(&iterable_numbers))
                .expect("avg")
                .expect("value")
                .to_string(),
            "1.5"
        );
        assert_eq!(
            aggregates
                .avg_numbers(Some(&numbers))
                .expect("avg")
                .expect("value")
                .to_string(),
            "1.5"
        );
        assert_eq!(render(aggregates.avg_bytes(Some(&[1, 2]))), "1.5");
        assert_eq!(render(aggregates.avg_shorts(Some(&[1, 2]))), "1.5");
        assert_eq!(render(aggregates.avg_ints(Some(&[1, 2]))), "1.5");
        assert_eq!(render(aggregates.avg_longs(Some(&[1, 2]))), "1.5");
        assert_eq!(render(aggregates.avg_floats(Some(&[0.5, 0.25]))), "0.375");
        assert_eq!(render(aggregates.avg_doubles(Some(&[0.5, 0.25]))), "0.375");
    }

    #[test]
    fn facade_preserves_errors_and_empty_null_result() {
        let aggregates = Aggregates::new();
        assert_eq!(
            aggregates.sum_ints(None),
            Err(AggregateError::IllegalArgument {
                message: "Cannot aggregate on null"
            })
        );
        assert_eq!(aggregates.avg_ints(Some(&[])).expect("empty"), None);
    }

    fn render(result: Result<Option<BigDecimalValue>, AggregateError>) -> String {
        result.expect("aggregate").expect("value").to_string()
    }
}
