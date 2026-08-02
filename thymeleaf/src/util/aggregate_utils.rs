use std::fmt::{Display, Formatter};

use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::{Signed, Zero};
use thiserror::Error;

const NULL_TARGET_MESSAGE: &str = "Cannot aggregate on null";
const ITERABLE_NULL_SUM_MESSAGE: &str = "Cannot aggregate on iterable containing nulls";
const ARRAY_NULL_MESSAGE: &str = "Cannot aggregate on array containing nulls";

/// Java `BigDecimal` 的值、精度和 scale 适配。
///
/// 对应 Java: `java.math.BigDecimal`，由
/// `org.thymeleaf.util.AggregateUtils` 的全部聚合方法返回。
///
/// 本类型分别保存任意精度 unscaled value 与 32 位 scale，因而不会把 Java 的
/// `1`、`1.0`、`1.00` 或负 scale 科学计数法错误合并。
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct JavaBigDecimal {
    unscaled_value: BigInt,
    scale: i32,
}

impl JavaBigDecimal {
    /// 从 Java unscaled value 与 scale 创建十进制值。
    ///
    /// # 参数
    /// - `unscaled_value`：十进制小数点移除后的任意精度整数。
    /// - `scale`：Java `BigDecimal#scale()`。
    ///
    /// # 返回
    /// 保留两个构造参数且不做规范化的新值。
    #[must_use]
    pub fn from_unscaled(unscaled_value: BigInt, scale: i32) -> Self {
        Self {
            unscaled_value,
            scale,
        }
    }

    /// 按 Java `BigDecimal(String)` 语法解析十进制文本。
    ///
    /// # 参数
    /// - `value`：包含可选符号、小数点和十进制指数的文本。
    ///
    /// # 返回
    /// 成功时返回保留尾随零和 scale 的值。
    ///
    /// # 错误
    /// 语法非法或指数导致 scale 超出 Java `i32` 范围时返回数字格式错误。
    pub fn parse(value: &str) -> Result<Self, AggregateError> {
        parse_decimal(value)
    }

    /// 返回 Java `BigDecimal#unscaledValue()`。
    ///
    /// # 返回
    /// 未规范化的任意精度整数引用。
    #[must_use]
    pub fn unscaled_value(&self) -> &BigInt {
        &self.unscaled_value
    }

    /// 返回 Java `BigDecimal#scale()`。
    ///
    /// # 返回
    /// 小数点右侧位数；负值表示正指数。
    #[must_use]
    pub const fn scale(&self) -> i32 {
        self.scale
    }

    /// 返回 Java `BigDecimal#precision()`。
    ///
    /// # 返回
    /// unscaled value 的十进制数字数；零的精度固定为 1。
    #[must_use]
    pub fn precision(&self) -> usize {
        if self.unscaled_value.is_zero() {
            1
        } else {
            self.unscaled_value.abs().to_str_radix(10).len()
        }
    }

    /// 返回 Java `BigDecimal#toPlainString()` 等价文本。
    ///
    /// # 返回
    /// 不使用指数记法且保留 scale 的十进制文本。
    #[must_use]
    pub fn to_plain_string(&self) -> String {
        let negative = self.unscaled_value.sign() == Sign::Minus;
        let digits = self.unscaled_value.abs().to_str_radix(10);
        let mut result = if self.scale <= 0 {
            let zero_count = i64::from(self.scale)
                .checked_neg()
                .and_then(|count| usize::try_from(count).ok())
                .expect("Java scale magnitude must fit this platform");
            format!("{digits}{}", "0".repeat(zero_count))
        } else {
            let scale = usize::try_from(self.scale).expect("positive scale");
            if scale >= digits.len() {
                format!("0.{}{}", "0".repeat(scale - digits.len()), digits)
            } else {
                let split = digits.len() - scale;
                format!("{}.{}", &digits[..split], &digits[split..])
            }
        };
        if negative {
            result.insert(0, '-');
        }
        result
    }

    fn zero() -> Self {
        Self::from_unscaled(BigInt::ZERO, 0)
    }

    fn from_i64(value: i64) -> Self {
        Self::from_unscaled(BigInt::from(value), 0)
    }

    fn from_big_integer(value: &BigInt) -> Self {
        Self::from_unscaled(value.clone(), 0)
    }

    fn from_f64(value: f64) -> Result<Self, AggregateError> {
        if !value.is_finite() {
            return Err(AggregateError::NumberFormat {
                value: java_double_string(value),
            });
        }
        parse_decimal(&java_double_string(value))
    }

    pub(crate) fn from_f64_exact(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        if value == 0.0 {
            return Some(Self::zero());
        }

        // BigDecimal(double) 使用 IEEE-754 的精确二进制值，而不是 valueOf(double)
        // 的最短十进制文本。先还原 significand × 2^exponent，再把负二进制指数
        // 转换成最小的十进制 scale。
        let bits = value.to_bits();
        let negative = bits >> 63 != 0;
        let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
        let fraction = bits & ((1_u64 << 52) - 1);
        let (mut significand, binary_exponent) = if exponent_bits == 0 {
            (fraction, -1074)
        } else {
            ((1_u64 << 52) | fraction, exponent_bits - 1023 - 52)
        };

        let mut unscaled = if binary_exponent >= 0 {
            BigInt::from(significand)
                << usize::try_from(binary_exponent).expect("non-negative binary exponent")
        } else {
            let mut scale = -binary_exponent;
            while scale > 0 && significand & 1 == 0 {
                significand >>= 1;
                scale -= 1;
            }
            let scale_u32 = u32::try_from(scale).expect("IEEE-754 scale fits u32");
            let mut value = BigInt::from(significand) * BigInt::from(5_u8).pow(scale_u32);
            if negative {
                value = -value;
            }
            return Some(Self::from_unscaled(value, scale));
        };
        if negative {
            unscaled = -unscaled;
        }
        Some(Self::from_unscaled(unscaled, 0))
    }

    pub(crate) fn add_java(&self, other: &Self) -> Self {
        let result_scale = self.scale.max(other.scale);
        let left = rescale_unscaled(&self.unscaled_value, self.scale, result_scale);
        let right = rescale_unscaled(&other.unscaled_value, other.scale, result_scale);
        Self::from_unscaled(left + right, result_scale)
    }

