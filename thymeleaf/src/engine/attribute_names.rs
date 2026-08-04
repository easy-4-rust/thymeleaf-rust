use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::templatemode::TemplateMode;
use crate::util::{Utf16String, case_fold_unit};

use super::{
    AttributeName, AttributeNameError, HTMLAttributeName, TextAttributeName, XMLAttributeName,
};

static HTML_REPOSITORY: OnceLock<RwLock<AttributeNamesRepository>> = OnceLock::new();
static XML_REPOSITORY: OnceLock<RwLock<AttributeNamesRepository>> = OnceLock::new();
static TEXT_REPOSITORY: OnceLock<RwLock<AttributeNamesRepository>> = OnceLock::new();

/// `AttributeNames` 返回的具体名称子类。
#[derive(Clone)]
/// 对应 Java 语义：`AttributeNames` 的 Rust 侧类型 `AttributeNameValue`。
pub enum AttributeNameValue {
    /// HTML 名称。
    Html(Arc<HTMLAttributeName>),
    /// XML 名称。
    Xml(Arc<XMLAttributeName>),
    /// TEXT/JAVASCRIPT/CSS 名称。
    Text(Arc<TextAttributeName>),
}

impl AttributeNameValue {
    /// 返回统一的 `AttributeName` 基类视图。
    #[must_use]
    /// 对应 Java 语义：`AttributeNames` 的 `as_attribute_name` 行为（Rust 侧辅助/私有路径）。
    pub fn as_attribute_name(&self) -> &AttributeName {
        match self {
            Self::Html(value) => value.as_attribute_name(),
            Self::Xml(value) => value.as_attribute_name(),
            Self::Text(value) => value.as_attribute_name(),
        }
    }
}

/// 属性名称规范化或 repository 访问错误。
#[derive(Clone, Debug, Eq, PartialEq)]
/// 对应 Java 语义：`AttributeNames` 的 Rust 侧类型 `AttributeNamesError`。
pub enum AttributeNamesError {
    /// 指定参数违反公开方法的 null/空规则。
    IllegalArgument(&'static str),
    /// UTF-16 buffer 范围非法。
    StringIndexOutOfBounds {
        /// 起始位置。
        offset: i32,
        /// 长度。
        length: i32,
        /// buffer 长度。
        buffer_length: usize,
    },
    /// RAW 不是结构化属性名称模式。
    UnknownTemplateMode(TemplateMode),
    /// 具体名称对象构造失败。
    AttributeName(AttributeNameError),
    /// 插入第二个 complete alias 时遇到已有 repository 项。
    RepositoryAliasCollision,
}

impl AttributeNamesError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn class_name(&self) -> &'static str {
        match self {
            Self::IllegalArgument(_)
            | Self::UnknownTemplateMode(_)
            | Self::AttributeName(AttributeNameError::InvalidAttributeName) => {
                "java.lang.IllegalArgumentException"
            }
            Self::StringIndexOutOfBounds { .. } => "java.lang.StringIndexOutOfBoundsException",
            Self::AttributeName(error) => error.class_name(),
            Self::RepositoryAliasCollision => "java.lang.IndexOutOfBoundsException",
        }
    }
}

impl Display for AttributeNamesError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalArgument(message) => formatter.write_str(message),
            Self::StringIndexOutOfBounds {
                offset,
                length,
                buffer_length,
            } => write!(
                formatter,
                "offset {offset}, count {length}, length {buffer_length}"
            ),
            Self::UnknownTemplateMode(mode) => {
                write!(formatter, "Unknown template mode '{mode}'")
            }
            Self::AttributeName(error) => Display::fmt(error, formatter),
            Self::RepositoryAliasCollision => {
                formatter.write_str("repository alias already exists")
            }
        }
    }
}

impl Error for AttributeNamesError {}

impl From<AttributeNameError> for AttributeNamesError {
    fn from(value: AttributeNameError) -> Self {
        Self::AttributeName(value)
    }
}

