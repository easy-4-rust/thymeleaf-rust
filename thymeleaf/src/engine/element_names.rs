use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::templatemode::TemplateMode;
use crate::util::{JavaString, java_case_fold_unit};

use super::{ElementName, ElementNameError, HTMLElementName, TextElementName, XMLElementName};

static HTML_REPOSITORY: OnceLock<RwLock<ElementNamesRepository>> = OnceLock::new();
static XML_REPOSITORY: OnceLock<RwLock<ElementNamesRepository>> = OnceLock::new();
static TEXT_REPOSITORY: OnceLock<RwLock<ElementNamesRepository>> = OnceLock::new();

/// `ElementNames` 返回的具体名称子类。
#[derive(Clone)]
pub enum ElementNameValue {
    /// HTML 名称。
    Html(Arc<HTMLElementName>),
    /// XML 名称。
    Xml(Arc<XMLElementName>),
    /// TEXT/JAVASCRIPT/CSS 名称。
    Text(Arc<TextElementName>),
}

impl ElementNameValue {
    /// 返回统一的 `ElementName` 基类视图。
    #[must_use]
    pub fn as_element_name(&self) -> &ElementName {
        match self {
            Self::Html(value) => value.as_element_name(),
            Self::Xml(value) => value.as_element_name(),
            Self::Text(value) => value.as_element_name(),
        }
    }
}

/// 元素名称规范化或 repository 访问错误。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ElementNamesError {
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
    /// RAW 不是结构化元素名称模式。
    UnknownTemplateMode(TemplateMode),
    /// 具体名称对象构造失败。
    ElementName(ElementNameError),
    /// 插入第二个 complete alias 时遇到已有 repository 项。
    RepositoryAliasCollision,
}

impl ElementNamesError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::IllegalArgument(_)
            | Self::UnknownTemplateMode(_)
            | Self::ElementName(ElementNameError::InvalidElementName) => {
                "java.lang.IllegalArgumentException"
            }
            Self::StringIndexOutOfBounds { .. } => "java.lang.StringIndexOutOfBoundsException",
            Self::ElementName(error) => error.java_class_name(),
            Self::RepositoryAliasCollision => "java.lang.IndexOutOfBoundsException",
        }
    }
}

impl Display for ElementNamesError {
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
            Self::ElementName(error) => Display::fmt(error, formatter),
            Self::RepositoryAliasCollision => {
                formatter.write_str("repository alias already exists")
            }
        }
    }
}

impl Error for ElementNamesError {}

impl From<ElementNameError> for ElementNamesError {
    fn from(value: ElementNameError) -> Self {
        Self::ElementName(value)
    }
}

/// 按 TemplateMode 规范化并复用元素名称的线程安全入口。
///
/// 对应 Java: `org.thymeleaf.engine.ElementNames`。
///
/// 三个静态 repository 分别服务 HTML、XML 与所有文本模式，同一 complete name
/// 的重复查询返回同一个 `Arc`，复现 Java 缓存对象身份。
pub struct ElementNames;

impl ElementNames {
    /// 从 UTF-16 buffer 子范围解析任意结构化模板模式的元素名。
    ///
    /// # 错误
    ///
    /// null mode、非法输入范围、RAW 模式或具体名称校验失败时返回对应 Java 错误。
    pub fn for_name_buffer(
        template_mode: Option<TemplateMode>,
        buffer: Option<&[u16]>,
        offset: i32,
        length: i32,
    ) -> Result<ElementNameValue, ElementNamesError> {
        let mode = require_mode(template_mode)?;
        if mode == TemplateMode::RAW {
            return Err(ElementNamesError::UnknownTemplateMode(mode));
        }
        let text = checked_buffer(buffer, offset, length, mode.is_text())?;
        Self::for_name(Some(mode), Some(&JavaString::from_utf16(text.to_vec())))
    }