    /// 按 `DecimalFormat` 默认的 HALF_EVEN 规则调整到指定 scale。
    pub(crate) fn with_scale_half_even(&self, target_scale: i32) -> Self {
        if target_scale >= self.scale {
            return Self::from_unscaled(
                rescale_unscaled(&self.unscaled_value, self.scale, target_scale),
                target_scale,
            );
        }
        let difference = u32::try_from(i64::from(self.scale) - i64::from(target_scale))
            .expect("positive scale difference fits u32");
        let divisor = BigInt::from(10_u8).pow(difference);
        let quotient = &self.unscaled_value / &divisor;
        let remainder = &self.unscaled_value % &divisor;
        let doubled = remainder.abs() * 2_u8;
        let increment = doubled > divisor || (doubled == divisor && quotient.is_odd());
        let rounded = if increment {
            quotient
                + if self.unscaled_value.sign() == Sign::Minus {
                    -BigInt::from(1_u8)
                } else {
                    BigInt::from(1_u8)
                }
        } else {
            quotient
        };
        Self::from_unscaled(rounded, target_scale)
    }

    /// 按 Java `BigDecimal#subtract` 语义相减。
    pub(crate) fn subtract_java(&self, other: &Self) -> Self {
        let result_scale = self.scale.max(other.scale);
        let left = rescale_unscaled(&self.unscaled_value, self.scale, result_scale);
        let right = rescale_unscaled(&other.unscaled_value, other.scale, result_scale);
        Self::from_unscaled(left - right, result_scale)
    }

    /// 按 Java `BigDecimal#compareTo` 比较数值，忽略 scale 差异。
    pub(crate) fn compare_java(&self, other: &Self) -> std::cmp::Ordering {
        let comparison_scale = self.scale.max(other.scale);
        let left = rescale_unscaled(&self.unscaled_value, self.scale, comparison_scale);
        let right = rescale_unscaled(&other.unscaled_value, other.scale, comparison_scale);
        left.cmp(&right)
    }

    /// 按 Java `BigDecimal#multiply` 语义相乘。
    pub(crate) fn multiply_java(
        &self,
        other: &Self,
    ) -> Result<Self, JavaBigDecimalArithmeticError> {
        let scale = checked_scale(i64::from(self.scale) + i64::from(other.scale))
            .map_err(|_| JavaBigDecimalArithmeticError::ScaleOverflow)?;
        Ok(Self::from_unscaled(
            &self.unscaled_value * &other.unscaled_value,
            scale,
        ))
    }

    /// 按 Java `BigDecimal#divide` 执行精确除法。
    pub(crate) fn divide_java(
        &self,
        divisor: &Self,
    ) -> Result<Self, JavaBigDecimalArithmeticError> {
        self.divide_exact(divisor).map_err(|error| match error {
            DivisionError::ByZero => JavaBigDecimalArithmeticError::DivisionByZero,
            DivisionError::NonTerminating => JavaBigDecimalArithmeticError::NonTerminating,
            DivisionError::ScaleOverflow => JavaBigDecimalArithmeticError::ScaleOverflow,
        })
    }

    /// 按指定 scale 和 HALF_UP 模式执行 Java BigDecimal 除法。
    pub(crate) fn divide_java_half_up(
        &self,
        divisor: &Self,
        scale: i32,
    ) -> Result<Self, JavaBigDecimalArithmeticError> {
        if divisor.unscaled_value.is_zero() {
            return Err(JavaBigDecimalArithmeticError::DivisionByZero);
        }
        let exponent = i64::from(divisor.scale) + i64::from(scale) - i64::from(self.scale);
        let mut numerator = self.unscaled_value.clone();
        let mut denominator = divisor.unscaled_value.clone();
        if exponent >= 0 {
            numerator *= BigInt::from(10_u8).pow(
                u32::try_from(exponent)
                    .map_err(|_| JavaBigDecimalArithmeticError::ScaleOverflow)?,
            );
        } else {
            denominator *= BigInt::from(10_u8).pow(
                u32::try_from(-exponent)
                    .map_err(|_| JavaBigDecimalArithmeticError::ScaleOverflow)?,
            );
        }
        let quotient = &numerator / &denominator;
        let remainder = &numerator % &denominator;
        let rounded = if remainder.abs() * 2_u8 >= denominator.abs() {
            if numerator.sign() == denominator.sign() {
                quotient + 1_u8
            } else {
                quotient - 1_u8
            }
        } else {
            quotient
        };
        Ok(Self::from_unscaled(rounded, scale))
    }

    /// 按 Java `BigDecimal#remainder` 返回截断商对应的余数。
    pub(crate) fn remainder_java(
        &self,
        divisor: &Self,
    ) -> Result<Self, JavaBigDecimalArithmeticError> {
        if divisor.unscaled_value.is_zero() {
            return Err(JavaBigDecimalArithmeticError::DivisionByZero);
        }
        let result_scale = self.scale.max(divisor.scale);
        let left = rescale_unscaled(&self.unscaled_value, self.scale, result_scale);
        let right = rescale_unscaled(&divisor.unscaled_value, divisor.scale, result_scale);
        Ok(Self::from_unscaled(left % right, result_scale))
    }

    fn divide_exact(&self, divisor: &Self) -> Result<Self, DivisionError> {
        if divisor.unscaled_value.is_zero() {
            return Err(DivisionError::ByZero);
        }

        let gcd = self.unscaled_value.abs().gcd(&divisor.unscaled_value.abs());
        let mut numerator = &self.unscaled_value / &gcd;
        let mut denominator = &divisor.unscaled_value / &gcd;
        if denominator.sign() == Sign::Minus {
            numerator = -numerator;
            denominator = -denominator;
        }

        let mut twos = 0_i64;
        while (&denominator % 2_u8).is_zero() {
            denominator /= 2_u8;
            twos += 1;
        }
        let mut fives = 0_i64;
        while (&denominator % 5_u8).is_zero() {
            denominator /= 5_u8;
            fives += 1;
        }
        if denominator != BigInt::from(1_u8) {
            return Err(DivisionError::NonTerminating);
        }

        let decimal_places = twos.max(fives);
        if twos < decimal_places {
            let exponent =
                u32::try_from(decimal_places - twos).expect("factor count fits Java scale");
            numerator *= BigInt::from(2_u8).pow(exponent);
        }
        if fives < decimal_places {
            let exponent =
                u32::try_from(decimal_places - fives).expect("factor count fits Java scale");
            numerator *= BigInt::from(5_u8).pow(exponent);
        }
        let preferred_scale = i64::from(self.scale) - i64::from(divisor.scale);
        let scale = checked_scale(preferred_scale + decimal_places)
            .map_err(|_| DivisionError::ScaleOverflow)?;
        Ok(Self::from_unscaled(numerator, scale))
    }

