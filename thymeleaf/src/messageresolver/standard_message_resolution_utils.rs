use std::any::TypeId;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{OnceLock, RwLock};

use num_bigint::BigInt;
use num_traits::Signed;

use crate::expression::TemplateValue;
use crate::templateresource::ITemplateResource;
use crate::util::{
    JavaBigDecimal, JavaLocale, JavaNumber, JavaString, NumberPointType, NumberUtils,
};
use crate::{TemplateInputException, TemplateProcessingException};

use super::MessageResolutionResult;

type Messages = HashMap<JavaString, JavaString>;
type OriginMessages = HashMap<(TypeId, JavaLocale), Messages>;
type OriginParents = HashMap<TypeId, TypeId>;

static ORIGIN_MESSAGES: OnceLock<RwLock<OriginMessages>> = OnceLock::new();
static ORIGIN_PARENTS: OnceLock<RwLock<OriginParents>> = OnceLock::new();

/// 标准消息资源定位、合并与格式化工具。
///
/// 对应 Java: `org.thymeleaf.messageresolver.StandardMessageResolutionUtils`。
pub(crate) struct StandardMessageResolutionUtils;

impl StandardMessageResolutionUtils {
    /// 按基础资源、语言、国家和变体由低到高合并模板消息。
    pub(crate) fn resolve_messages_for_template(
        template_resource: &dyn ITemplateResource,
        locale: &JavaLocale,
    ) -> MessageResolutionResult<Messages> {
        let Some(resource_base_name) = template_resource
            .get_base_name()
            .filter(|base_name| !base_name.is_empty())
        else {
            return Ok(HashMap::new());
        };

        let mut combined_messages = HashMap::new();
        for message_resource_name in
            Self::compute_message_resource_names_from_base(&resource_base_name, locale)?
        {
            let message_resource = template_resource
                .relative(Some(&message_resource_name))
                .map_err(|error| Box::new(error) as super::MessageResolutionError)?;
            let Ok(reader) = message_resource.reader() else {
                // Java 版本只忽略派生消息文件不存在或打开失败时产生的 IOException。
                continue;
            };
            combined_messages.extend(Self::read_messages_resource(reader)?);
        }
        Ok(combined_messages)
    }

    /// 返回宿主为 Rust 类型注册的 classpath 等价消息。
    pub(crate) fn resolve_messages_for_origin(origin: TypeId, locale: &JavaLocale) -> Messages {
        let messages = read_lock(origin_messages());
        let parents = read_lock(origin_parents());
        let mut combined = Messages::new();
        let mut current = Some(origin);
        let mut visited = std::collections::HashSet::new();

        // Java 从具体类向父类查找，并保留最具体类已经提供的值。
        while let Some(origin_type) = current {
            if !visited.insert(origin_type) {
                break;
            }
            if let Some(current_messages) = messages.get(&(origin_type, locale.clone())) {
                for (key, value) in current_messages {
                    combined.entry(key.clone()).or_insert_with(|| value.clone());
                }
            }
            current = parents.get(&origin_type).copied();
        }
        combined
    }

    /// 注册 Rust 类型对应的 origin 消息资源。
    ///
    /// Rust 没有 JVM `ClassLoader#getResourceAsStream`；宿主集成层在加载等价
    /// classpath 资源后通过此入口登记，解析器仍按 origin 与 Locale 缓存。
    pub(crate) fn register_origin_messages(origin: TypeId, locale: JavaLocale, messages: Messages) {
        write_lock(origin_messages()).insert((origin, locale), messages);
    }

    /// 登记 Rust origin 类型的直接父类型，复现 Java superclass 消息回退。
    pub(crate) fn register_origin_parent(
        origin: TypeId,
        parent: TypeId,
    ) -> MessageResolutionResult<()> {
        let mut parents = write_lock(origin_parents());
        if let Some(existing) = parents.get(&origin) {
            if *existing == parent {
                return Ok(());
            }
            return Err(Box::new(OriginRegistrationError::ConflictingParent));
        }
        let mut current = Some(parent);
        let mut visited = std::collections::HashSet::new();
        while let Some(candidate) = current {
            if candidate == origin {
                return Err(Box::new(OriginRegistrationError::Cycle));
            }
            if !visited.insert(candidate) {
                return Err(Box::new(OriginRegistrationError::Cycle));
            }
            current = parents.get(&candidate).copied();
        }
        parents.insert(origin, parent);
        Ok(())
    }

