use thiserror::Error;

use num_bigint::BigInt;

use super::{JavaBigDecimal, JavaLocale, JavaNumber, JavaString, NumberPointType};

/// 数字格式化与序列生成错误。
#[derive(Debug, Error)]
pub enum NumberUtilsError {
    /// 必填参数为空或序列方向非法。
    #[error("{message}")]
    InvalidArgument {
        /// Java 异常消息。
        message: String,
    },
}

/// Thymeleaf 数字格式化基础工具。
///
/// 对应 Java: `org.thymeleaf.util.NumberUtils`。
pub struct NumberUtils;

impl NumberUtils {
    /// 使用指定整数、小数位和分隔符规则格式化数字。
    pub fn format(
        target: Option<&JavaNumber>,
        min_integer_digits: Option<i32>,
        thousands_point_type: Option<NumberPointType>,
        fraction_digits: Option<i32>,
        decimal_point_type: Option<NumberPointType>,
        locale: Option<&JavaLocale>,
    ) -> Result<Option<JavaString>, NumberUtilsError> {
        let Some(target) = target else {
            return Ok(None);
        };
        let thousands_point_type =
            required(thousands_point_type, "Thousands point type cannot be null")?;
        let fraction_digits = required(fraction_digits, "Fraction digits cannot be null")?;
        let decimal_point_type = required(decimal_point_type, "Decimal point type cannot be null")?;
        let locale = locale.ok_or_else(|| invalid("Locale cannot be null"))?;
        if fraction_digits < 0 {
            return Err(invalid(
                "Minimum fraction digits must be greater than or equal to zero",
            ));
        }
        if min_integer_digits.is_some_and(|digits| digits < 0) {
            return Err(invalid(
                "Minimum integer digits must be greater than or equal to zero",
            ));
        }

        let rendered = match number_as_decimal(target) {
            Some(value) => value
                .with_scale_half_even(fraction_digits)
                .to_plain_string(),
            None => number_as_f64(target).to_string(),
        };
        let negative = rendered.starts_with('-');
        let rendered = rendered.strip_prefix('-').unwrap_or(&rendered);
        let (integer, fraction) = rendered
            .split_once('.')
            .map_or((rendered, None), |(integer, fraction)| {
                (integer, Some(fraction))
            });
        let mut integer = integer.to_owned();
        let fraction = fraction.map(str::to_owned);
        let min_integer_digits = min_integer_digits.unwrap_or(0).max(0) as usize;
        if integer.len() < min_integer_digits {
            integer.insert_str(0, &"0".repeat(min_integer_digits - integer.len()));
        }
        if thousands_point_type != NumberPointType::None {
            integer = group_integer(
                &integer,
                point_character(thousands_point_type, locale, false),
            );
        }
        let mut rendered = String::new();
        if negative {
            rendered.push('-');
        }
        rendered.push_str(&integer);
        if fraction_digits > 0 {
            // Java NumberPointType.NONE 的 point character 是 '?'；它只禁用
            // 千位分组，不会在请求小数位时删除整个小数部分。
            rendered.push(point_character(decimal_point_type, locale, true));
            rendered.push_str(fraction.as_deref().unwrap_or(""));
        }
        Ok(Some(JavaString::from_rust_str(&rendered)))
    }

    /// 创建包含边界且按方向选择默认步长的整数序列。
    pub fn sequence(from: Option<i32>, to: Option<i32>) -> Result<Vec<i32>, NumberUtilsError> {
        let from = required(from, "Value to start the sequence from cannot be null")?;
        let to = required(to, "Value to generate the sequence up to cannot be null")?;
        Self::sequence_with_step(Some(from), Some(to), Some(if from <= to { 1 } else { -1 }))
    }