    fn divide_half_up_by_positive_integer(&self, divisor: i64, scale: i32) -> Self {
        let exponent = i64::from(scale) - i64::from(self.scale);
        let mut numerator = self.unscaled_value.clone();
        let exponent = u32::try_from(exponent)
            .expect("AggregateUtils fallback scale is never below total scale");
        numerator *= BigInt::from(10_u8).pow(exponent);
        let denominator = BigInt::from(divisor);

        let quotient = &numerator / &denominator;
        let remainder = &numerator % &denominator;
        let rounded = if remainder.abs() * 2_u8 >= denominator.abs() {
            if numerator.sign() == Sign::Minus {
                quotient - 1_u8
            } else {
                quotient + 1_u8
            }
        } else {
            quotient
        };
        Self::from_unscaled(rounded, scale)
    }

    fn java_string(&self) -> String {
        let negative = self.unscaled_value.sign() == Sign::Minus;
        let digits = self.unscaled_value.abs().to_str_radix(10);
        let adjusted_exponent = i64::try_from(self.precision()).expect("precision fits i64")
            - i64::from(self.scale)
            - 1;
        if self.scale >= 0 && adjusted_exponent >= -6 {
            return self.to_plain_string();
        }

        let mut result = String::new();
        if negative {
            result.push('-');
        }
        result.push_str(&digits[..1]);
        if digits.len() > 1 {
            result.push('.');
            result.push_str(&digits[1..]);
        }
        result.push('E');
        if adjusted_exponent >= 0 {
            result.push('+');
        }
        result.push_str(&adjusted_exponent.to_string());
        result
    }
}

impl Display for JavaBigDecimal {
    /// 输出 Java `BigDecimal#toString()` 等价文本。
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.java_string())
    }
}

/// Java `Number` 运行时类型及其转换语义。
///
/// 对应 Java: `java.lang.Number` 及 `AggregateUtils#toBigDecimal(Number)` 的
/// `instanceof` 分派。`Other` 表示自定义 `Number` 子类，只读取其 `doubleValue()`。
#[derive(Clone, Debug, PartialEq)]
pub enum JavaNumber {
    /// `java.math.BigDecimal`，转换时保留同一数值表示。
    BigDecimal(JavaBigDecimal),
    /// `java.math.BigInteger`。
    BigInteger(BigInt),
    /// `java.lang.Byte`。
    Byte(i8),
    /// `java.lang.Short`。
    Short(i16),
    /// `java.lang.Integer`。
    Integer(i32),
    /// `java.lang.Long`。
    Long(i64),
    /// `java.lang.Float`。
    Float(f32),
    /// `java.lang.Double`。
    Double(f64),
    /// 其他 `Number` 子类的类名和 `doubleValue()`。
    Other {
        /// Java 运行时类名。
        class_name: String,
        /// `Number#doubleValue()` 返回值。
        double_value: f64,
    },
}

/// Java `Object[]` 聚合元素。
///
/// 对应 Java: `AggregateUtils#sum(Object[])` 与 `avg(Object[])`。先完整检查 null，
/// 再在第二遍遍历时执行 `Number` 强制转换，从而保留异常优先级。
#[derive(Clone, Debug, PartialEq)]
pub enum JavaAggregateObject {
    /// Java null。
    Null,
    /// 任意 Java `Number`。
    Number(JavaNumber),
    /// 非 `Number` 对象及其运行时类名。
    Other(String),
}

/// 可重复获取 Java `Iterable<? extends Number>` 迭代器的契约。
///
/// Java `AggregateUtils` 先由 `Validate.containsNoNulls` 遍历一次，再执行第二次聚合
/// 遍历。本 trait 明确保留“两次调用 iterator()”的可观察行为。
pub trait JavaNumberIterable {
    /// 创建一次新的 Java 迭代器。
    ///
    /// # 返回
    /// 元素中的 `None` 对应 Java null。
    fn iter_java_numbers(&self) -> Box<dyn Iterator<Item = Option<&JavaNumber>> + '_>;
}

/// 可重复遍历的 Java 数字列表适配。
///
/// 对应 `Iterable<? extends Number>` 的常规集合输入；元素中的 `None` 保留 Java
/// null，并由 `AggregateUtils` 按原异常顺序校验。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JavaNumberList {
    values: Vec<Option<JavaNumber>>,
}

impl JavaNumberList {
    /// 从数字槽位创建列表。
    ///
    /// # 参数
    /// - `values`：按 Java 迭代顺序排列的可空数字。
    ///
    /// # 返回
    /// 拥有输入内容且可重复创建迭代器的列表。
    #[must_use]
    pub fn new(values: Vec<Option<JavaNumber>>) -> Self {
        Self { values }
    }

    /// 返回列表的可空数字切片。
    ///
    /// # 返回
    /// 保留顺序和 null 槽位的只读视图。
    #[must_use]
    pub fn as_slice(&self) -> &[Option<JavaNumber>] {
        &self.values
    }
}

impl JavaNumberIterable for JavaNumberList {
    fn iter_java_numbers(&self) -> Box<dyn Iterator<Item = Option<&JavaNumber>> + '_> {
        Box::new(self.values.iter().map(Option::as_ref))
    }
}

/// 聚合过程的 Java 异常等价分类。
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum AggregateError {
    /// Java `IllegalArgumentException`，用于 null 目标或 null 元素。
    #[error("{message}")]
    IllegalArgument {
        /// 上游固定异常消息。
        message: &'static str,
    },
    /// Java `ClassCastException`，用于 `Object[]` 中的非数字对象。
    #[error("class {actual_class} cannot be cast to class java.lang.Number")]
    ClassCast {
        /// 不能转换的 Java 运行时类名。
        actual_class: String,
    },
    /// Java `NumberFormatException`，用于非有限浮点值。
    #[error("Character array is missing \"e\" notation exponential mark for value {value}")]
    NumberFormat {
        /// 导致失败的 Java 浮点文本。
        value: String,
    },
    /// Java `ArithmeticException`，用于 scale 溢出等算术边界。
    #[error("{message}")]
    Arithmetic {
        /// 算术失败说明。
        message: String,
    },
}

/// Thymeleaf 数值聚合工具。
///
/// 对应 Java: `org.thymeleaf.util.AggregateUtils`。
///
/// 所有重载均返回 Java `BigDecimal` 等价值；空集合返回 Java null，null 目标和
/// null 元素保留原异常类别与消息，浮点输入按 `BigDecimal.valueOf(double)` 转换。
pub struct AggregateUtils;