/// 按 TemplateMode 规范化并复用属性名称的线程安全入口。
///
/// 对应 Java: `org.thymeleaf.engine.AttributeNames`。
///
/// HTML repository 同时索引 `prefix:name` 与 `data-prefix-name`；XML 和文本
/// repository 保持大小写敏感。重复查询返回同一 `Arc`。
pub struct AttributeNames;

impl AttributeNames {
    /// 从 UTF-16 buffer 子范围解析任意结构化模板模式的属性名。
    ///
    /// # 错误
    ///
    /// null、空范围、负数、越界、RAW 模式或具体名称校验失败时返回 Java 对应错误。
    /// 对应 Java 语义：`AttributeNames` 的 `for_name_buffer` 行为（Rust 侧辅助/私有路径）。
    pub fn for_name_buffer(
        template_mode: Option<TemplateMode>,
        buffer: Option<&[u16]>,
        offset: i32,
        length: i32,
    ) -> Result<AttributeNameValue, AttributeNamesError> {
        let mode = require_mode(template_mode)?;
        if mode == TemplateMode::RAW {
            return Err(AttributeNamesError::UnknownTemplateMode(mode));
        }
        let text = checked_buffer(buffer, offset, length)?;
        Self::for_name(Some(mode), Some(&Utf16String::from_utf16(text.to_vec())))
    }

    /// 从完整 Java String 解析任意结构化模板模式的属性名。
    /// 对应 Java: `AttributeNames#forName()`。
    pub fn for_name(
        template_mode: Option<TemplateMode>,
        attribute_name: Option<&Utf16String>,
    ) -> Result<AttributeNameValue, AttributeNamesError> {
        let mode = require_mode(template_mode)?;
        match mode {
            TemplateMode::HTML => Self::for_html_name(attribute_name).map(AttributeNameValue::Html),
            TemplateMode::XML => Self::for_xml_name(attribute_name).map(AttributeNameValue::Xml),
            mode if mode.is_text() => {
                Self::for_text_name(attribute_name).map(AttributeNameValue::Text)
            }
            mode => Err(AttributeNamesError::UnknownTemplateMode(mode)),
        }
    }

    /// 从显式 prefix 与本地名解析任意结构化模板模式的属性名。
    /// 对应 Java 语义：`AttributeNames` 的 `for_name_with_prefix` 行为（Rust 侧辅助/私有路径）。
    pub fn for_name_with_prefix(
        template_mode: Option<TemplateMode>,
        prefix: Option<&Utf16String>,
        attribute_name: Option<&Utf16String>,
    ) -> Result<AttributeNameValue, AttributeNamesError> {
        let mode = require_mode(template_mode)?;
        match mode {
            TemplateMode::HTML => Self::for_html_name_with_prefix(prefix, attribute_name)
                .map(AttributeNameValue::Html),
            TemplateMode::XML => {
                Self::for_xml_name_with_prefix(prefix, attribute_name).map(AttributeNameValue::Xml)
            }
            mode if mode.is_text() => Self::for_text_name_with_prefix(prefix, attribute_name)
                .map(AttributeNameValue::Text),
            mode => Err(AttributeNamesError::UnknownTemplateMode(mode)),
        }
    }

    /// 解析并缓存文本模式属性名。
    /// 对应 Java: `AttributeNames#forTextName()`。
    pub fn for_text_name(
        attribute_name: Option<&Utf16String>,
    ) -> Result<Arc<TextAttributeName>, AttributeNamesError> {
        let attribute_name = require_non_blank_name(attribute_name)?;
        match repository_get_or_store(TemplateMode::TEXT, attribute_name, || {
            build_text(attribute_name)
        })? {
            AttributeNameValue::Text(value) => Ok(value),
            _ => unreachable!("text repository contains only text attributes"),
        }
    }