    /// 从完整 Java String 解析任意结构化模板模式的元素名。
    pub fn for_name(
        template_mode: Option<TemplateMode>,
        element_name: Option<&JavaString>,
    ) -> Result<ElementNameValue, ElementNamesError> {
        let mode = require_mode(template_mode)?;
        match mode {
            TemplateMode::HTML => Self::for_html_name(element_name).map(ElementNameValue::Html),
            TemplateMode::XML => Self::for_xml_name(element_name).map(ElementNameValue::Xml),
            mode if mode.is_text() => Self::for_text_name(element_name).map(ElementNameValue::Text),
            mode => Err(ElementNamesError::UnknownTemplateMode(mode)),
        }
    }

    /// 从显式 prefix 与本地名解析任意结构化模板模式的元素名。
    pub fn for_name_with_prefix(
        template_mode: Option<TemplateMode>,
        prefix: Option<&JavaString>,
        element_name: Option<&JavaString>,
    ) -> Result<ElementNameValue, ElementNamesError> {
        let mode = require_mode(template_mode)?;
        match mode {
            TemplateMode::HTML => {
                Self::for_html_name_with_prefix(prefix, element_name).map(ElementNameValue::Html)
            }
            TemplateMode::XML => {
                Self::for_xml_name_with_prefix(prefix, element_name).map(ElementNameValue::Xml)
            }
            mode if mode.is_text() => {
                Self::for_text_name_with_prefix(prefix, element_name).map(ElementNameValue::Text)
            }
            mode => Err(ElementNamesError::UnknownTemplateMode(mode)),
        }
    }

    /// 解析并缓存文本模式元素名；空字符串合法。
    pub fn for_text_name(
        element_name: Option<&JavaString>,
    ) -> Result<Arc<TextElementName>, ElementNamesError> {
        let element_name = element_name.ok_or(ElementNamesError::IllegalArgument(
            "Name cannot be null or empty",
        ))?;
        match repository_get_or_store(TemplateMode::TEXT, element_name, || {
            build_text(element_name)
        })? {
            ElementNameValue::Text(value) => Ok(value),
            _ => unreachable!("text repository contains only text names"),
        }
    }

    /// 解析并缓存 XML 元素名。
    pub fn for_xml_name(
        element_name: Option<&JavaString>,
    ) -> Result<Arc<XMLElementName>, ElementNamesError> {
        let element_name = require_non_blank_name(element_name)?;
        match repository_get_or_store(TemplateMode::XML, element_name, || build_xml(element_name))?
        {
            ElementNameValue::Xml(value) => Ok(value),
            _ => unreachable!("xml repository contains only xml names"),
        }
    }

    /// 解析并缓存 HTML 元素名。
    pub fn for_html_name(
        element_name: Option<&JavaString>,
    ) -> Result<Arc<HTMLElementName>, ElementNamesError> {
        let element_name = require_non_blank_name(element_name)?;
        match repository_get_or_store(TemplateMode::HTML, element_name, || {
            build_html(element_name)
        })? {
            ElementNameValue::Html(value) => Ok(value),
            _ => unreachable!("html repository contains only html names"),
        }
    }

    /// 使用显式 prefix 解析文本模式元素名。
    pub fn for_text_name_with_prefix(
        prefix: Option<&JavaString>,
        element_name: Option<&JavaString>,
    ) -> Result<Arc<TextElementName>, ElementNamesError> {
        let element_name = element_name.ok_or(ElementNamesError::IllegalArgument(
            "Name cannot be null (nor empty if prefix is not empty)",
        ))?;
        if java_trim_is_empty(element_name) && has_non_blank_prefix(prefix) {
            return Err(ElementNamesError::IllegalArgument(
                "Name cannot be null (nor empty if prefix is not empty)",
            ));
        }
        if !has_non_blank_prefix(prefix) {
            return Self::for_text_name(Some(element_name));
        }
        let lookup = namespaced(prefix.expect("non-blank prefix"), element_name);
        match repository_get_or_store(TemplateMode::TEXT, &lookup, || {
            Ok(ElementNameValue::Text(Arc::new(TextElementName::for_name(
                prefix.cloned(),
                Some(element_name.clone()),
            )?)))
        })? {
            ElementNameValue::Text(value) => Ok(value),
            _ => unreachable!("text repository contains only text names"),
        }
    }