impl AggregateUtils {
    /// 求可迭代数字之和。
    ///
    /// 对应 Java: `AggregateUtils#sum(Iterable<? extends Number>)`。
    ///
    /// # 参数
    /// - `target`：Java 参数 `target`；`None` 对应 null。
    ///
    /// # 返回
    /// 空 iterable 返回 `Ok(None)`，否则返回精确十进制总和。
    ///
    /// # 错误
    /// null 目标、null 元素或无法转换的数字返回 Java 等价错误。
    pub fn sum_iterable(
        target: Option<&dyn JavaNumberIterable>,
    ) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_iterable(target, ITERABLE_NULL_SUM_MESSAGE, false)
    }

    /// 求 Java `Object[]` 数字之和。
    ///
    /// 对应 Java: `AggregateUtils#sum(Object[])`。
    ///
    /// # 参数
    /// - `target`：对象数组；`None` 对应 Java null。
    ///
    /// # 返回
    /// 空数组返回 `Ok(None)`，否则返回精确十进制总和。
    ///
    /// # 错误
    /// null 元素优先于类型转换检查；非数字元素返回 `ClassCast`。
    pub fn sum_objects(
        target: Option<&[JavaAggregateObject]>,
    ) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_objects(target, false)
    }

    /// 求 Java `byte[]` 之和。对应 Java: `AggregateUtils#sum(byte[])`。
    pub fn sum_bytes(target: Option<&[i8]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Bytes), false)
    }

    /// 求 Java `short[]` 之和。对应 Java: `AggregateUtils#sum(short[])`。
    pub fn sum_shorts(target: Option<&[i16]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Shorts), false)
    }

    /// 求 Java `int[]` 之和。对应 Java: `AggregateUtils#sum(int[])`。
    pub fn sum_ints(target: Option<&[i32]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Ints), false)
    }

    /// 求 Java `long[]` 之和。对应 Java: `AggregateUtils#sum(long[])`。
    pub fn sum_longs(target: Option<&[i64]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Longs), false)
    }

    /// 求 Java `float[]` 之和。对应 Java: `AggregateUtils#sum(float[])`。
    pub fn sum_floats(target: Option<&[f32]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Floats), false)
    }

    /// 求 Java `double[]` 之和。对应 Java: `AggregateUtils#sum(double[])`。
    pub fn sum_doubles(target: Option<&[f64]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Doubles), false)
    }

    /// 求可迭代数字的平均值。
    ///
    /// 对应 Java: `AggregateUtils#avg(Iterable<? extends Number>)`。
    ///
    /// Java 源码在此重载中使用数组 null 消息；该看似不一致的文本被原样保留。
    pub fn avg_iterable(
        target: Option<&dyn JavaNumberIterable>,
    ) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_iterable(target, ARRAY_NULL_MESSAGE, true)
    }

    /// 求 Java `Object[]` 数字的平均值。对应 Java: `AggregateUtils#avg(Object[])`。
    pub fn avg_objects(
        target: Option<&[JavaAggregateObject]>,
    ) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_objects(target, true)
    }

    /// 求 Java `byte[]` 平均值。对应 Java: `AggregateUtils#avg(byte[])`。
    pub fn avg_bytes(target: Option<&[i8]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Bytes), true)
    }

    /// 求 Java `short[]` 平均值。对应 Java: `AggregateUtils#avg(short[])`。
    pub fn avg_shorts(target: Option<&[i16]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Shorts), true)
    }

    /// 求 Java `int[]` 平均值。对应 Java: `AggregateUtils#avg(int[])`。
    pub fn avg_ints(target: Option<&[i32]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Ints), true)
    }

    /// 求 Java `long[]` 平均值。对应 Java: `AggregateUtils#avg(long[])`。
    pub fn avg_longs(target: Option<&[i64]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Longs), true)
    }

    /// 求 Java `float[]` 平均值。对应 Java: `AggregateUtils#avg(float[])`。
    pub fn avg_floats(target: Option<&[f32]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Floats), true)
    }

    /// 求 Java `double[]` 平均值。对应 Java: `AggregateUtils#avg(double[])`。
    pub fn avg_doubles(target: Option<&[f64]>) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_primitive(target.map(PrimitiveArray::Doubles), true)
    }

    pub(crate) fn sum_numbers(
        target: Option<&[Option<JavaNumber>]>,
    ) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_number_array(target, false)
    }

    pub(crate) fn avg_numbers(
        target: Option<&[Option<JavaNumber>]>,
    ) -> Result<Option<JavaBigDecimal>, AggregateError> {
        aggregate_number_array(target, true)
    }
}

fn aggregate_iterable(
    target: Option<&dyn JavaNumberIterable>,
    null_message: &'static str,
    average: bool,
) -> Result<Option<JavaBigDecimal>, AggregateError> {
    let target = target.ok_or(AggregateError::IllegalArgument {
        message: NULL_TARGET_MESSAGE,
    })?;
    for element in target.iter_java_numbers() {
        if element.is_none() {
            return Err(AggregateError::IllegalArgument {
                message: null_message,
            });
        }
    }

    let mut total = JavaBigDecimal::zero();
    let mut size = 0_usize;
    for element in target.iter_java_numbers() {
        let number = element.ok_or(AggregateError::IllegalArgument {
            message: null_message,
        })?;
        total = total.add_java(&to_big_decimal(number)?);
        size += 1;
    }
    finish_aggregate(total, size, average)
}

fn aggregate_objects(
    target: Option<&[JavaAggregateObject]>,
    average: bool,
) -> Result<Option<JavaBigDecimal>, AggregateError> {
    let target = target.ok_or(AggregateError::IllegalArgument {
        message: NULL_TARGET_MESSAGE,
    })?;
    if target
        .iter()
        .any(|element| matches!(element, JavaAggregateObject::Null))
    {
        return Err(AggregateError::IllegalArgument {
            message: ARRAY_NULL_MESSAGE,
        });
    }

    let mut total = JavaBigDecimal::zero();
    for element in target {
        total = total.add_java(&to_big_decimal(object_number(element)?)?);
    }
    finish_aggregate(total, target.len(), average)
}