    /// 使用 Java `MessageFormat` 的索引占位符和引号规则格式化消息。
    pub(crate) fn format_message(
        locale: &JavaLocale,
        message: &JavaString,
        message_parameters: Option<&[Option<std::sync::Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<JavaString> {
        let units = message.as_utf16();
        if !units.contains(&u16::from(b'}')) && !units.contains(&u16::from(b'\'')) {
            return Ok(message.clone());
        }

        let parameters = message_parameters.unwrap_or(&[]);
        let mut result = Vec::with_capacity(units.len());
        let mut index = 0;
        let mut quoted = false;
        while index < units.len() {
            let unit = units[index];
            if unit == u16::from(b'\'') {
                if units.get(index + 1) == Some(&u16::from(b'\'')) {
                    result.push(u16::from(b'\''));
                    index += 2;
                    continue;
                }
                quoted = !quoted;
                index += 1;
                continue;
            }
            if unit == u16::from(b'{') && !quoted {
                let Some(end) = find_format_element_end(units, index + 1) else {
                    return Err(Box::new(MessageFormatError::UnmatchedBraces));
                };
                let element = JavaString::from_utf16(units[index + 1..end].to_vec());
                let formatted = format_message_element(&element, parameters, locale)?;
                result.extend_from_slice(formatted.as_utf16());
                index = end + 1;
                continue;
            }
            // `MessageFormat` 把未与格式元素配对的右花括号作为普通文本保留。
            result.push(unit);
            index += 1;
        }
        // Java MessageFormat 接受未闭合引号，并将其视为延续到模式末尾。
        Ok(JavaString::from_utf16(result))
    }

    fn compute_message_resource_names_from_base(
        resource_base_name: &str,
        locale: &JavaLocale,
    ) -> Result<Vec<String>, TemplateProcessingException> {
        let language = locale.get_language().to_string_lossy();
        if is_java_empty_or_whitespace(&language) {
            return Err(TemplateProcessingException::new(Some(format!(
                "Locale \"{locale}\" cannot be used as it does not specify a language."
            ))));
        }

        let country = locale.get_country().to_string_lossy();
        let variant = locale.get_variant().to_string_lossy();
        let mut resource_names = Vec::with_capacity(4);
        resource_names.push(format!("{resource_base_name}.properties"));
        resource_names.push(format!("{resource_base_name}_{language}.properties"));
        if !is_java_empty_or_whitespace(&country) {
            resource_names.push(format!(
                "{resource_base_name}_{language}_{country}.properties"
            ));
        }
        if !is_java_empty_or_whitespace(&variant) {
            resource_names.push(format!(
                "{resource_base_name}_{language}_{country}-{variant}.properties"
            ));
        }
        Ok(resource_names)
    }

    fn read_messages_resource(
        mut properties_reader: Box<dyn Read>,
    ) -> Result<Messages, TemplateInputException> {
        let mut bytes = Vec::new();
        properties_reader.read_to_end(&mut bytes).map_err(|error| {
            TemplateInputException::with_cause(
                Some("Exception loading messages file".to_owned()),
                error,
            )
        })?;
        let mut messages = Messages::new();
        java_properties::PropertiesIter::new_with_encoding(bytes.as_slice(), encoding_rs::UTF_8)
            .read_into(|key, value| {
                messages.insert(
                    JavaString::from_rust_str(&key),
                    JavaString::from_rust_str(&value),
                );
            })
            .map_err(|error| {
                TemplateInputException::with_cause(
                    Some("Exception loading messages file".to_owned()),
                    error,
                )
            })?;
        Ok(messages)
    }
}

#[derive(Debug, thiserror::Error)]
enum MessageFormatError {
    #[error("can't parse argument number: {0}")]
    ArgumentNumber(String),
    #[error("unknown format type: {0}")]
    UnknownFormatType(String),
    #[error("Unmatched braces in the pattern.")]
    UnmatchedBraces,
    #[error("Cannot format given Object as a Number")]
    NotANumber,
    #[error("Choice Pattern incorrect: {0}")]
    InvalidChoice(String),
    #[error("{0}")]
    ChoiceArrayIndex(String),
    #[error("{0}")]
    NumberFormatting(String),
}

#[derive(Debug, thiserror::Error)]
enum OriginRegistrationError {
    #[error("Origin parent metadata would create a cycle")]
    Cycle,
    #[error("Origin already has a different registered parent")]
    ConflictingParent,
}

fn format_message_element(
    element: &JavaString,
    parameters: &[Option<std::sync::Arc<TemplateValue>>],
    locale: &JavaLocale,
) -> MessageResolutionResult<JavaString> {
    let element_text = element.to_string_lossy();
    let mut sections = element_text.splitn(3, ',');
    let argument_text = sections.next().unwrap_or("").trim();
    let argument_index = argument_text.parse::<usize>().map_err(|_| {
        Box::new(MessageFormatError::ArgumentNumber(argument_text.to_owned()))
            as super::MessageResolutionError
    })?;
    let Some(parameter) = parameters.get(argument_index) else {
        let mut missing = Vec::with_capacity(element.len() + 2);
        missing.push(u16::from(b'{'));
        missing.extend_from_slice(element.as_utf16());
        missing.push(u16::from(b'}'));
        return Ok(JavaString::from_utf16(missing));
    };
    let value = parameter.as_deref();
    let Some(format_type) = sections.next().map(str::trim) else {
        return format_default_parameter(value, locale);
    };
    let format_style = sections.next().map(str::trim).unwrap_or("");

    match format_type {
        "" => format_default_parameter(value, locale),
        "number" => format_number_parameter(value, locale, format_style),
        "choice" => format_choice_parameter(value, locale, format_style, parameters),
        "date" | "time" => format_temporal_parameter(value, locale, format_type, format_style),
        unknown => Err(Box::new(MessageFormatError::UnknownFormatType(
            unknown.to_owned(),
        ))),
    }
}

fn format_default_parameter(
    value: Option<&TemplateValue>,
    locale: &JavaLocale,
) -> MessageResolutionResult<JavaString> {
    match value {
        Some(TemplateValue::Number(number)) => format_number(number, locale, NumberStyle::Default),
        Some(TemplateValue::Object(object)) if object.as_any().is::<crate::util::JavaDate>() => {
            format_temporal_parameter(value, locale, "", "")
        }
        Some(value) => Ok(value
            .to_java_string()
            .unwrap_or_else(|| JavaString::from_rust_str("null"))),
        None => Ok(JavaString::from_rust_str("null")),
    }
}

#[derive(Clone, Copy)]
enum NumberStyle {
    Default,
    Integer,
    Currency,
    Percent,
}

fn format_number_parameter(
    value: Option<&TemplateValue>,
    locale: &JavaLocale,
    style: &str,
) -> MessageResolutionResult<JavaString> {
    let Some(TemplateValue::Number(number)) = value else {
        return Err(Box::new(MessageFormatError::NotANumber));
    };
    let style = match style {
        "" => NumberStyle::Default,
        "integer" => NumberStyle::Integer,
        "currency" => NumberStyle::Currency,
        "percent" => NumberStyle::Percent,
        pattern => return format_decimal_pattern(number, locale, pattern),
    };
    format_number(number, locale, style)
}

fn format_decimal_pattern(
    number: &JavaNumber,
    locale: &JavaLocale,
    pattern: &str,
) -> MessageResolutionResult<JavaString> {
    let subpatterns = split_unquoted(pattern, ';');
    if subpatterns.len() > 2 {
        return malformed_decimal_pattern(pattern);
    }
    let positive = parse_decimal_subpattern(subpatterns[0], pattern)?;
    let negative = subpatterns
        .get(1)
        .map(|subpattern| parse_decimal_subpattern(subpattern, pattern))
        .transpose()?;
    let is_negative = number_is_negative(number);
    let selected = if is_negative {
        negative.as_ref().unwrap_or(&positive)
    } else {
        &positive
    };
    let prefix = if is_negative && negative.is_none() {
        format!("-{}", decimal_affix(selected.prefix, locale))
    } else {
        decimal_affix(selected.prefix, locale)
    };
    let suffix = decimal_affix(selected.suffix, locale);

    let absolute = absolute_number(number);
    let scaled = scale_number(&absolute, selected.multiplier);
    let formatted = if selected.scientific {
        format_scientific_number(&scaled, selected)?
    } else {
        format_fixed_decimal_number(&scaled, locale, selected)?
    };
    Ok(JavaString::from_rust_str(&format!(
        "{prefix}{formatted}{suffix}"
    )))
}

#[derive(Debug)]
struct DecimalSubpattern<'a> {
    prefix: &'a str,
    suffix: &'a str,
    min_integer_digits: usize,
    min_fraction_digits: usize,
    max_fraction_digits: usize,
    grouping: bool,
    multiplier: i32,
    scientific: bool,
    min_exponent_digits: usize,
}

fn parse_decimal_subpattern<'a>(
    subpattern: &'a str,
    full_pattern: &str,
) -> MessageResolutionResult<DecimalSubpattern<'a>> {
    let (first_digit, last_digit) =
        numeric_pattern_bounds(subpattern).ok_or_else(|| decimal_pattern_error(full_pattern))?;
    let prefix = &subpattern[..first_digit];
    let numeric_pattern = &subpattern[first_digit..last_digit];
    let suffix = &subpattern[last_digit..];
    let (mantissa_pattern, exponent_pattern) = numeric_pattern
        .split_once('E')
        .map_or((numeric_pattern, None), |(mantissa, exponent)| {
            (mantissa, Some(exponent))
        });
    if numeric_pattern.matches('E').count() > 1 {
        return malformed_decimal_pattern(full_pattern);
    }
    let (integer_pattern, fraction_pattern) = mantissa_pattern
        .split_once('.')
        .unwrap_or((mantissa_pattern, ""));
    if mantissa_pattern.matches('.').count() > 1
        || !integer_pattern
            .chars()
            .all(|character| matches!(character, '#' | '0' | ','))
        || !fraction_pattern
            .chars()
            .all(|character| matches!(character, '#' | '0'))
        || exponent_pattern.is_some_and(|exponent| {
            exponent.is_empty() || !exponent.chars().all(|character| character == '0')
        })
    {
        return malformed_decimal_pattern(full_pattern);
    }
    let min_integer_digits = integer_pattern
        .chars()
        .filter(|character| *character == '0')
        .count();
    let min_fraction_digits = fraction_pattern
        .chars()
        .filter(|character| *character == '0')
        .count();
    let max_fraction_digits = fraction_pattern
        .chars()
        .filter(|character| matches!(character, '0' | '#'))
        .count();
    let multiplier = if contains_unquoted(prefix, '%') || contains_unquoted(suffix, '%') {
        100
    } else if contains_unquoted(prefix, '‰') || contains_unquoted(suffix, '‰') {
        1000
    } else {
        1
    };
    Ok(DecimalSubpattern {
        prefix,
        suffix,
        min_integer_digits,
        min_fraction_digits,
        max_fraction_digits,
        grouping: integer_pattern.contains(','),
        multiplier,
        scientific: exponent_pattern.is_some(),
        min_exponent_digits: exponent_pattern.map_or(0, str::len),
    })
}