    /// 使用显式 prefix 解析 XML 元素名。
    pub fn for_xml_name_with_prefix(
        prefix: Option<&JavaString>,
        element_name: Option<&JavaString>,
    ) -> Result<Arc<XMLElementName>, ElementNamesError> {
        let element_name = require_non_blank_name(element_name)?;
        if !has_non_blank_prefix(prefix) {
            return Self::for_xml_name(Some(element_name));
        }
        let lookup = namespaced(prefix.expect("non-blank prefix"), element_name);
        match repository_get_or_store(TemplateMode::XML, &lookup, || {
            Ok(ElementNameValue::Xml(Arc::new(XMLElementName::for_name(
                prefix.cloned(),
                Some(element_name.clone()),
            )?)))
        })? {
            ElementNameValue::Xml(value) => Ok(value),
            _ => unreachable!("xml repository contains only xml names"),
        }
    }

    /// 使用显式 prefix 解析 HTML 元素名。
    pub fn for_html_name_with_prefix(
        prefix: Option<&JavaString>,
        element_name: Option<&JavaString>,
    ) -> Result<Arc<HTMLElementName>, ElementNamesError> {
        let element_name = require_non_blank_name(element_name)?;
        if !has_non_blank_prefix(prefix) {
            return Self::for_html_name(Some(element_name));
        }
        let lookup = namespaced(prefix.expect("non-blank prefix"), element_name);
        match repository_get_or_store(TemplateMode::HTML, &lookup, || {
            Ok(ElementNameValue::Html(Arc::new(HTMLElementName::for_name(
                prefix.cloned(),
                Some(element_name.clone()),
            )?)))
        })? {
            ElementNameValue::Html(value) => Ok(value),
            _ => unreachable!("html repository contains only html names"),
        }
    }
}

struct ElementNamesRepository {
    values: HashMap<Vec<u16>, ElementNameValue>,
}