fn aggregate_number_array(
    target: Option<&[Option<JavaNumber>]>,
    average: bool,
) -> Result<Option<JavaBigDecimal>, AggregateError> {
    let target = target.ok_or(AggregateError::IllegalArgument {
        message: NULL_TARGET_MESSAGE,
    })?;
    let mut numbers = Vec::with_capacity(target.len());
    for number in target {
        match number {
            Some(number) => numbers.push(number),
            None => {
                return Err(AggregateError::IllegalArgument {
                    message: ARRAY_NULL_MESSAGE,
                });
            }
        }
    }
    let mut total = JavaBigDecimal::zero();
    for number in numbers {
        total = total.add_java(&to_big_decimal(number)?);
    }
    finish_aggregate(total, target.len(), average)
}

enum PrimitiveArray<'a> {
    Bytes(&'a [i8]),
    Shorts(&'a [i16]),
    Ints(&'a [i32]),
    Longs(&'a [i64]),
    Floats(&'a [f32]),
    Doubles(&'a [f64]),
}

fn aggregate_primitive(
    target: Option<PrimitiveArray<'_>>,
    average: bool,
) -> Result<Option<JavaBigDecimal>, AggregateError> {
    let target = target.ok_or(AggregateError::IllegalArgument {
        message: NULL_TARGET_MESSAGE,
    })?;
    let mut total = JavaBigDecimal::zero();
    let size = match target {
        PrimitiveArray::Bytes(values) => {
            for value in values {
                total = total.add_java(&JavaBigDecimal::from_i64(i64::from(*value)));
            }
            values.len()
        }
        PrimitiveArray::Shorts(values) => {
            for value in values {
                total = total.add_java(&JavaBigDecimal::from_i64(i64::from(*value)));
            }
            values.len()
        }
        PrimitiveArray::Ints(values) => {
            for value in values {
                total = total.add_java(&JavaBigDecimal::from_i64(i64::from(*value)));
            }
            values.len()
        }
        PrimitiveArray::Longs(values) => {
            for value in values {
                total = total.add_java(&JavaBigDecimal::from_i64(*value));
            }
            values.len()
        }
        PrimitiveArray::Floats(values) => {
            for value in values {
                total = total.add_java(&JavaBigDecimal::from_f64(f64::from(*value))?);
            }
            values.len()
        }
        PrimitiveArray::Doubles(values) => {
            for value in values {
                total = total.add_java(&JavaBigDecimal::from_f64(*value)?);
            }
            values.len()
        }
    };
    finish_aggregate(total, size, average)
}

fn finish_aggregate(
    total: JavaBigDecimal,
    size: usize,
    average: bool,
) -> Result<Option<JavaBigDecimal>, AggregateError> {
    if size == 0 {
        return Ok(None);
    }
    if !average {
        return Ok(Some(total));
    }

    let divisor_value = i64::try_from(size).map_err(|_| AggregateError::Arithmetic {
        message: "BigDecimal divisor exceeds Java long range".to_owned(),
    })?;
    let divisor = JavaBigDecimal::from_i64(divisor_value);
    match total.divide_exact(&divisor) {
        Ok(value) => Ok(Some(value)),
        Err(DivisionError::NonTerminating) => {
            let scale = total.scale.max(10);
            Ok(Some(
                total.divide_half_up_by_positive_integer(divisor_value, scale),
            ))
        }
        Err(error) => Err(AggregateError::from(error)),
    }
}

fn to_big_decimal(number: &JavaNumber) -> Result<JavaBigDecimal, AggregateError> {
    match number {
        JavaNumber::BigDecimal(value) => Ok(value.clone()),
        JavaNumber::BigInteger(value) => Ok(JavaBigDecimal::from_big_integer(value)),
        JavaNumber::Byte(value) => Ok(JavaBigDecimal::from_i64(i64::from(*value))),
        JavaNumber::Short(value) => Ok(JavaBigDecimal::from_i64(i64::from(*value))),
        JavaNumber::Integer(value) => Ok(JavaBigDecimal::from_i64(i64::from(*value))),
        JavaNumber::Long(value) => Ok(JavaBigDecimal::from_i64(*value)),
        JavaNumber::Float(value) => JavaBigDecimal::from_f64(f64::from(*value)),
        JavaNumber::Double(value) => JavaBigDecimal::from_f64(*value),
        JavaNumber::Other { double_value, .. } => JavaBigDecimal::from_f64(*double_value),
    }
}

fn object_number(element: &JavaAggregateObject) -> Result<&JavaNumber, AggregateError> {
    match element {
        JavaAggregateObject::Number(number) => Ok(number),
        JavaAggregateObject::Other(actual_class) => Err(AggregateError::ClassCast {
            actual_class: actual_class.clone(),
        }),
        JavaAggregateObject::Null => Err(AggregateError::IllegalArgument {
            message: ARRAY_NULL_MESSAGE,
        }),
    }
}

fn rescale_unscaled(unscaled: &BigInt, source_scale: i32, target_scale: i32) -> BigInt {
    if unscaled.is_zero() {
        return BigInt::ZERO;
    }
    let exponent = i64::from(target_scale) - i64::from(source_scale);
    let exponent = u32::try_from(exponent).expect("target scale is the maximum source scale");
    unscaled * BigInt::from(10_u8).pow(exponent)
}

fn checked_scale(scale: i64) -> Result<i32, AggregateError> {
    i32::try_from(scale).map_err(|_| AggregateError::Arithmetic {
        message: "Underflow".to_owned(),
    })
}

fn parse_decimal(value: &str) -> Result<JavaBigDecimal, AggregateError> {
    let (mantissa, exponent) = match value.find(['e', 'E']) {
        Some(index) => {
            let exponent =
                value[index + 1..]
                    .parse::<i64>()
                    .map_err(|_| AggregateError::NumberFormat {
                        value: value.to_owned(),
                    })?;
            (&value[..index], exponent)
        }
        None => (value, 0),
    };
    let negative = mantissa.starts_with('-');
    let unsigned = mantissa.strip_prefix(['-', '+']).unwrap_or(mantissa);
    let mut split = unsigned.split('.');
    let integer = split.next().unwrap_or_default();
    let fraction = split.next();
    if split.next().is_some()
        || (integer.is_empty() && fraction.is_none_or(str::is_empty))
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.is_some_and(|digits| !digits.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(AggregateError::NumberFormat {
            value: value.to_owned(),
        });
    }
    let fraction = fraction.unwrap_or_default();
    let digits = format!("{integer}{fraction}");
    let mut unscaled =
        BigInt::parse_bytes(digits.as_bytes(), 10).expect("validated decimal digits");
    if negative {
        unscaled = -unscaled;
    }
    let scale =
        checked_scale(i64::try_from(fraction.len()).expect("fraction length fits i64") - exponent)?;
    Ok(JavaBigDecimal::from_unscaled(unscaled, scale))
}

pub(crate) fn java_double_string(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == f64::INFINITY {
        return "Infinity".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_owned();
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0.0".to_owned()
        } else {
            "0.0".to_owned()
        };
    }
    if value.to_bits() == 1 {
        return "4.9E-324".to_owned();
    }
    if value.to_bits() == (1_u64 << 63) | 1 {
        return "-4.9E-324".to_owned();
    }

    let mut buffer = ryu::Buffer::new();
    let raw = buffer.format(value);
    let negative = raw.starts_with('-');
    let unsigned = raw.strip_prefix('-').unwrap_or(raw);
    let (mantissa, explicit_exponent) = match unsigned.find(['e', 'E']) {
        Some(index) => (
            &unsigned[..index],
            unsigned[index + 1..].parse::<i32>().expect("ryu exponent"),
        ),
        None => (unsigned, 0),
    };
    let fraction_length = mantissa
        .find('.')
        .map_or(0, |index| mantissa.len() - index - 1);
    let mut digits = mantissa.replace('.', "");
    let mut exponent = explicit_exponent - i32::try_from(fraction_length).expect("fraction");
    while digits.starts_with('0') && digits.len() > 1 {
        digits.remove(0);
    }
    while digits.ends_with('0') && digits.len() > 1 {
        digits.pop();
        exponent += 1;
    }
    let decimal_position = i32::try_from(digits.len()).expect("digits") + exponent;
    let absolute = value.abs();
    let mut result = if (1.0e-3..1.0e7).contains(&absolute) {
        if decimal_position <= 0 {
            format!(
                "0.{}{}",
                "0".repeat(usize::try_from(-decimal_position).expect("zero count")),
                digits
            )
        } else if usize::try_from(decimal_position).expect("position") >= digits.len() {
            format!(
                "{}{}.0",
                digits,
                "0".repeat(usize::try_from(decimal_position).expect("position") - digits.len())
            )
        } else {
            let split = usize::try_from(decimal_position).expect("position");
            format!("{}.{}", &digits[..split], &digits[split..])
        }
    } else {
        let scientific_exponent = decimal_position - 1;
        let fraction = if digits.len() == 1 { "0" } else { &digits[1..] };
        format!("{}.{fraction}E{scientific_exponent}", &digits[..1])
    };
    if negative {
        result.insert(0, '-');
    }
    result
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DivisionError {
    ByZero,
    NonTerminating,
    ScaleOverflow,
}

/// Standard Expression 使用 Java BigDecimal 运算时的 ArithmeticException 分类。
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum JavaBigDecimalArithmeticError {
    /// 除数为零。
    #[error("Division by zero")]
    DivisionByZero,
    /// 精确除法结果具有无限循环小数。
    #[error("Non-terminating decimal expansion")]
    NonTerminating,
    /// 结果 scale 超出 Java int 范围。
    #[error("Underflow")]
    ScaleOverflow,
}

impl From<DivisionError> for AggregateError {
    fn from(error: DivisionError) -> Self {
        let message = match error {
            DivisionError::ByZero => "Division by zero",
            DivisionError::NonTerminating => "Non-terminating decimal expansion",
            DivisionError::ScaleOverflow => "Underflow",
        };
        Self::Arithmetic {
            message: message.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use num_bigint::BigInt;

    use super::{
        AggregateError, AggregateUtils, JavaAggregateObject, JavaBigDecimal, JavaNumber,
        JavaNumberIterable, JavaNumberList, java_double_string,
    };

    #[test]
    fn preserves_big_decimal_representation_and_java_display_boundaries() {
        let values = [
            ("1", "1", "1", 0),
            ("1.00", "1.00", "1.00", 2),
            ("1E+7", "1E+7", "10000000", -7),
            ("0.000001", "0.000001", "0.000001", 6),
            ("0.0000001", "1E-7", "0.0000001", 7),
            ("-0.0", "0.0", "0.0", 1),
        ];
        for (source, display, plain, scale) in values {
            let value = JavaBigDecimal::parse(source).expect("decimal");
            assert_eq!(value.to_string(), display);
            assert_eq!(value.to_plain_string(), plain);
            assert_eq!(value.scale(), scale);
            assert!(value.precision() >= 1);
        }
        let explicit = JavaBigDecimal::from_unscaled(BigInt::from(123), 2);
        assert_eq!(explicit.unscaled_value(), &BigInt::from(123));
        assert_eq!(explicit.to_string(), "1.23");
    }

    #[test]
    fn formats_java_double_thresholds_special_values_and_signed_zero() {
        assert_eq!(java_double_string(1.0), "1.0");
        assert_eq!(java_double_string(9_999_999.0), "9999999.0");
        assert_eq!(java_double_string(10_000_000.0), "1.0E7");
        assert_eq!(java_double_string(0.001), "0.001");
        assert_eq!(java_double_string(0.0001), "1.0E-4");
        assert_eq!(java_double_string(-0.0), "-0.0");
        assert_eq!(java_double_string(f64::NAN), "NaN");
        assert_eq!(java_double_string(f64::INFINITY), "Infinity");
        assert_eq!(java_double_string(f64::NEG_INFINITY), "-Infinity");
    }

    #[test]
    fn sums_all_number_runtime_types_and_preserves_scale() {
        let numbers = JavaNumberList::new(vec![
            Some(JavaNumber::BigDecimal(
                JavaBigDecimal::parse("1.20").expect("decimal"),
            )),
            Some(JavaNumber::BigInteger(BigInt::from(2))),
            Some(JavaNumber::Byte(3)),
            Some(JavaNumber::Short(4)),
            Some(JavaNumber::Integer(5)),
            Some(JavaNumber::Long(6)),
            Some(JavaNumber::Float(0.5)),
            Some(JavaNumber::Double(0.25)),
            Some(JavaNumber::Other {
                class_name: "example.CustomNumber".to_owned(),
                double_value: 0.05,
            }),
        ]);
        assert_eq!(numbers.as_slice().len(), 9);
        let result = AggregateUtils::sum_iterable(Some(&numbers))
            .expect("sum")
            .expect("value");
        assert_eq!(result.to_string(), "22.00");
        assert_eq!(result.scale(), 2);
    }

    #[test]
    fn averages_exactly_or_with_java_half_up_scale() {
        let exact = AggregateUtils::avg_ints(Some(&[1, 2]))
            .expect("average")
            .expect("value");
        assert_eq!(exact.to_string(), "1.5");
        let repeating = AggregateUtils::avg_ints(Some(&[1, 1, 2]))
            .expect("average")
            .expect("value");
        assert_eq!(repeating.to_string(), "1.3333333333");
        let negative = AggregateUtils::avg_ints(Some(&[-1, -1, -2]))
            .expect("average")
            .expect("value");
        assert_eq!(negative.to_string(), "-1.3333333333");
        let scaled = AggregateUtils::avg_objects(Some(&[
            JavaAggregateObject::Number(JavaNumber::BigDecimal(
                JavaBigDecimal::parse("1.000000000000").expect("decimal"),
            )),
            JavaAggregateObject::Number(JavaNumber::Integer(2)),
            JavaAggregateObject::Number(JavaNumber::Integer(2)),
        ]))
        .expect("average")
        .expect("value");
        assert_eq!(scaled.to_string(), "1.666666666667");
        assert_eq!(scaled.scale(), 12);
    }

    #[test]
    fn preserves_null_validation_order_and_error_categories() {
        assert_eq!(
            AggregateUtils::sum_ints(None),
            Err(AggregateError::IllegalArgument {
                message: "Cannot aggregate on null"
            })
        );
        let null_numbers = JavaNumberList::new(vec![Some(JavaNumber::Integer(1)), None]);
        assert_eq!(
            AggregateUtils::sum_iterable(Some(&null_numbers)),
            Err(AggregateError::IllegalArgument {
                message: "Cannot aggregate on iterable containing nulls"
            })
        );
        assert_eq!(
            AggregateUtils::avg_iterable(Some(&null_numbers)),
            Err(AggregateError::IllegalArgument {
                message: "Cannot aggregate on array containing nulls"
            })
        );
        let objects = [
            JavaAggregateObject::Other("java.lang.String".to_owned()),
            JavaAggregateObject::Null,
        ];
        assert_eq!(
            AggregateUtils::sum_objects(Some(&objects)),
            Err(AggregateError::IllegalArgument {
                message: "Cannot aggregate on array containing nulls"
            })
        );
        assert_eq!(
            AggregateUtils::sum_objects(Some(&[JavaAggregateObject::Other(
                "java.lang.String".to_owned()
            )])),
            Err(AggregateError::ClassCast {
                actual_class: "java.lang.String".to_owned()
            })
        );
        assert_eq!(
            AggregateUtils::sum_doubles(Some(&[f64::NAN])),
            Err(AggregateError::NumberFormat {
                value: "NaN".to_owned()
            })
        );
        let nan_numbers = JavaNumberList::new(vec![Some(JavaNumber::Double(f64::NAN))]);
        assert_eq!(
            AggregateUtils::sum_iterable(Some(&nan_numbers)),
            Err(AggregateError::NumberFormat {
                value: "NaN".to_owned()
            })
        );
        assert_eq!(
            AggregateUtils::sum_objects(Some(&[JavaAggregateObject::Number(JavaNumber::Double(
                f64::NAN
            ))])),
            Err(AggregateError::NumberFormat {
                value: "NaN".to_owned()
            })
        );
    }

    #[test]
    fn invokes_java_iterable_twice_and_handles_empty_targets() {
        struct CountingIterable {
            values: Vec<Option<JavaNumber>>,
            iterations: Cell<usize>,
        }
        impl JavaNumberIterable for CountingIterable {
            fn iter_java_numbers(&self) -> Box<dyn Iterator<Item = Option<&JavaNumber>> + '_> {
                self.iterations.set(self.iterations.get() + 1);
                Box::new(self.values.iter().map(Option::as_ref))
            }
        }

        let iterable = CountingIterable {
            values: vec![Some(JavaNumber::Integer(1))],
            iterations: Cell::new(0),
        };
        assert_eq!(
            AggregateUtils::sum_iterable(Some(&iterable))
                .expect("sum")
                .expect("value")
                .to_string(),
            "1"
        );
        assert_eq!(iterable.iterations.get(), 2);
        assert_eq!(AggregateUtils::sum_ints(Some(&[])).expect("sum"), None);
        assert_eq!(AggregateUtils::avg_ints(Some(&[])).expect("avg"), None);
        assert_eq!(AggregateUtils::sum_objects(Some(&[])).expect("sum"), None);
    }

    #[test]
    fn exercises_every_primitive_overload_and_float_failure() {
        assert_eq!(
            AggregateUtils::sum_bytes(Some(&[1, 2]))
                .expect("sum")
                .expect("value")
                .to_string(),
            "3"
        );
        assert_eq!(
            AggregateUtils::sum_shorts(Some(&[1, 2]))
                .expect("sum")
                .expect("value")
                .to_string(),
            "3"
        );
        assert_eq!(
            AggregateUtils::sum_longs(Some(&[1, 2]))
                .expect("sum")
                .expect("value")
                .to_string(),
            "3"
        );
        assert_eq!(
            AggregateUtils::sum_floats(Some(&[0.5, 0.25]))
                .expect("sum")
                .expect("value")
                .to_string(),
            "0.75"
        );
        assert_eq!(
            AggregateUtils::avg_bytes(Some(&[1, 2]))
                .expect("avg")
                .expect("value")
                .to_string(),
            "1.5"
        );
        assert_eq!(
            AggregateUtils::avg_shorts(Some(&[1, 2]))
                .expect("avg")
                .expect("value")
                .to_string(),
            "1.5"
        );
        assert_eq!(
            AggregateUtils::avg_longs(Some(&[1, 2]))
                .expect("avg")
                .expect("value")
                .to_string(),
            "1.5"
        );
        assert_eq!(
            AggregateUtils::avg_floats(Some(&[0.5, 0.25]))
                .expect("avg")
                .expect("value")
                .to_string(),
            "0.375"
        );
        assert_eq!(
            AggregateUtils::sum_doubles(Some(&[0.5, 0.25]))
                .expect("sum")
                .expect("value")
                .to_string(),
            "0.75"
        );
        assert_eq!(
            AggregateUtils::avg_doubles(Some(&[0.5, 0.25]))
                .expect("avg")
                .expect("value")
                .to_string(),
            "0.375"
        );
        assert!(AggregateUtils::sum_floats(Some(&[f32::INFINITY])).is_err());
    }

    #[test]
    fn rejects_malformed_decimal_and_covers_arithmetic_errors() {
        assert_eq!(
            JavaBigDecimal::parse(".1")
                .expect("leading point")
                .to_string(),
            "0.1"
        );
        assert_eq!(
            JavaBigDecimal::parse("1.")
                .expect("trailing point")
                .to_string(),
            "1"
        );
        for malformed in ["", ".", "1.2.3", "1e", "NaN"] {
            assert!(JavaBigDecimal::parse(malformed).is_err(), "{malformed}");
        }
        assert!(JavaBigDecimal::parse("1e2147483649").is_err());
        assert_eq!(
            AggregateError::from(super::DivisionError::ByZero).to_string(),
            "Division by zero"
        );
        assert_eq!(
            AggregateError::from(super::DivisionError::NonTerminating).to_string(),
            "Non-terminating decimal expansion"
        );
        assert_eq!(
            AggregateError::from(super::DivisionError::ScaleOverflow).to_string(),
            "Underflow"
        );
    }

    #[test]
    fn covers_exact_division_rounding_and_scale_boundaries() {
        let one = JavaBigDecimal::parse("1").expect("one");
        let zero = JavaBigDecimal::parse("0").expect("zero");
        assert_eq!(one.divide_exact(&zero), Err(super::DivisionError::ByZero));
        assert_eq!(
            one.divide_exact(&JavaBigDecimal::parse("-8").expect("negative divisor"))
                .expect("exact")
                .to_string(),
            "-0.125"
        );
        assert_eq!(
            one.divide_exact(&JavaBigDecimal::parse("8").expect("eight"))
                .expect("exact")
                .to_string(),
            "0.125"
        );
        assert_eq!(
            one.divide_exact(&JavaBigDecimal::parse("125").expect("one hundred twenty five"))
                .expect("exact")
                .to_string(),
            "0.008"
        );
        assert_eq!(
            JavaBigDecimal::from_unscaled(BigInt::from(1), i32::MAX)
                .divide_exact(&JavaBigDecimal::from_unscaled(BigInt::from(1), i32::MIN)),
            Err(super::DivisionError::ScaleOverflow)
        );

        assert_eq!(
            JavaBigDecimal::parse("1")
                .expect("one")
                .divide_half_up_by_positive_integer(3, 0)
                .to_string(),
            "0"
        );
        assert_eq!(
            JavaBigDecimal::parse("2")
                .expect("two")
                .divide_half_up_by_positive_integer(3, 0)
                .to_string(),
            "1"
        );
        assert_eq!(
            JavaBigDecimal::parse("-1")
                .expect("negative one")
                .divide_half_up_by_positive_integer(2, 0)
                .to_string(),
            "-1"
        );
        assert!(super::finish_aggregate(JavaBigDecimal::zero(), usize::MAX, true).is_err());
    }

    #[test]
    fn covers_internal_validation_and_stateful_second_iteration() {
        assert!(AggregateUtils::sum_iterable(None).is_err());
        assert_eq!(
            super::object_number(&JavaAggregateObject::Null),
            Err(AggregateError::IllegalArgument {
                message: "Cannot aggregate on array containing nulls"
            })
        );
        struct ChangesAfterValidation {
            calls: Cell<usize>,
            number: JavaNumber,
        }
        impl JavaNumberIterable for ChangesAfterValidation {
            fn iter_java_numbers(&self) -> Box<dyn Iterator<Item = Option<&JavaNumber>> + '_> {
                let call = self.calls.get();
                self.calls.set(call + 1);
                if call == 0 {
                    Box::new(std::iter::once(Some(&self.number)))
                } else {
                    Box::new(std::iter::once(None))
                }
            }
        }
        let iterable = ChangesAfterValidation {
            calls: Cell::new(0),
            number: JavaNumber::Integer(1),
        };
        assert_eq!(
            AggregateUtils::sum_iterable(Some(&iterable)),
            Err(AggregateError::IllegalArgument {
                message: "Cannot aggregate on iterable containing nulls"
            })
        );

        let negative_scientific = JavaBigDecimal::parse("-12E+7").expect("scientific");
        assert_eq!(negative_scientific.to_string(), "-1.2E+8");
        assert_eq!(
            JavaBigDecimal::parse("+1.5E+2")
                .expect("positive exponent")
                .to_string(),
            "1.5E+2"
        );
    }

    #[test]
    fn covers_every_array_null_and_number_array_validation_path() {
        assert!(AggregateUtils::sum_bytes(None).is_err());
        assert!(AggregateUtils::avg_bytes(None).is_err());
        assert!(AggregateUtils::sum_shorts(None).is_err());
        assert!(AggregateUtils::avg_shorts(None).is_err());
        assert!(AggregateUtils::avg_ints(None).is_err());
        assert!(AggregateUtils::sum_longs(None).is_err());
        assert!(AggregateUtils::avg_longs(None).is_err());
        assert!(AggregateUtils::sum_floats(None).is_err());
        assert!(AggregateUtils::avg_floats(None).is_err());
        assert!(AggregateUtils::sum_doubles(None).is_err());
        assert!(AggregateUtils::avg_doubles(None).is_err());
        assert!(AggregateUtils::avg_floats(Some(&[f32::NAN])).is_err());
        assert!(AggregateUtils::avg_doubles(Some(&[f64::NAN])).is_err());

        assert!(AggregateUtils::sum_numbers(None).is_err());
        assert!(AggregateUtils::avg_numbers(None).is_err());
        let null_numbers = [None];
        assert!(AggregateUtils::sum_numbers(Some(&null_numbers)).is_err());
        assert!(AggregateUtils::avg_numbers(Some(&null_numbers)).is_err());
        let nan_numbers = [Some(JavaNumber::Double(f64::NAN))];
        assert!(AggregateUtils::sum_numbers(Some(&nan_numbers)).is_err());
        assert!(AggregateUtils::avg_numbers(Some(&nan_numbers)).is_err());

        let overflow_average = [
            JavaAggregateObject::Number(JavaNumber::BigDecimal(JavaBigDecimal::from_unscaled(
                BigInt::from(1),
                i32::MAX,
            ))),
            JavaAggregateObject::Number(JavaNumber::BigDecimal(JavaBigDecimal::from_unscaled(
                BigInt::from(0),
                i32::MAX,
            ))),
        ];
        assert_eq!(
            AggregateUtils::avg_objects(Some(&overflow_average)),
            Err(AggregateError::Arithmetic {
                message: "Underflow".to_owned()
            })
        );
    }
}