fn numeric_pattern_bounds(pattern: &str) -> Option<(usize, usize)> {
    let mut quoted = false;
    let mut first = None;
    let mut last = None;
    let characters = pattern.char_indices().collect::<Vec<_>>();
    let mut index = 0;
    while index < characters.len() {
        let (byte_index, character) = characters[index];
        if character == '\''
            && characters
                .get(index + 1)
                .is_some_and(|(_, next)| *next == '\'')
        {
            index += 2;
            continue;
        }
        if character == '\'' {
            quoted = !quoted;
        } else if !quoted && matches!(character, '#' | '0') {
            first.get_or_insert(byte_index);
            last = Some(byte_index + character.len_utf8());
        }
        index += 1;
    }
    first.zip(last)
}

fn format_fixed_decimal_number(
    number: &JavaNumber,
    locale: &JavaLocale,
    pattern: &DecimalSubpattern<'_>,
) -> MessageResolutionResult<String> {
    let mut formatted = match number {
        JavaNumber::Double(value) if value.is_nan() => "NaN".to_owned(),
        JavaNumber::Float(value) if value.is_nan() => "NaN".to_owned(),
        JavaNumber::Double(value) if value.is_infinite() => "∞".to_owned(),
        JavaNumber::Float(value) if value.is_infinite() => "∞".to_owned(),
        _ => NumberUtils::format(
            Some(number),
            Some(pattern.min_integer_digits.max(1) as i32),
            Some(if pattern.grouping {
                NumberPointType::Default
            } else {
                NumberPointType::None
            }),
            Some(pattern.max_fraction_digits as i32),
            Some(NumberPointType::Default),
            Some(locale),
        )
        .map_err(|error| {
            Box::new(MessageFormatError::NumberFormatting(error.to_string()))
                as super::MessageResolutionError
        })?
        .expect("non-null number")
        .to_string_lossy(),
    };
    trim_optional_fraction(
        &mut formatted,
        pattern.min_fraction_digits,
        pattern.max_fraction_digits,
        locale,
    );
    Ok(formatted)
}

