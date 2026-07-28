use std::any::Any;
use std::collections::{BTreeSet, HashMap};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use thiserror::Error;

use crate::TemplateMode;

/// Java `Set<String>` 形式的模板选择器输入。
///
/// 对应 Java: `java.util.Set<String>`，用于
/// `org.thymeleaf.TemplateSpec` 的 `templateSelectors` 参数。
///
/// 外层 `Option` 表示 Java 集合元素可以为 `null`；构造成功后，
/// `TemplateSpec` 会保证所有选择器均非空并按 Java UTF-16 字典序保存。
pub type TemplateSelectorSet = BTreeSet<Option<String>>;

/// Java `Map<String,Object>` 形式的模板解析属性。
///
/// 对应 Java: `org.thymeleaf.TemplateSpec#templateResolutionAttributes`。
/// 键和值均保留 Java `null` 的表达能力。构造 `TemplateSpec` 时会复制整个映射，
/// 后续只能通过共享引用读取，从而对应 Java 的不可修改防御性副本。
pub type TemplateResolutionAttributes = HashMap<Option<String>, TemplateResolutionAttributeValue>;

trait ErasedAttributeValue: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn equals_erased(&self, other: &dyn ErasedAttributeValue) -> bool;
    fn hash_erased(&self) -> u64;
    fn fmt_java(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result;
}

struct TypedAttributeValue<T>(T);

impl<T> ErasedAttributeValue for TypedAttributeValue<T>
where
    T: Any + Display + Eq + Hash + Send + Sync,
{
    fn as_any(&self) -> &dyn Any {
        &self.0
    }

    fn equals_erased(&self, other: &dyn ErasedAttributeValue) -> bool {
        other.as_any().downcast_ref::<T>() == Some(&self.0)
    }

    fn hash_erased(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.0.type_id().hash(&mut hasher);
        self.0.hash(&mut hasher);
        hasher.finish()
    }

    fn fmt_java(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

/// 单个模板解析属性值的线程安全、可克隆类型擦除容器。
///
/// 对应 Java: `java.lang.Object`，用于
/// `org.thymeleaf.TemplateSpec#templateResolutionAttributes` 的值。
///
/// Java 要求这些值具有有效的 `equals`、`hashCode` 和 `toString` 实现；
/// 因而 Rust 值必须实现 `Eq`、`Hash` 和 `Display`。比较时同时检查具体 Rust
/// 类型，使 `i32` 与 `i64` 等不同 Java 包装类型的语义保持区分。
#[derive(Clone)]
pub struct TemplateResolutionAttributeValue {
    inner: Option<Arc<dyn ErasedAttributeValue>>,
}

impl TemplateResolutionAttributeValue {
    /// 包装一个非空解析属性值。
    ///
    /// # 参数
    /// - `value`：对应 Java 映射中的非 `null` `Object` 值。
    ///
    /// # 返回
    /// 可在线程间安全共享、按值比较和哈希的类型擦除属性值。
    #[must_use]
    pub fn new<T>(value: T) -> Self
    where
        T: Any + Display + Eq + Hash + Send + Sync,
    {
        Self {
            inner: Some(Arc::new(TypedAttributeValue(value))),
        }
    }

    /// 创建对应 Java `null` 的解析属性值。
    ///
    /// # 返回
    /// 一个可放入 `TemplateResolutionAttributes` 的空值标记。
    #[must_use]
    pub const fn null() -> Self {
        Self { inner: None }
    }

    /// 判断当前值是否对应 Java `null`。
    ///
    /// # 返回
    /// 值为空时返回 `true`。
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.inner.is_none()
    }
}

impl Debug for TemplateResolutionAttributeValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, formatter)
    }
}

impl Display for TemplateResolutionAttributeValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            Some(inner) => inner.fmt_java(formatter),
            None => formatter.write_str("null"),
        }
    }
}

impl PartialEq for TemplateResolutionAttributeValue {
    fn eq(&self, other: &Self) -> bool {
        match (&self.inner, &other.inner) {
            (Some(left), Some(right)) => left.equals_erased(right.as_ref()),
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        }
    }
}

impl Eq for TemplateResolutionAttributeValue {}

impl Hash for TemplateResolutionAttributeValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.inner {
            Some(inner) => {
                1_u8.hash(state);
                inner.hash_erased().hash(state);
            }
            None => 0_u8.hash(state),
        }
    }
}