    /// 解析并缓存 XML 属性名。
    /// 对应 Java 语义：`AttributeNames` 的 `for_xml_name` 行为（Rust 侧辅助/私有路径）。
    pub fn for_xml_name(
        attribute_name: Option<&Utf16String>,
    ) -> Result<Arc<XMLAttributeName>, AttributeNamesError> {
        let attribute_name = require_non_blank_name(attribute_name)?;
        match repository_get_or_store(TemplateMode::XML, attribute_name, || {
            build_xml(attribute_name)
        })? {
            AttributeNameValue::Xml(value) => Ok(value),
            _ => unreachable!("xml repository contains only xml attributes"),
        }
    }

    /// 解析并缓存 HTML 属性名。
    /// 对应 Java 语义：`AttributeNames` 的 `for_html_name` 行为（Rust 侧辅助/私有路径）。
    pub fn for_html_name(
        attribute_name: Option<&Utf16String>,
    ) -> Result<Arc<HTMLAttributeName>, AttributeNamesError> {
        let attribute_name = require_non_blank_name(attribute_name)?;
        match repository_get_or_store(TemplateMode::HTML, attribute_name, || {
            build_html(attribute_name)
        })? {
            AttributeNameValue::Html(value) => Ok(value),
            _ => unreachable!("html repository contains only html attributes"),
        }
    }

    /// 使用显式 prefix 解析文本模式属性名。
    /// 对应 Java 语义：`AttributeNames` 的 `for_text_name_with_prefix` 行为（Rust 侧辅助/私有路径）。
    pub fn for_text_name_with_prefix(
        prefix: Option<&Utf16String>,
        attribute_name: Option<&Utf16String>,
    ) -> Result<Arc<TextAttributeName>, AttributeNamesError> {
        let attribute_name = require_non_blank_name(attribute_name)?;
        if !has_non_blank_prefix(prefix) {
            return Self::for_text_name(Some(attribute_name));
        }
        let lookup = namespaced(prefix.expect("non-blank prefix"), attribute_name);
        match repository_get_or_store(TemplateMode::TEXT, &lookup, || {
            Ok(AttributeNameValue::Text(Arc::new(
                TextAttributeName::for_name(prefix.cloned(), Some(attribute_name.clone()))?,
            )))
        })? {
            AttributeNameValue::Text(value) => Ok(value),
            _ => unreachable!("text repository contains only text attributes"),
        }
    }

    /// 使用显式 prefix 解析 XML 属性名。
    /// 对应 Java 语义：`AttributeNames` 的 `for_xml_name_with_prefix` 行为（Rust 侧辅助/私有路径）。
    pub fn for_xml_name_with_prefix(
        prefix: Option<&Utf16String>,
        attribute_name: Option<&Utf16String>,
    ) -> Result<Arc<XMLAttributeName>, AttributeNamesError> {
        let attribute_name = require_non_blank_name(attribute_name)?;
        if !has_non_blank_prefix(prefix) {
            return Self::for_xml_name(Some(attribute_name));
        }
        let lookup = namespaced(prefix.expect("non-blank prefix"), attribute_name);
        match repository_get_or_store(TemplateMode::XML, &lookup, || {
            Ok(AttributeNameValue::Xml(Arc::new(
                XMLAttributeName::for_name(prefix.cloned(), Some(attribute_name.clone()))?,
            )))
        })? {
            AttributeNameValue::Xml(value) => Ok(value),
            _ => unreachable!("xml repository contains only xml attributes"),
        }
    }

    /// 使用显式 prefix 解析 HTML 属性名。
    /// 对应 Java 语义：`AttributeNames` 的 `for_html_name_with_prefix` 行为（Rust 侧辅助/私有路径）。
    pub fn for_html_name_with_prefix(
        prefix: Option<&Utf16String>,
        attribute_name: Option<&Utf16String>,
    ) -> Result<Arc<HTMLAttributeName>, AttributeNamesError> {
        let attribute_name = require_non_blank_name(attribute_name)?;
        if !has_non_blank_prefix(prefix) {
            return Self::for_html_name(Some(attribute_name));
        }
        let lookup = namespaced(prefix.expect("non-blank prefix"), attribute_name);
        match repository_get_or_store(TemplateMode::HTML, &lookup, || {
            Ok(AttributeNameValue::Html(Arc::new(
                HTMLAttributeName::for_name(prefix.cloned(), Some(attribute_name.clone()))?,
            )))
        })? {
            AttributeNameValue::Html(value) => Ok(value),
            _ => unreachable!("html repository contains only html attributes"),
        }
    }
}