    /// 创建包含边界的整数序列；步长方向不匹配时返回空序列。
    pub fn sequence_with_step(
        from: Option<i32>,
        to: Option<i32>,
        step: Option<i32>,
    ) -> Result<Vec<i32>, NumberUtilsError> {
        let from = required(from, "Value to start the sequence from cannot be null")?;
        let to = required(to, "Value to generate the sequence up to cannot be null")?;
        let step = required(step, "Step to generate the sequence cannot be null")?;
        if from == to {
            return Ok(vec![from]);
        }
        if step == 0 {
            return Err(invalid(format!(
                "Cannot create sequence from {from} to {to} with step {step}"
            )));
        }
        let mut values = Vec::new();
        if from < to && step > 0 {
            let mut value = from;
            while value <= to {
                values.push(value);
                let next = value.wrapping_add(step);
                if next <= value {
                    break;
                }
                value = next;
            }
        } else if from > to && step < 0 {
            let mut value = from;
            while value >= to {
                values.push(value);
                let next = value.wrapping_add(step);
                if next >= value {
                    break;
                }
                value = next;
            }
        }
        Ok(values)
    }

    /// 按 Locale 货币格式格式化数字。
    pub fn format_currency(
        target: Option<&JavaNumber>,
        locale: Option<&JavaLocale>,
    ) -> Result<Option<JavaString>, NumberUtilsError> {
        let locale = locale.ok_or_else(|| invalid("Locale cannot be null"))?;
        let Some(target) = target else {
            return Ok(None);
        };
        let number = Self::format(
            Some(target),
            Some(1),
            Some(NumberPointType::Default),
            Some(currency_fraction_digits(locale)),
            Some(NumberPointType::Default),
            Some(locale),
        )?
        .expect("non-null target");
        let symbol = currency_symbol(locale);
        let text = if currency_symbol_after(locale) {
            format!("{} {symbol}", number.to_string_lossy())
        } else {
            format!("{symbol}{}", number.to_string_lossy())
        };
        Ok(Some(JavaString::from_rust_str(&text)))
    }

    /// 把数字乘以一百并按 Locale 百分号格式输出。
    pub fn format_percent(
        target: Option<&JavaNumber>,
        min_integer_digits: Option<i32>,
        fraction_digits: Option<i32>,
        locale: Option<&JavaLocale>,
    ) -> Result<Option<JavaString>, NumberUtilsError> {
        let fraction_digits = required(fraction_digits, "Fraction digits cannot be null")?;
        let locale = locale.ok_or_else(|| invalid("Locale cannot be null"))?;
        let Some(target) = target else {
            return Ok(None);
        };
        let percent = number_as_decimal(target).map_or_else(
            || JavaNumber::Double(number_as_f64(target) * 100.0),
            |value| {
                JavaNumber::BigDecimal(JavaBigDecimal::from_unscaled(
                    value.unscaled_value().clone() * BigInt::from(100_u8),
                    value.scale(),
                ))
            },
        );
        let number = Self::format(
            Some(&percent),
            min_integer_digits,
            Some(NumberPointType::Default),
            Some(fraction_digits),
            Some(NumberPointType::Default),
            Some(locale),
        )?
        .expect("non-null target");
        let separator = if locale_uses_space_before_percent(locale) {
            " "
        } else {
            ""
        };
        Ok(Some(JavaString::from_rust_str(&format!(
            "{}{separator}%",
            number.to_string_lossy()
        ))))
    }
}

fn required<T>(value: Option<T>, message: &str) -> Result<T, NumberUtilsError> {
    value.ok_or_else(|| invalid(message))
}

fn invalid(message: impl Into<String>) -> NumberUtilsError {
    NumberUtilsError::InvalidArgument {
        message: message.into(),
    }
}

fn number_as_f64(number: &JavaNumber) -> f64 {
    match number {
        JavaNumber::BigDecimal(value) => value.to_string().parse().unwrap_or(f64::NAN),
        JavaNumber::BigInteger(value) => value.to_string().parse().unwrap_or(f64::INFINITY),
        JavaNumber::Byte(value) => f64::from(*value),
        JavaNumber::Short(value) => f64::from(*value),
        JavaNumber::Integer(value) => f64::from(*value),
        JavaNumber::Long(value) => *value as f64,
        JavaNumber::Float(value) => f64::from(*value),
        JavaNumber::Double(value) => *value,
        JavaNumber::Other { double_value, .. } => *double_value,
    }
}