fn format_scientific_number(
    number: &JavaNumber,
    pattern: &DecimalSubpattern<'_>,
) -> MessageResolutionResult<String> {
    let value = number_as_f64(number);
    if value.is_nan() {
        return Ok("NaN".to_owned());
    }
    if value.is_infinite() {
        return Ok("∞".to_owned());
    }
    let precision = pattern.max_fraction_digits;
    let raw = format!("{value:.precision$E}");
    let (mut mantissa, exponent) = raw
        .split_once('E')
        .ok_or_else(|| decimal_pattern_error("scientific"))?;
    while mantissa.contains('.')
        && mantissa.ends_with('0')
        && mantissa
            .split_once('.')
            .map_or(0, |(_, fraction)| fraction.len())
            > pattern.min_fraction_digits
    {
        mantissa = &mantissa[..mantissa.len() - 1];
    }
    if mantissa.ends_with('.') {
        mantissa = &mantissa[..mantissa.len() - 1];
    }
    let exponent_value = exponent.parse::<i32>().map_err(|_| {
        Box::new(MessageFormatError::NumberFormatting(raw.clone())) as super::MessageResolutionError
    })?;
    let exponent_sign = if exponent_value < 0 { "-" } else { "" };
    let exponent_digits = exponent_value.unsigned_abs().to_string();
    Ok(format!(
        "{mantissa}E{exponent_sign}{:0>width$}",
        exponent_digits,
        width = pattern.min_exponent_digits.max(1)
    ))
}