/// 模板处理所需的模板标识、选择器、模式、解析属性及输出类型。
///
/// 对应 Java: `org.thymeleaf.TemplateSpec`。
///
/// `template` 是唯一必填字段，它既可以是模板名称，也可以是由字符串模板解析器
/// 直接处理的完整模板内容。模板规格描述模板本身，不包含上下文变量或区域设置。
/// 构造时会复制并冻结选择器和解析属性，因此对象可安全地在线程之间共享。
#[derive(Clone, Debug)]
pub struct TemplateSpec {
    template: String,
    template_selectors: Option<Vec<String>>,
    template_mode: Option<TemplateMode>,
    template_resolution_attributes: Option<TemplateResolutionAttributes>,
    output_content_type: Option<String>,
    output_sse: bool,
}

impl TemplateSpec {
    /// 使用模板和可选强制模板模式构造规格。
    ///
    /// 对应 Java: `TemplateSpec(String, TemplateMode)`。
    ///
    /// # 参数
    /// - `template`：模板名称或完整模板内容；`None` 对应 Java `null`。
    /// - `template_mode`：需要强制使用的模板模式，可以为空。
    ///
    /// # 错误
    /// `template` 为 `None` 时返回 `TemplateSpecError::TemplateCannotBeNull`。
    pub fn with_template_mode(
        template: Option<&str>,
        template_mode: Option<TemplateMode>,
    ) -> Result<Self, TemplateSpecError> {
        Self::try_new(template, None, template_mode, None, None)
    }

    /// 使用模板和可选输出 MIME 类型构造规格。
    ///
    /// 对应 Java: `TemplateSpec(String, String)`。
    ///
    /// # 参数
    /// - `template`：模板名称或完整模板内容；`None` 对应 Java `null`。
    /// - `output_content_type`：期望的输出内容类型，可以为空；参数部分会被忽略。
    ///
    /// # 错误
    /// 模板为空或 MIME 文本仅含分隔符时返回相应 `TemplateSpecError`。
    pub fn with_output_content_type(
        template: Option<&str>,
        output_content_type: Option<&str>,
    ) -> Result<Self, TemplateSpecError> {
        Self::try_new(template, None, None, output_content_type, None)
    }

    /// 使用模板和模板解析属性构造规格。
    ///
    /// 对应 Java: `TemplateSpec(String, Map<String,Object>)`。
    ///
    /// # 参数
    /// - `template`：模板名称或完整模板内容；`None` 对应 Java `null`。
    /// - `template_resolution_attributes`：传递给模板解析器并参与缓存键计算的属性。
    ///
    /// # 错误
    /// `template` 为 `None` 时返回 `TemplateSpecError::TemplateCannotBeNull`。
    pub fn with_resolution_attributes(
        template: Option<&str>,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
    ) -> Result<Self, TemplateSpecError> {
        Self::try_new(template, None, None, None, template_resolution_attributes)
    }

    /// 使用选择器、强制模板模式和解析属性构造规格。
    ///
    /// 对应 Java:
    /// `TemplateSpec(String, Set<String>, TemplateMode, Map<String,Object>)`。
    ///
    /// # 参数
    /// - `template`：模板名称或完整模板内容。
    /// - `template_selectors`：只处理模板指定片段的选择器集合。
    /// - `template_mode`：需要强制使用的模板模式，可以为空。
    /// - `template_resolution_attributes`：传递给解析器并参与缓存键的属性。
    ///
    /// # 错误
    /// 模板为空，或选择器包含 `null`、空字符串、纯空白时返回错误。
    pub fn with_selectors_and_template_mode(
        template: Option<&str>,
        template_selectors: Option<&TemplateSelectorSet>,
        template_mode: Option<TemplateMode>,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
    ) -> Result<Self, TemplateSpecError> {
        Self::try_new(
            template,
            template_selectors,
            template_mode,
            None,
            template_resolution_attributes,
        )
    }

    /// 使用选择器、输出 MIME 类型和解析属性构造规格。
    ///
    /// 对应 Java:
    /// `TemplateSpec(String, Set<String>, String, Map<String,Object>)`。
    ///
    /// # 参数
    /// - `template`：模板名称或完整模板内容。
    /// - `template_selectors`：只处理模板指定片段的选择器集合。
    /// - `output_content_type`：输出 MIME 类型；可据此强制模板模式或启用 SSE。
    /// - `template_resolution_attributes`：传递给解析器并参与缓存键的属性。
    ///
    /// # 错误
    /// 模板、选择器或 MIME 输入不符合 Java 构造器约束时返回错误。
    pub fn with_selectors_and_output_content_type(
        template: Option<&str>,
        template_selectors: Option<&TemplateSelectorSet>,
        output_content_type: Option<&str>,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
    ) -> Result<Self, TemplateSpecError> {
        Self::try_new(
            template,
            template_selectors,
            None,
            output_content_type,
            template_resolution_attributes,
        )
    }