fn number_as_decimal(number: &JavaNumber) -> Option<JavaBigDecimal> {
    match number {
        JavaNumber::BigDecimal(value) => Some(value.clone()),
        JavaNumber::BigInteger(value) => Some(JavaBigDecimal::from_unscaled(value.clone(), 0)),
        JavaNumber::Byte(value) => Some(JavaBigDecimal::from_unscaled(BigInt::from(*value), 0)),
        JavaNumber::Short(value) => Some(JavaBigDecimal::from_unscaled(BigInt::from(*value), 0)),
        JavaNumber::Integer(value) => Some(JavaBigDecimal::from_unscaled(BigInt::from(*value), 0)),
        JavaNumber::Long(value) => Some(JavaBigDecimal::from_unscaled(BigInt::from(*value), 0)),
        JavaNumber::Float(value) => JavaBigDecimal::parse(&value.to_string()).ok(),
        JavaNumber::Double(value) => JavaBigDecimal::parse(&value.to_string()).ok(),
        JavaNumber::Other { double_value, .. } => {
            JavaBigDecimal::parse(&double_value.to_string()).ok()
        }
    }
}

fn group_integer(integer: &str, separator: char) -> String {
    let mut result = String::with_capacity(integer.len() + integer.len() / 3);
    let first_group = match integer.len() % 3 {
        0 => 3,
        value => value,
    };
    for (index, character) in integer.chars().enumerate() {
        if index != 0 && index >= first_group && (index - first_group) % 3 == 0 {
            result.push(separator);
        }
        result.push(character);
    }
    result
}

fn point_character(point_type: NumberPointType, locale: &JavaLocale, decimal: bool) -> char {
    match point_type {
        NumberPointType::Point => '.',
        NumberPointType::Comma => ',',
        NumberPointType::Whitespace => ' ',
        NumberPointType::Default => {
            if locale_uses_decimal_comma(locale) {
                if decimal { ',' } else { '.' }
            } else if decimal {
                '.'
            } else {
                ','
            }
        }
        NumberPointType::None => '?',
    }
}

fn locale_uses_decimal_comma(locale: &JavaLocale) -> bool {
    matches!(
        locale.get_language().to_string_lossy().as_str(),
        "ar" | "bg"
            | "cs"
            | "da"
            | "de"
            | "el"
            | "es"
            | "fi"
            | "fr"
            | "hu"
            | "id"
            | "it"
            | "nl"
            | "no"
            | "pl"
            | "pt"
            | "ro"
            | "ru"
            | "sk"
            | "sl"
            | "sv"
            | "tr"
            | "uk"
            | "vi"
    )
}

fn currency_symbol(locale: &JavaLocale) -> &'static str {
    match locale.get_country().to_string_lossy().as_str() {
        "US" => "$",
        "GB" => "£",
        "JP" => "￥",
        "CN" => "¥",
        "KR" => "₩",
        "CH" => "CHF",
        "IN" => "₹",
        "CA" => "CA$",
        "AU" => "A$",
        _ if locale_uses_decimal_comma(locale) => "€",
        _ => "¤",
    }
}

fn currency_fraction_digits(locale: &JavaLocale) -> i32 {
    if matches!(locale.get_country().to_string_lossy().as_str(), "JP" | "KR") {
        0
    } else {
        2
    }
}

fn currency_symbol_after(locale: &JavaLocale) -> bool {
    locale_uses_decimal_comma(locale)
}

fn locale_uses_space_before_percent(locale: &JavaLocale) -> bool {
    matches!(
        locale.get_language().to_string_lossy().as_str(),
        "fr" | "ru" | "uk" | "pl" | "cs" | "sk"
    )
}