fn repository_get_or_store(
    mode: TemplateMode,
    lookup: &JavaString,
    builder: impl FnOnce() -> Result<ElementNameValue, ElementNamesError>,
) -> Result<ElementNameValue, ElementNamesError> {
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
    let names = value.as_element_name().get_complete_element_names();
    let names = read_recovering_poison(&names).clone();
    let mut keys = Vec::with_capacity(names.len());
    for name in names.into_iter().flatten() {
        let alias = repository_key(mode, &name);
        // 对应 Java `ElementNamesRepository` 的首注册者胜语义：任何 complete name 键
        // 已被不同对象占用时，返回既有绑定（Java 读路径 short-circuit），keep-first。
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

fn repository(mode: TemplateMode) -> &'static RwLock<ElementNamesRepository> {
    let slot = match mode {
        TemplateMode::HTML => &HTML_REPOSITORY,
        TemplateMode::XML => &XML_REPOSITORY,
        _ => &TEXT_REPOSITORY,
    };
    slot.get_or_init(|| {
        RwLock::new(ElementNamesRepository {
            values: HashMap::with_capacity(500),
        })
    })
}

fn repository_key(mode: TemplateMode, value: &JavaString) -> Vec<u16> {
    if mode.is_case_sensitive() {
        value.as_utf16().to_vec()
    } else {
        value
            .as_utf16()
            .iter()
            .map(|unit| java_case_fold_unit(*unit))
            .collect()
    }
}

fn build_text(name: &JavaString) -> Result<ElementNameValue, ElementNamesError> {
    let (prefix, local) = split_first(name, &[u16::from(b':')], false);
    Ok(ElementNameValue::Text(Arc::new(TextElementName::for_name(
        prefix,
        Some(local),
    )?)))
}

fn build_xml(name: &JavaString) -> Result<ElementNameValue, ElementNamesError> {
    let (prefix, local) = split_first(name, &[u16::from(b':')], false);
    Ok(ElementNameValue::Xml(Arc::new(XMLElementName::for_name(
        prefix,
        Some(local),
    )?)))
}

fn build_html(name: &JavaString) -> Result<ElementNameValue, ElementNamesError> {
    let units = name.as_utf16();
    let split = units.iter().position(|unit| matches!(*unit, 0x3a | 0x2d));
    let (prefix, local) = match split {
        Some(0) | None => (None, name.clone()),
        Some(index) if units[index] == u16::from(b':') => {
            let candidate = &units[..=index];
            if equals_ascii_ignore_case(candidate, "xml:")
                || equals_ascii_ignore_case(candidate, "xmlns:")
            {
                (None, name.clone())
            } else {
                (
                    Some(JavaString::from_utf16(units[..index].to_vec())),
                    JavaString::from_utf16(units[index + 1..].to_vec()),
                )
            }
        }
        Some(index) => (
            Some(JavaString::from_utf16(units[..index].to_vec())),
            JavaString::from_utf16(units[index + 1..].to_vec()),
        ),
    };
    Ok(ElementNameValue::Html(Arc::new(HTMLElementName::for_name(
        prefix,
        Some(local),
    )?)))
}

fn split_first(
    name: &JavaString,
    separators: &[u16],
    _unused: bool,
) -> (Option<JavaString>, JavaString) {
    let units = name.as_utf16();
    match units.iter().position(|unit| separators.contains(unit)) {
        Some(0) | None => (None, name.clone()),
        Some(index) => (
            Some(JavaString::from_utf16(units[..index].to_vec())),
            JavaString::from_utf16(units[index + 1..].to_vec()),
        ),
    }
}

fn require_mode(mode: Option<TemplateMode>) -> Result<TemplateMode, ElementNamesError> {
    mode.ok_or(ElementNamesError::IllegalArgument(
        "Template Mode cannot be null",
    ))
}

fn require_non_blank_name(name: Option<&JavaString>) -> Result<&JavaString, ElementNamesError> {
    let name = name.ok_or(ElementNamesError::IllegalArgument(
        "Name cannot be null or empty",
    ))?;
    if java_trim_is_empty(name) {
        return Err(ElementNamesError::IllegalArgument(
            "Name cannot be null or empty",
        ));
    }
    Ok(name)
}

fn checked_buffer(
    buffer: Option<&[u16]>,
    offset: i32,
    length: i32,
    allow_empty: bool,
) -> Result<&[u16], ElementNamesError> {
    let buffer = buffer.ok_or(ElementNamesError::IllegalArgument(
        "Name cannot be null or empty",
    ))?;
    if (!allow_empty && length == 0) || offset < 0 || length < 0 {
        return Err(ElementNamesError::IllegalArgument(if length == 0 {
            "Name cannot be null or empty"
        } else {
            "Both name offset and length must be equal to or greater than zero"
        }));
    }
    let start = usize::try_from(offset).unwrap_or(usize::MAX);
    let count = usize::try_from(length).unwrap_or(usize::MAX);
    if start > buffer.len() || count > buffer.len().saturating_sub(start) {
        return Err(ElementNamesError::StringIndexOutOfBounds {
            offset,
            length,
            buffer_length: buffer.len(),
        });
    }
    Ok(&buffer[start..start + count])
}

fn java_trim_is_empty(value: &JavaString) -> bool {
    value.as_utf16().iter().all(|unit| *unit <= 0x20)
}

fn has_non_blank_prefix(prefix: Option<&JavaString>) -> bool {
    prefix.is_some_and(|value| !java_trim_is_empty(value))
}

fn namespaced(prefix: &JavaString, name: &JavaString) -> JavaString {
    let mut result = prefix.as_utf16().to_vec();
    result.push(u16::from(b':'));
    result.extend_from_slice(name.as_utf16());
    JavaString::from_utf16(result)
}

fn equals_ascii_ignore_case(value: &[u16], expected: &str) -> bool {
    value.len() == expected.len()
        && value
            .iter()
            .zip(expected.bytes())
            .all(|(actual, expected)| {
                java_case_fold_unit(*actual) == java_case_fold_unit(u16::from(expected))
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