    pub(crate) fn try_new(
        template: Option<&str>,
        template_selectors: Option<&TemplateSelectorSet>,
        template_mode: Option<TemplateMode>,
        output_content_type: Option<&str>,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
    ) -> Result<Self, TemplateSpecError> {
        let template = template.ok_or(TemplateSpecError::TemplateCannotBeNull)?;
        if template_mode.is_some() && output_content_type.is_some() {
            return Err(TemplateSpecError::ModeAndContentTypeConflict);
        }

        let template_selectors = normalize_selectors(template_selectors)?;
        let template_resolution_attributes =
            template_resolution_attributes.filter(|attributes| !attributes.is_empty());
        let output_content_type = output_content_type.map(str::to_owned);
        let normalized_mime_type = parse_mime_type(output_content_type.as_deref())?;
        let computed_template_mode =
            compute_template_mode_for_mime_type(normalized_mime_type.as_deref());
        let output_sse = normalized_mime_type.as_deref() == Some("text/event-stream");

        Ok(Self {
            template: template.to_owned(),
            template_selectors,
            template_mode: computed_template_mode.or(template_mode),
            template_resolution_attributes: template_resolution_attributes.cloned(),
            output_content_type,
            output_sse,
        })
    }

    /// 返回模板名称或完整模板内容。
    ///
    /// 对应 Java: `TemplateSpec#getTemplate()`。
    ///
    /// # 返回
    /// 构造时传入的非空模板字符串。
    #[must_use]
    pub fn get_template(&self) -> &str {
        &self.template
    }

    /// 判断是否指定了模板选择器。
    ///
    /// 对应 Java: `TemplateSpec#hasTemplateSelectors()`。
    ///
    /// # 返回
    /// 规范包含至少一个选择器时返回 `true`。
    #[must_use]
    pub const fn has_template_selectors(&self) -> bool {
        self.template_selectors.is_some()
    }

    /// 返回已按 Java UTF-16 字典序冻结的模板选择器。
    ///
    /// 对应 Java: `TemplateSpec#getTemplateSelectors()`。
    ///
    /// # 返回
    /// 未指定选择器时返回 `None`，否则返回不可变集合共享引用。
    #[must_use]
    pub fn get_template_selectors(&self) -> Option<&[String]> {
        self.template_selectors.as_deref()
    }

    /// 判断是否指定或由 MIME 类型推导出了模板模式。
    ///
    /// 对应 Java: `TemplateSpec#hasTemplateMode()`。
    ///
    /// # 返回
    /// 存在模板模式时返回 `true`。
    #[must_use]
    pub const fn has_template_mode(&self) -> bool {
        self.template_mode.is_some()
    }

    /// 返回显式指定或由输出 MIME 类型推导出的模板模式。
    ///
    /// 对应 Java: `TemplateSpec#getTemplateMode()`。
    ///
    /// # 返回
    /// 未指定且无法推导时返回 `None`。
    #[must_use]
    pub const fn get_template_mode(&self) -> Option<TemplateMode> {
        self.template_mode
    }

    /// 判断是否存在模板解析属性。
    ///
    /// 对应 Java: `TemplateSpec#hasTemplateResolutionAttributes()`。
    ///
    /// # 返回
    /// 构造时传入非空属性映射时返回 `true`。
    #[must_use]
    pub const fn has_template_resolution_attributes(&self) -> bool {
        self.template_resolution_attributes.is_some()
    }

    /// 返回模板解析属性的不可变防御性副本。
    ///
    /// 对应 Java: `TemplateSpec#getTemplateResolutionAttributes()`。
    ///
    /// 属性会传递给模板解析器，并作为模板标识和缓存键的一部分；值必须提供稳定的
    /// 相等与哈希语义。
    ///
    /// # 返回
    /// 未指定属性时返回 `None`。
    #[must_use]
    pub const fn get_template_resolution_attributes(
        &self,
    ) -> Option<&TemplateResolutionAttributes> {
        self.template_resolution_attributes.as_ref()
    }