fn trim_optional_fraction(
    formatted: &mut String,
    min_fraction_digits: usize,
    max_fraction_digits: usize,
    locale: &JavaLocale,
) {
    if max_fraction_digits <= min_fraction_digits {
        return;
    }
    let decimal_separator = if locale_uses_decimal_comma(locale) {
        ','
    } else {
        '.'
    };
    if let Some(decimal_index) = formatted.rfind(decimal_separator) {
        while formatted.ends_with('0')
            && formatted.len() - decimal_index - decimal_separator.len_utf8() > min_fraction_digits
        {
            formatted.pop();
        }
        if min_fraction_digits == 0 && formatted.ends_with(decimal_separator) {
            formatted.pop();
        }
    }
}

fn decimal_affix(affix: &str, locale: &JavaLocale) -> String {
    let currency = NumberUtils::format_currency(Some(&JavaNumber::Integer(0)), Some(locale))
        .ok()
        .flatten()
        .map(|formatted| {
            formatted
                .to_string_lossy()
                .trim_matches(|character: char| {
                    character.is_ascii_digit()
                        || matches!(character, '.' | ',' | '-' | '\u{a0}' | ' ')
                })
                .to_owned()
        })
        .unwrap_or_else(|| "¤".to_owned());
    let mut result = String::new();
    let characters = affix.chars().collect::<Vec<_>>();
    let mut index = 0;
    let mut quoted = false;
    while index < characters.len() {
        if characters[index] == '\'' && characters.get(index + 1) == Some(&'\'') {
            result.push('\'');
            index += 2;
            continue;
        }
        if characters[index] == '\'' {
            quoted = !quoted;
            index += 1;
            continue;
        }
        if !quoted && characters[index] == '¤' {
            result.push_str(&currency);
        } else {
            result.push(characters[index]);
        }
        index += 1;
    }
    result
}

fn split_unquoted(value: &str, delimiter: char) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut quoted = false;
    let characters = value.char_indices().collect::<Vec<_>>();
    let mut index = 0;
    while index < characters.len() {
        let (byte_index, character) = characters[index];
        if character == '\''
            && characters
                .get(index + 1)
                .is_some_and(|(_, next)| *next == '\'')
        {
            index += 2;
            continue;
        }
        if character == '\'' {
            quoted = !quoted;
        } else if !quoted && character == delimiter {
            result.push(&value[start..byte_index]);
            start = byte_index + character.len_utf8();
        }
        index += 1;
    }
    result.push(&value[start..]);
    result
}

fn contains_unquoted(value: &str, target: char) -> bool {
    let mut quoted = false;
    let characters = value.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < characters.len() {
        if characters[index] == '\'' && characters.get(index + 1) == Some(&'\'') {
            index += 2;
            continue;
        }
        if characters[index] == '\'' {
            quoted = !quoted;
        } else if !quoted && characters[index] == target {
            return true;
        }
        index += 1;
    }
    false
}

fn number_is_negative(number: &JavaNumber) -> bool {
    match number {
        JavaNumber::BigDecimal(value) => value.unscaled_value().is_negative(),
        JavaNumber::BigInteger(value) => value.is_negative(),
        JavaNumber::Byte(value) => *value < 0,
        JavaNumber::Short(value) => *value < 0,
        JavaNumber::Integer(value) => *value < 0,
        JavaNumber::Long(value) => *value < 0,
        JavaNumber::Float(value) => value.is_sign_negative(),
        JavaNumber::Double(value) => value.is_sign_negative(),
        JavaNumber::Other { double_value, .. } => double_value.is_sign_negative(),
    }
}

fn absolute_number(number: &JavaNumber) -> JavaNumber {
    match number {
        JavaNumber::BigDecimal(value) => JavaNumber::BigDecimal(JavaBigDecimal::from_unscaled(
            value.unscaled_value().abs(),
            value.scale(),
        )),
        JavaNumber::BigInteger(value) => JavaNumber::BigInteger(value.abs()),
        JavaNumber::Byte(value) => JavaNumber::Integer(i32::from(*value).abs()),
        JavaNumber::Short(value) => JavaNumber::Integer(i32::from(*value).abs()),
        JavaNumber::Integer(value) => JavaNumber::Long(i64::from(*value).abs()),
        JavaNumber::Long(value) => {
            if *value == i64::MIN {
                JavaNumber::BigInteger(BigInt::from(*value).abs())
            } else {
                JavaNumber::Long(value.abs())
            }
        }
        JavaNumber::Float(value) => JavaNumber::Float(value.abs()),
        JavaNumber::Double(value) => JavaNumber::Double(value.abs()),
        JavaNumber::Other {
            class_name,
            double_value,
        } => JavaNumber::Other {
            class_name: class_name.clone(),
            double_value: double_value.abs(),
        },
    }
}