struct AttributeNamesRepository {
    values: HashMap<Vec<u16>, AttributeNameValue>,
}

fn repository_get_or_store(
    mode: TemplateMode,
    lookup: &Utf16String,
    builder: impl FnOnce() -> Result<AttributeNameValue, AttributeNamesError>,
) -> Result<AttributeNameValue, AttributeNamesError> {
    let repository = repository(mode);
    let key = repository_key(mode, lookup);
    if let Some(value) = read_recovering_poison(repository).values.get(&key) {
        return Ok(value.clone());
    }
    let mut repository = write_recovering_poison(repository);
    if let Some(value) = repository.values.get(&key) {
        return Ok(value.clone());
    }
    let value = builder()?;
    let names = value.as_attribute_name().get_complete_attribute_names();
    let names = read_recovering_poison(&names).clone();
    let mut keys = Vec::with_capacity(names.len());
    for name in names.into_iter().flatten() {
        let alias = repository_key(mode, &name);
        // 对应 Java `AttributeNamesRepository` 的首注册者胜语义：任何 complete name
        // 键已被不同对象占用时，返回既有绑定（Java 读路径的 short-circuit），不报错
        // 也不覆盖——Rust 侧以 keep-first 代替 Java 的重复键崩溃。
        if let Some(existing) = repository.values.get(&alias) {
            return Ok(existing.clone());
        }
        keys.push(alias);
    }
    for alias in keys {
        repository.values.insert(alias, value.clone());
    }
    Ok(value)
}

fn repository(mode: TemplateMode) -> &'static RwLock<AttributeNamesRepository> {
    let slot = match mode {
        TemplateMode::HTML => &HTML_REPOSITORY,
        TemplateMode::XML => &XML_REPOSITORY,
        _ => &TEXT_REPOSITORY,
    };
    slot.get_or_init(|| {
        RwLock::new(AttributeNamesRepository {
            values: HashMap::with_capacity(500),
        })
    })
}

fn repository_key(mode: TemplateMode, value: &Utf16String) -> Vec<u16> {
    if mode.is_case_sensitive() {
        value.as_utf16().to_vec()
    } else {
        value
            .as_utf16()
            .iter()
            .map(|unit| case_fold_unit(*unit))
            .collect()
    }
}

fn build_text(name: &Utf16String) -> Result<AttributeNameValue, AttributeNamesError> {
    let (prefix, local) = split_colon(name);
    Ok(AttributeNameValue::Text(Arc::new(
        TextAttributeName::for_name(prefix, Some(local))?,
    )))
}

fn build_xml(name: &Utf16String) -> Result<AttributeNameValue, AttributeNamesError> {
    let (prefix, local) = split_colon(name);
    Ok(AttributeNameValue::Xml(Arc::new(
        XMLAttributeName::for_name(prefix, Some(local))?,
    )))
}