    /// 返回构造时指定的原始输出内容类型。
    ///
    /// 对应 Java: `TemplateSpec#getOutputContentType()`。
    ///
    /// # 返回
    /// 未指定时返回 `None`；否则保留大小写、空白和 MIME 参数的原始文本。
    #[must_use]
    pub fn get_output_content_type(&self) -> Option<&str> {
        self.output_content_type.as_deref()
    }

    /// 判断输出是否使用 Server-Sent Events 模式。
    ///
    /// 对应 Java: `TemplateSpec#isOutputSSE()`。
    ///
    /// # 返回
    /// 输出 MIME 类型归一化为 `text/event-stream` 时返回 `true`。
    #[must_use]
    pub const fn is_output_sse(&self) -> bool {
        self.output_sse
    }

    /// 按 Java `equals(Object)` 的可观察顺序比较两个规格。
    ///
    /// 对应 Java: `TemplateSpec#equals(Object)`。
    ///
    /// 上游 3.1.5 在比较到 `outputContentType` 时直接调用实例方法；当接收者该字段
    /// 为 `null` 且不是同一对象时会抛出 `NullPointerException`。本方法用类型化错误
    /// 保留该行为；Rust 自身的 `PartialEq` 则提供满足集合契约的安全比较。
    ///
    /// # 参数
    /// - `other`：Java `Object` 参数；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// 同一对象或所有 Java 字段相等时返回 `Ok(true)`；类型不匹配或字段不同时返回
    /// `Ok(false)`。
    ///
    /// # 错误
    /// 比较执行到空 `outputContentType` 时返回
    /// `TemplateSpecError::JavaEqualsNullOutputContentType`。
    pub fn equals_java(&self, other: Option<&dyn Any>) -> Result<bool, TemplateSpecError> {
        let Some(other) = other else {
            return Ok(false);
        };
        let Some(that) = other.downcast_ref::<Self>() else {
            return Ok(false);
        };
        if std::ptr::eq(self, that) {
            return Ok(true);
        }
        if self.template != that.template
            || self.template_selectors != that.template_selectors
            || self.template_mode != that.template_mode
        {
            return Ok(false);
        }
        let Some(output_content_type) = &self.output_content_type else {
            return Err(TemplateSpecError::JavaEqualsNullOutputContentType);
        };
        if Some(output_content_type) != that.output_content_type.as_ref() {
            return Ok(false);
        }
        Ok(self.template_resolution_attributes == that.template_resolution_attributes)
    }
}

impl PartialEq for TemplateSpec {
    fn eq(&self, other: &Self) -> bool {
        self.template == other.template
            && self.template_selectors == other.template_selectors
            && self.template_mode == other.template_mode
            && self.output_content_type == other.output_content_type
            && self.template_resolution_attributes == other.template_resolution_attributes
    }
}

impl Eq for TemplateSpec {}

impl Hash for TemplateSpec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.template.hash(state);
        self.template_selectors.hash(state);
        self.template_mode.hash(state);
        self.output_content_type.hash(state);
        hash_attributes(&self.template_resolution_attributes, state);
    }
}

impl Display for TemplateSpec {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let mut rendered = loggify_template_name(&self.template);
        if let Some(selectors) = &self.template_selectors {
            rendered.push_str("::");
            rendered.push_str(&format_selectors(selectors));
        }
        if let Some(template_mode) = self.template_mode {
            rendered.push_str(" @");
            rendered.push_str(&template_mode.to_string());
        }
        if let Some(attributes) = &self.template_resolution_attributes {
            rendered.push_str(" (");
            rendered.push_str(&format_attributes(attributes));
            rendered.push(')');
        }
        if let Some(output_content_type) = &self.output_content_type {
            rendered.push_str(" [");
            rendered.push_str(output_content_type);
            rendered.push(']');
        }
        formatter.write_str(&rendered)
    }
}

/// `TemplateSpec` 构造或 Java 兼容比较期间的输入与语义错误。
///
/// 对应 Java: `org.thymeleaf.TemplateSpec` 抛出的
/// `IllegalArgumentException`、`ArrayIndexOutOfBoundsException` 与
/// `NullPointerException`。
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TemplateSpecError {
    /// Java 构造器收到 `null` 模板。
    #[error("Template cannot be null")]
    TemplateCannotBeNull,
    /// 同时指定模板模式与输出内容类型。
    #[error("If template mode or output content type are specified, the other one cannot")]
    ModeAndContentTypeConflict,
    /// 选择器集合包含 `null`、空字符串或纯空白。
    #[error("If specified, the Template Selector set cannot contain any nulls or empties")]
    NullOrEmptyTemplateSelector,
    /// MIME 字符串非空但经 Java `StringTokenizer` 拆分后没有任何令牌。
    #[error("Index 0 out of bounds for length 0")]
    MalformedOutputContentType,
    /// Java `equals` 在空输出内容类型上调用实例方法。
    #[error("Cannot invoke \"String.equals(Object)\" because \"this.outputContentType\" is null")]
    JavaEqualsNullOutputContentType,
}