fn scale_number(number: &JavaNumber, multiplier: i32) -> JavaNumber {
    if multiplier == 1 {
        return number.clone();
    }
    match number_as_decimal(number) {
        Some(value) => JavaNumber::BigDecimal(JavaBigDecimal::from_unscaled(
            value.unscaled_value() * BigInt::from(multiplier),
            value.scale(),
        )),
        None => JavaNumber::Double(number_as_f64(number) * f64::from(multiplier)),
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
        JavaNumber::Float(value) if value.is_finite() => {
            JavaBigDecimal::parse(&value.to_string()).ok()
        }
        JavaNumber::Double(value) if value.is_finite() => {
            JavaBigDecimal::parse(&value.to_string()).ok()
        }
        JavaNumber::Other { double_value, .. } if double_value.is_finite() => {
            JavaBigDecimal::parse(&double_value.to_string()).ok()
        }
        JavaNumber::Float(_) | JavaNumber::Double(_) | JavaNumber::Other { .. } => None,
    }
}

fn decimal_pattern_error(pattern: &str) -> super::MessageResolutionError {
    Box::new(MessageFormatError::NumberFormatting(format!(
        "Malformed pattern \"{pattern}\""
    )))
}

fn malformed_decimal_pattern<T>(pattern: &str) -> MessageResolutionResult<T> {
    Err(decimal_pattern_error(pattern))
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

fn format_number(
    number: &JavaNumber,
    locale: &JavaLocale,
    style: NumberStyle,
) -> MessageResolutionResult<JavaString> {
    if let Some(non_finite) = non_finite_number_text(number) {
        let formatted = match style {
            NumberStyle::Currency => {
                let symbol = currency_symbol(locale);
                if currency_symbol_after(locale) {
                    format!("{non_finite}\u{a0}{symbol}")
                } else {
                    format!("{symbol}{non_finite}")
                }
            }
            NumberStyle::Percent => format!("{non_finite}%"),
            NumberStyle::Default | NumberStyle::Integer => non_finite.to_owned(),
        };
        return Ok(JavaString::from_rust_str(&formatted));
    }
    let formatted = match style {
        NumberStyle::Currency => NumberUtils::format_currency(Some(number), Some(locale)),
        NumberStyle::Percent => {
            NumberUtils::format_percent(Some(number), Some(1), Some(0), Some(locale))
        }
        NumberStyle::Integer => NumberUtils::format(
            Some(number),
            Some(1),
            Some(NumberPointType::Default),
            Some(0),
            Some(NumberPointType::Default),
            Some(locale),
        ),
        NumberStyle::Default => {
            let rounded = number_as_f64(number);
            let fraction_digits = default_fraction_digits(rounded);
            NumberUtils::format(
                Some(number),
                Some(1),
                Some(NumberPointType::Default),
                Some(fraction_digits),
                Some(NumberPointType::Default),
                Some(locale),
            )
        }
    }
    .map_err(|error| {
        Box::new(MessageFormatError::NumberFormatting(error.to_string()))
            as super::MessageResolutionError
    })?;
    Ok(formatted.unwrap_or_else(|| JavaString::from_rust_str("null")))
}

fn non_finite_number_text(number: &JavaNumber) -> Option<&'static str> {
    let value = match number {
        JavaNumber::Float(value) => f64::from(*value),
        JavaNumber::Double(value)
        | JavaNumber::Other {
            double_value: value,
            ..
        } => *value,
        _ => return None,
    };
    if value.is_nan() {
        Some("NaN")
    } else if value == f64::INFINITY {
        Some("∞")
    } else if value == f64::NEG_INFINITY {
        Some("-∞")
    } else {
        None
    }
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

fn currency_symbol_after(locale: &JavaLocale) -> bool {
    locale_uses_decimal_comma(locale)
}

fn default_fraction_digits(value: f64) -> i32 {
    if !value.is_finite() {
        return 0;
    }
    let rounded = (value * 1000.0).round_ties_even() / 1000.0;
    let rendered = format!("{rounded:.3}");
    rendered.split_once('.').map_or(0, |(_, fraction)| {
        fraction.trim_end_matches('0').len() as i32
    })
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

fn format_choice_parameter(
    value: Option<&TemplateValue>,
    locale: &JavaLocale,
    pattern: &str,
    parameters: &[Option<std::sync::Arc<TemplateValue>>],
) -> MessageResolutionResult<JavaString> {
    let Some(TemplateValue::Number(number)) = value else {
        return Err(Box::new(MessageFormatError::NotANumber));
    };
    let number = number_as_f64(number);
    let mut selected = None;
    for (alternative_index, alternative) in
        split_choice_alternatives(pattern).into_iter().enumerate()
    {
        let Some((limit, comparator, text)) = parse_choice_alternative(alternative) else {
            return if alternative_index == 0 {
                Err(Box::new(MessageFormatError::ChoiceArrayIndex(
                    "Index 0 out of bounds for length 0".to_owned(),
                )))
            } else {
                Err(Box::new(MessageFormatError::InvalidChoice(
                    pattern.to_owned(),
                )))
            };
        };
        if alternative_index == 0 {
            // `ChoiceFormat` 对低于首个 limit 的值仍返回第一个子格式。
            selected = Some(text);
        }
        let matches = match comparator {
            '#' | '≤' => number >= limit,
            '<' => number > limit,
            _ => false,
        };
        if matches {
            selected = Some(text);
        }
    }
    let selected = selected.unwrap_or("");
    StandardMessageResolutionUtils::format_message(
        locale,
        &JavaString::from_rust_str(selected),
        Some(parameters),
    )
}

fn split_choice_alternatives(pattern: &str) -> Vec<&str> {
    let mut alternatives = Vec::new();
    let mut start = 0;
    let mut quoted = false;
    for (index, character) in pattern.char_indices() {
        if character == '\'' {
            quoted = !quoted;
        } else if character == '|' && !quoted {
            alternatives.push(&pattern[start..index]);
            start = index + character.len_utf8();
        }
    }
    alternatives.push(&pattern[start..]);
    alternatives
}

fn parse_choice_alternative(alternative: &str) -> Option<(f64, char, &str)> {
    let (index, comparator) = alternative
        .char_indices()
        .find(|(_, character)| matches!(character, '#' | '<' | '≤'))?;
    let limit_text = alternative[..index].trim();
    let limit = match limit_text {
        "∞" | "+∞" => f64::INFINITY,
        "-∞" => f64::NEG_INFINITY,
        _ => limit_text.parse().ok()?,
    };
    Some((
        limit,
        comparator,
        &alternative[index + comparator.len_utf8()..],
    ))
}

fn format_temporal_parameter(
    value: Option<&TemplateValue>,
    locale: &JavaLocale,
    format_type: &str,
    style: &str,
) -> MessageResolutionResult<JavaString> {
    let Some(TemplateValue::Object(object)) = value else {
        return Err(Box::new(MessageFormatError::NumberFormatting(
            "Cannot format given Object as a Date".to_owned(),
        )));
    };
    let Some(date) = object.as_any().downcast_ref::<crate::util::JavaDate>() else {
        return Err(Box::new(MessageFormatError::NumberFormatting(
            "Cannot format given Object as a Date".to_owned(),
        )));
    };
    let pattern = temporal_pattern(locale, format_type, style)?;
    crate::util::DateUtils::format(
        Some(date),
        Some(&JavaString::from_rust_str(&pattern)),
        Some(locale),
    )
    .map_err(|error| Box::new(error) as super::MessageResolutionError)
    .map(|formatted| formatted.unwrap_or_else(|| JavaString::from_rust_str("null")))
}

fn temporal_pattern(
    locale: &JavaLocale,
    format_type: &str,
    style: &str,
) -> MessageResolutionResult<String> {
    let language = locale.get_language().to_string_lossy();
    let date = match (language.as_str(), style) {
        ("en", "short") => "M/d/yy",
        ("en", "" | "medium") => "MMM d, yyyy",
        ("en", "long") => "MMMM d, yyyy",
        ("en", "full") => "EEEE, MMMM d, yyyy",
        ("de", "short") => "dd.MM.yy",
        ("de", "" | "medium") => "dd.MM.yyyy",
        ("de", "long") => "d. MMMM yyyy",
        ("de", "full") => "EEEE, d. MMMM yyyy",
        ("fr", "short") => "dd/MM/y",
        ("fr", "" | "medium") => "d MMM yyyy",
        ("fr", "long") => "d MMMM yyyy",
        ("fr", "full") => "EEEE d MMMM yyyy",
        ("ja", "short" | "" | "medium") => "yyyy/MM/dd",
        ("ja", "long") => "yyyy年M月d日",
        ("ja", "full") => "yyyy年M月d日EEEE",
        (_, "short") => "dd/MM/yy",
        (_, "" | "medium") => "dd MMM yyyy",
        (_, "long" | "full") => "d MMMM yyyy",
        _ => style,
    };
    let time = match (language.as_str(), style) {
        ("en", "short") => "h:mm\u{202f}a",
        ("en", "" | "medium") => "h:mm:ss\u{202f}a",
        ("en", "long") => "h:mm:ss\u{202f}a z",
        ("en", "full") => "h:mm:ss\u{202f}a zzzz",
        (_, "short") => "HH:mm",
        (_, "" | "medium") => "HH:mm:ss",
        (_, "long") => "HH:mm:ss z",
        (_, "full") => "HH:mm:ss zzzz",
        _ => style,
    };
    Ok(match format_type {
        "time" => time,
        "date" => date,
        "" if language == "en" => "M/d/yy, h:mm\u{202f}a",
        "" => "dd/MM/yy, HH:mm",
        _ => date,
    }
    .to_owned())
}

fn is_java_empty_or_whitespace(value: &str) -> bool {
    value.chars().all(|character| {
        character == ' '
            || (character.is_whitespace()
                && !matches!(character, '\u{00a0}' | '\u{2007}' | '\u{202f}'))
    })
}

fn find_format_element_end(units: &[u16], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut index = start;
    while index < units.len() {
        let unit = units[index];
        if unit == u16::from(b'\'') && units.get(index + 1) == Some(&u16::from(b'\'')) {
            index += 2;
            continue;
        }
        if unit == u16::from(b'\'') {
            quoted = !quoted;
        } else if !quoted && unit == u16::from(b'{') {
            depth += 1;
        } else if !quoted && unit == u16::from(b'}') {
            if depth == 0 {
                return Some(index);
            }
            depth -= 1;
        }
        index += 1;
    }
    None
}

fn origin_messages() -> &'static RwLock<OriginMessages> {
    ORIGIN_MESSAGES.get_or_init(|| RwLock::new(HashMap::new()))
}

fn origin_parents() -> &'static RwLock<OriginParents> {
    ORIGIN_PARENTS.get_or_init(|| RwLock::new(HashMap::new()))
}