fn build_html(name: &Utf16String) -> Result<AttributeNameValue, AttributeNamesError> {
    let units = name.as_utf16();
    let (prefix, local) = if starts_ascii_ignore_case(units, "data-") {
        match units[5..]
            .iter()
            .position(|unit| *unit == u16::from(b'-'))
            .map(|index| index + 5)
        {
            Some(5) | None => (None, name.clone()),
            Some(index) => (
                Some(Utf16String::from_utf16(units[5..index].to_vec())),
                Utf16String::from_utf16(units[index + 1..].to_vec()),
            ),
        }
    } else {
        match units.iter().position(|unit| matches!(*unit, 0x3a | 0x2d)) {
            Some(0) | None => (None, name.clone()),
            Some(index) if units[index] == u16::from(b'-') => (None, name.clone()),
            Some(index) => {
                let candidate = &units[..=index];
                if equals_ascii_ignore_case(candidate, "xml:")
                    || equals_ascii_ignore_case(candidate, "xmlns:")
                {
                    (None, name.clone())
                } else {
                    (
                        Some(Utf16String::from_utf16(units[..index].to_vec())),
                        Utf16String::from_utf16(units[index + 1..].to_vec()),
                    )
                }
            }
        }
    };
    Ok(AttributeNameValue::Html(Arc::new(
        HTMLAttributeName::for_name(prefix, Some(local))?,
    )))
}

fn split_colon(name: &Utf16String) -> (Option<Utf16String>, Utf16String) {
    let units = name.as_utf16();
    match units.iter().position(|unit| *unit == u16::from(b':')) {
        Some(0) | None => (None, name.clone()),
        Some(index) => (
            Some(Utf16String::from_utf16(units[..index].to_vec())),
            Utf16String::from_utf16(units[index + 1..].to_vec()),
        ),
    }
}

fn require_mode(mode: Option<TemplateMode>) -> Result<TemplateMode, AttributeNamesError> {
    mode.ok_or(AttributeNamesError::IllegalArgument(
        "Template Mode cannot be null",
    ))
}

fn require_non_blank_name(name: Option<&Utf16String>) -> Result<&Utf16String, AttributeNamesError> {
    let name = name.ok_or(AttributeNamesError::IllegalArgument(
        "Name cannot be null or empty",
    ))?;
    if trim_is_empty(name) {
        return Err(AttributeNamesError::IllegalArgument(
            "Name cannot be null or empty",
        ));
    }
    Ok(name)
}

fn checked_buffer(
    buffer: Option<&[u16]>,
    offset: i32,
    length: i32,
) -> Result<&[u16], AttributeNamesError> {
    let buffer = buffer.ok_or(AttributeNamesError::IllegalArgument(
        "Name cannot be null or empty",
    ))?;
    if length == 0 {
        return Err(AttributeNamesError::IllegalArgument(
            "Name cannot be null or empty",
        ));
    }
    if offset < 0 || length < 0 {
        return Err(AttributeNamesError::IllegalArgument(
            "Both name offset and length must be equal to or greater than zero",
        ));
    }
    let start = offset as usize;
    let count = length as usize;
    if start > buffer.len() || count > buffer.len().saturating_sub(start) {
        return Err(AttributeNamesError::StringIndexOutOfBounds {
            offset,
            length,
            buffer_length: buffer.len(),
        });
    }
    Ok(&buffer[start..start + count])
}

fn trim_is_empty(value: &Utf16String) -> bool {
    value.as_utf16().iter().all(|unit| *unit <= 0x20)
}

fn has_non_blank_prefix(prefix: Option<&Utf16String>) -> bool {
    prefix.is_some_and(|value| !trim_is_empty(value))
}

fn namespaced(prefix: &Utf16String, name: &Utf16String) -> Utf16String {
    let mut result = prefix.as_utf16().to_vec();
    result.push(u16::from(b':'));
    result.extend_from_slice(name.as_utf16());
    Utf16String::from_utf16(result)
}

fn starts_ascii_ignore_case(value: &[u16], prefix: &str) -> bool {
    value.len() >= prefix.len() && equals_ascii_ignore_case(&value[..prefix.len()], prefix)
}

fn equals_ascii_ignore_case(value: &[u16], expected: &str) -> bool {
    value.len() == expected.len()
        && value
            .iter()
            .zip(expected.bytes())
            .all(|(actual, expected)| {
                case_fold_unit(*actual) == case_fold_unit(u16::from(expected))
            })
}

fn read_recovering_poison<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_recovering_poison<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