fn normalize_selectors(
    template_selectors: Option<&TemplateSelectorSet>,
) -> Result<Option<Vec<String>>, TemplateSpecError> {
    let Some(template_selectors) = template_selectors.filter(|selectors| !selectors.is_empty())
    else {
        return Ok(None);
    };
    let mut normalized = Vec::with_capacity(template_selectors.len());
    for selector in template_selectors {
        let Some(selector) = selector else {
            return Err(TemplateSpecError::NullOrEmptyTemplateSelector);
        };
        if is_java_empty_or_whitespace(selector) {
            return Err(TemplateSpecError::NullOrEmptyTemplateSelector);
        }
        normalized.push(selector.clone());
    }
    normalized.sort_by(|left, right| left.encode_utf16().cmp(right.encode_utf16()));
    Ok(Some(normalized))
}

fn compute_template_mode_for_mime_type(mime_type: Option<&str>) -> Option<TemplateMode> {
    let mime_type = mime_type?;
    match mime_type {
        "text/html" | "application/xhtml+xml" => Some(TemplateMode::HTML),
        "application/xml" | "text/xml" | "application/rss+xml" | "application/atom+xml" => {
            Some(TemplateMode::XML)
        }
        "application/javascript"
        | "application/x-javascript"
        | "application/ecmascript"
        | "text/javascript"
        | "text/ecmascript"
        | "application/json" => Some(TemplateMode::JAVASCRIPT),
        "text/css" => Some(TemplateMode::CSS),
        "text/plain" => Some(TemplateMode::TEXT),
        _ => None,
    }
}

fn parse_mime_type(output_content_type: Option<&str>) -> Result<Option<String>, TemplateSpecError> {
    let Some(output_content_type) = output_content_type else {
        return Ok(None);
    };
    if java_trim(output_content_type).is_empty() {
        return Ok(None);
    }

    // Java StringTokenizer 会忽略连续、前导及尾部分隔符。
    let Some(mime_type) = output_content_type
        .split(';')
        .find(|token| !token.is_empty())
    else {
        return Err(TemplateSpecError::MalformedOutputContentType);
    };
    let lowercase_mime_type = mime_type.to_lowercase();
    Ok(Some(java_trim(&lowercase_mime_type).to_owned()))
}

fn java_trim(value: &str) -> &str {
    value.trim_matches(|character| character <= '\u{0020}')
}

fn is_java_empty_or_whitespace(value: &str) -> bool {
    value.is_empty() || value.chars().all(is_java_whitespace)
}

fn is_java_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000D}'
            | '\u{001C}'..='\u{0020}'
            | '\u{1680}'
            | '\u{2000}'..='\u{2006}'
            | '\u{2008}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

pub(crate) fn loggify_template_name(template: &str) -> String {
    let utf16 = template.encode_utf16().collect::<Vec<_>>();
    if utf16.len() <= 120 {
        return template.replace('\n', " ");
    }

    // Java String.length/substring 以 UTF-16 代码单元计数；有效 Unicode 边界处可精确对应。
    let prefix = String::from_utf16_lossy(&utf16[..35]).replace('\n', " ");
    let suffix = String::from_utf16_lossy(&utf16[utf16.len() - 80..]).replace('\n', " ");
    format!("{prefix}[...]{suffix}")
}

fn format_selectors(selectors: &[String]) -> String {
    format!("[{}]", selectors.join(", "))
}

pub(crate) fn format_attributes(attributes: &TemplateResolutionAttributes) -> String {
    let mut entries = attributes
        .iter()
        .map(|(key, value)| format!("{}={value}", key.as_deref().unwrap_or("null")))
        .collect::<Vec<_>>();
    entries.sort();
    format!("{{{}}}", entries.join(", "))
}