fn read_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use std::collections::HashMap;

    use crate::util::{JavaLocale, JavaString};

    use super::StandardMessageResolutionUtils;

    struct Parent;
    struct Child;
    struct OtherParent;
    struct CycleA;
    struct CycleB;

    #[test]
    fn origin_messages_use_specific_class_before_registered_parent() {
        let locale = JavaLocale::new(
            JavaString::from_rust_str("en-US"),
            JavaString::from_rust_str("US"),
        );
        let parent = TypeId::of::<Parent>();
        let child = TypeId::of::<Child>();
        StandardMessageResolutionUtils::register_origin_messages(
            parent,
            locale.clone(),
            HashMap::from([
                (
                    JavaString::from_rust_str("parent"),
                    JavaString::from_rust_str("parent-value"),
                ),
                (
                    JavaString::from_rust_str("same"),
                    JavaString::from_rust_str("parent-value"),
                ),
            ]),
        );
        StandardMessageResolutionUtils::register_origin_messages(
            child,
            locale.clone(),
            HashMap::from([(
                JavaString::from_rust_str("same"),
                JavaString::from_rust_str("child-value"),
            )]),
        );
        StandardMessageResolutionUtils::register_origin_parent(child, parent)
            .expect("valid parent");

        let resolved = StandardMessageResolutionUtils::resolve_messages_for_origin(child, &locale);
        assert_eq!(
            resolved.get(&JavaString::from_rust_str("parent")),
            Some(&JavaString::from_rust_str("parent-value"))
        );
        assert_eq!(
            resolved.get(&JavaString::from_rust_str("same")),
            Some(&JavaString::from_rust_str("child-value"))
        );
    }

    #[test]
    fn origin_parent_registration_is_idempotent_and_rejects_impossible_hierarchies() {
        let child = TypeId::of::<Child>();
        let parent = TypeId::of::<Parent>();
        StandardMessageResolutionUtils::register_origin_parent(child, parent)
            .expect("same registration is idempotent");
        let conflicting = StandardMessageResolutionUtils::register_origin_parent(
            child,
            TypeId::of::<OtherParent>(),
        )
        .expect_err("Java class cannot have two direct superclasses");
        assert_eq!(
            conflicting.to_string(),
            "Origin already has a different registered parent"
        );

        let cycle_a = TypeId::of::<CycleA>();
        let cycle_b = TypeId::of::<CycleB>();
        StandardMessageResolutionUtils::register_origin_parent(cycle_a, cycle_b)
            .expect("first edge");
        let cycle = StandardMessageResolutionUtils::register_origin_parent(cycle_b, cycle_a)
            .expect_err("Java class hierarchy cannot contain a cycle");
        assert_eq!(
            cycle.to_string(),
            "Origin parent metadata would create a cycle"
        );
    }
}