fn hash_attributes<H: Hasher>(attributes: &Option<TemplateResolutionAttributes>, state: &mut H) {
    let Some(attributes) = attributes else {
        0_u8.hash(state);
        return;
    };
    1_u8.hash(state);
    let mut entry_hashes = attributes
        .iter()
        .map(|entry| {
            let mut hasher = DefaultHasher::new();
            entry.hash(&mut hasher);
            hasher.finish()
        })
        .collect::<Vec<_>>();
    entry_hashes.sort_unstable();
    entry_hashes.hash(state);
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::collections::{HashMap, hash_map::DefaultHasher};
    use std::hash::{Hash, Hasher};

    use super::{
        TemplateResolutionAttributeValue, TemplateResolutionAttributes, TemplateSelectorSet,
        TemplateSpec, TemplateSpecError, loggify_template_name,
    };
    use crate::TemplateMode;

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    fn selectors(values: &[Option<&str>]) -> TemplateSelectorSet {
        values
            .iter()
            .map(|value| value.map(str::to_owned))
            .collect()
    }

    fn attributes() -> TemplateResolutionAttributes {
        HashMap::from([
            (
                Some("tenant".to_owned()),
                TemplateResolutionAttributeValue::new("acme".to_owned()),
            ),
            (
                Some("attempt".to_owned()),
                TemplateResolutionAttributeValue::new(3_i32),
            ),
            (None, TemplateResolutionAttributeValue::null()),
        ])
    }

    #[test]
    fn constructors_validate_template_and_conflicting_fields() {
        assert_eq!(
            TemplateSpec::with_template_mode(None, None),
            Err(TemplateSpecError::TemplateCannotBeNull)
        );
        assert_eq!(
            TemplateSpec::try_new(
                Some("index"),
                None,
                Some(TemplateMode::HTML),
                Some("text/html"),
                None
            ),
            Err(TemplateSpecError::ModeAndContentTypeConflict)
        );
        assert_eq!(
            TemplateSpec::with_output_content_type(Some("index"), Some(";;;")),
            Err(TemplateSpecError::MalformedOutputContentType)
        );
        assert_eq!(
            TemplateSpecError::TemplateCannotBeNull.to_string(),
            "Template cannot be null"
        );
        assert_eq!(
            TemplateSpecError::ModeAndContentTypeConflict.to_string(),
            "If template mode or output content type are specified, the other one cannot"
        );
        assert_eq!(
            TemplateSpecError::MalformedOutputContentType.to_string(),
            "Index 0 out of bounds for length 0"
        );
    }

    #[test]
    fn constructors_normalize_selectors_and_copy_attributes() {
        let empty_selectors = TemplateSelectorSet::new();
        let empty_attributes = TemplateResolutionAttributes::new();
        let plain = TemplateSpec::with_selectors_and_template_mode(
            Some("index"),
            Some(&empty_selectors),
            None,
            Some(&empty_attributes),
        )
        .unwrap();
        assert!(!plain.has_template_selectors());
        assert_eq!(plain.get_template_selectors(), None);
        assert!(!plain.has_template_resolution_attributes());
        assert_eq!(plain.get_template_resolution_attributes(), None);

        for invalid in [
            selectors(&[None]),
            selectors(&[Some("")]),
            selectors(&[Some(" \n")]),
            selectors(&[Some("\u{2003}")]),
            selectors(&[Some("\u{2009}")]),
        ] {
            assert_eq!(
                TemplateSpec::with_selectors_and_template_mode(
                    Some("index"),
                    Some(&invalid),
                    None,
                    None
                ),
                Err(TemplateSpecError::NullOrEmptyTemplateSelector)
            );
        }
        assert_eq!(
            TemplateSpecError::NullOrEmptyTemplateSelector.to_string(),
            "If specified, the Template Selector set cannot contain any nulls or empties"
        );

        let selected = selectors(&[
            Some("footer"),
            Some("article"),
            Some("article"),
            Some("\u{00A0}"),
        ]);
        let mut source_attributes = attributes();
        let spec = TemplateSpec::with_selectors_and_template_mode(
            Some("index"),
            Some(&selected),
            Some(TemplateMode::XML),
            Some(&source_attributes),
        )
        .unwrap();
        source_attributes.clear();

        assert_eq!(spec.get_template(), "index");
        assert!(spec.has_template_selectors());
        assert_eq!(
            spec.get_template_selectors(),
            Some(
                [
                    "article".to_owned(),
                    "footer".to_owned(),
                    "\u{00A0}".to_owned()
                ]
                .as_slice()
            )
        );
        assert!(spec.has_template_mode());
        assert_eq!(spec.get_template_mode(), Some(TemplateMode::XML));
        assert!(spec.has_template_resolution_attributes());
        assert_eq!(spec.get_template_resolution_attributes().unwrap().len(), 3);
        assert_eq!(spec.get_output_content_type(), None);
        assert!(!spec.is_output_sse());
    }

    #[test]
    fn content_types_force_exact_modes_and_preserve_original_text() {
        let cases = [
            ("text/html", Some(TemplateMode::HTML), false),
            ("application/xhtml+xml", Some(TemplateMode::HTML), false),
            ("application/xml", Some(TemplateMode::XML), false),
            ("text/xml", Some(TemplateMode::XML), false),
            ("application/rss+xml", Some(TemplateMode::XML), false),
            ("application/atom+xml", Some(TemplateMode::XML), false),
            (
                "application/javascript",
                Some(TemplateMode::JAVASCRIPT),
                false,
            ),
            (
                "application/x-javascript",
                Some(TemplateMode::JAVASCRIPT),
                false,
            ),
            (
                "application/ecmascript",
                Some(TemplateMode::JAVASCRIPT),
                false,
            ),
            ("text/javascript", Some(TemplateMode::JAVASCRIPT), false),
            ("text/ecmascript", Some(TemplateMode::JAVASCRIPT), false),
            ("application/json", Some(TemplateMode::JAVASCRIPT), false),
            ("text/css", Some(TemplateMode::CSS), false),
            ("text/plain", Some(TemplateMode::TEXT), false),
            ("text/event-stream", None, true),
            ("application/octet-stream", None, false),
            ("", None, false),
            (" \t", None, false),
        ];

        for (content_type, expected_mode, expected_sse) in cases {
            let spec =
                TemplateSpec::with_output_content_type(Some("index"), Some(content_type)).unwrap();
            assert_eq!(spec.get_template_mode(), expected_mode, "{content_type}");
            assert_eq!(spec.is_output_sse(), expected_sse, "{content_type}");
            assert_eq!(spec.get_output_content_type(), Some(content_type));
        }

        let normalized = TemplateSpec::with_output_content_type(
            Some("index"),
            Some("; TEXT/HTML ;; Charset=UTF-8"),
        )
        .unwrap();
        assert_eq!(normalized.get_template_mode(), Some(TemplateMode::HTML));
        assert_eq!(
            normalized.get_output_content_type(),
            Some("; TEXT/HTML ;; Charset=UTF-8")
        );
    }

    #[test]
    fn all_public_constructor_shapes_preserve_fields() {
        let attrs = attributes();
        let by_mode =
            TemplateSpec::with_template_mode(Some("index"), Some(TemplateMode::RAW)).unwrap();
        assert_eq!(by_mode.get_template_mode(), Some(TemplateMode::RAW));

        let by_attributes =
            TemplateSpec::with_resolution_attributes(Some("index"), Some(&attrs)).unwrap();
        assert!(by_attributes.has_template_resolution_attributes());

        let selector_set = selectors(&[Some("main")]);
        let by_content_type = TemplateSpec::with_selectors_and_output_content_type(
            Some("index"),
            Some(&selector_set),
            Some("text/css"),
            Some(&attrs),
        )
        .unwrap();
        assert_eq!(by_content_type.get_template_mode(), Some(TemplateMode::CSS));
        assert_eq!(by_content_type.get_template_selectors().unwrap().len(), 1);
    }

    #[test]
    fn attribute_values_preserve_null_type_equality_hash_and_display() {
        let null = TemplateResolutionAttributeValue::null();
        let another_null = TemplateResolutionAttributeValue::null();
        let i32_value = TemplateResolutionAttributeValue::new(7_i32);
        let same_i32 = TemplateResolutionAttributeValue::new(7_i32);
        let other_i32 = TemplateResolutionAttributeValue::new(8_i32);
        let i64_value = TemplateResolutionAttributeValue::new(7_i64);

        assert!(null.is_null());
        assert!(!i32_value.is_null());
        assert_eq!(null, another_null);
        assert_ne!(null, i32_value);
        assert_eq!(i32_value, same_i32);
        assert_ne!(i32_value, other_i32);
        assert_ne!(i32_value, i64_value);
        assert_eq!(hash_of(&i32_value), hash_of(&same_i32));
        assert_eq!(null.to_string(), "null");
        assert_eq!(format!("{null:?}"), "null");
        assert_eq!(i32_value.to_string(), "7");
    }

    #[test]
    fn rust_equality_and_hash_are_safe_and_order_independent() {
        let first_attributes = attributes();
        let second_attributes = HashMap::from_iter(first_attributes.clone());
        let first =
            TemplateSpec::with_resolution_attributes(Some("index"), Some(&first_attributes))
                .unwrap();
        let second =
            TemplateSpec::with_resolution_attributes(Some("index"), Some(&second_attributes))
                .unwrap();
        let different = TemplateSpec::with_template_mode(Some("other"), None).unwrap();

        assert_eq!(first, second);
        assert_eq!(hash_of(&first), hash_of(&second));
        assert_ne!(first, different);
        assert_ne!(hash_of(&first), hash_of(&different));
    }

    #[test]
    fn java_equals_preserves_identity_order_and_null_output_bug() {
        let without_content_type = TemplateSpec::with_template_mode(Some("index"), None).unwrap();
        let same_fields = TemplateSpec::with_template_mode(Some("index"), None).unwrap();
        assert_eq!(
            without_content_type.equals_java(Some(&without_content_type)),
            Ok(true)
        );
        assert_eq!(without_content_type.equals_java(None), Ok(false));
        assert_eq!(
            without_content_type.equals_java(Some(&"not a spec" as &dyn Any)),
            Ok(false)
        );
        assert_eq!(
            without_content_type.equals_java(Some(&same_fields)),
            Err(TemplateSpecError::JavaEqualsNullOutputContentType)
        );
        assert_eq!(
            TemplateSpecError::JavaEqualsNullOutputContentType.to_string(),
            "Cannot invoke \"String.equals(Object)\" because \"this.outputContentType\" is null"
        );

        let different_template =
            TemplateSpec::with_output_content_type(Some("other"), Some("text/html")).unwrap();
        let left =
            TemplateSpec::with_output_content_type(Some("index"), Some("text/html")).unwrap();
        assert_eq!(left.equals_java(Some(&different_template)), Ok(false));

        let selectors = selectors(&[Some("main")]);
        let different_selectors = TemplateSpec::with_selectors_and_output_content_type(
            Some("index"),
            Some(&selectors),
            Some("text/html"),
            None,
        )
        .unwrap();
        assert_eq!(left.equals_java(Some(&different_selectors)), Ok(false));

        let different_mode =
            TemplateSpec::with_output_content_type(Some("index"), Some("text/plain")).unwrap();
        assert_eq!(left.equals_java(Some(&different_mode)), Ok(false));

        let missing_content = TemplateSpec::with_template_mode(Some("index"), None).unwrap();
        assert_eq!(left.equals_java(Some(&missing_content)), Ok(false));

        let same =
            TemplateSpec::with_output_content_type(Some("index"), Some("text/html")).unwrap();
        assert_eq!(left.equals_java(Some(&same)), Ok(true));

        let attrs = attributes();
        let with_attrs = TemplateSpec::with_selectors_and_output_content_type(
            Some("index"),
            None,
            Some("text/html"),
            Some(&attrs),
        )
        .unwrap();
        assert_eq!(left.equals_java(Some(&with_attrs)), Ok(false));
    }

    #[test]
    fn display_matches_java_shape_and_loggifies_long_names() {
        let selectors = selectors(&[Some("footer"), Some("article")]);
        let attrs = HashMap::from([
            (
                Some("tenant".to_owned()),
                TemplateResolutionAttributeValue::new("acme".to_owned()),
            ),
            (None, TemplateResolutionAttributeValue::null()),
        ]);
        let spec = TemplateSpec::with_selectors_and_output_content_type(
            Some("home\npage"),
            Some(&selectors),
            Some("text/html;charset=UTF-8"),
            Some(&attrs),
        )
        .unwrap();
        assert_eq!(
            spec.to_string(),
            "home page::[article, footer] @HTML ({null=null, tenant=acme}) [text/html;charset=UTF-8]"
        );
        assert!(format!("{spec:?}").contains("template: \"home\\npage\""));

        let short = "x".repeat(120);
        assert_eq!(loggify_template_name(&short), short);

        let long = format!("{}\n{}尾", "a".repeat(34), "b".repeat(90));
        let rendered = loggify_template_name(&long);
        assert!(rendered.starts_with(&format!("{} ", "a".repeat(34))));
        assert!(rendered.contains("[...]"));
        assert!(rendered.ends_with('尾'));
    }

    #[test]
    fn selectors_use_java_utf16_lexicographic_order() {
        let selector_set = selectors(&[Some("\u{E000}"), Some("\u{10000}")]);
        let spec = TemplateSpec::with_selectors_and_template_mode(
            Some("index"),
            Some(&selector_set),
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            spec.get_template_selectors(),
            Some(["\u{10000}".to_owned(), "\u{E000}".to_owned()].as_slice())
        );
    }
}
