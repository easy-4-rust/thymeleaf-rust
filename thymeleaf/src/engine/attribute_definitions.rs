use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};

use crate::element::{ElementProcessorSet, IElementProcessor};
use crate::templatemode::TemplateMode;
use crate::util::JavaString;

use super::{
    AttributeDefinition, AttributeDefinitionError, AttributeNameValue, AttributeNames,
    AttributeNamesError, HTMLAttributeDefinition, TextAttributeDefinition, XMLAttributeDefinition,
};

/// 按模板模式组织、且已按优先级排列的元素 Processor。
pub type ElementProcessorsByTemplateMode = HashMap<TemplateMode, Vec<Arc<dyn IElementProcessor>>>;

/// 单个模板模式下的线程安全属性定义仓储。
///
/// 对应 Java: `AttributeDefinitions.AttributeDefinitionRepository`。Java 使用有序数组
/// 加读写锁；Rust 以同样的读写锁边界配合哈希索引，保留并发查找与双重检查写入语义。
type AttributeDefinitionRepository = RwLock<HashMap<JavaString, AttributeDefinitionValue>>;

/// `AttributeDefinitions` 返回的具体属性定义。
#[derive(Clone)]
/// 对应 Java 语义：`AttributeDefinitions` 的 Rust 侧类型 `AttributeDefinitionValue`。
pub enum AttributeDefinitionValue {
    /// HTML 属性定义。
    Html(Arc<HTMLAttributeDefinition>),
    /// XML 属性定义。
    Xml(Arc<XMLAttributeDefinition>),
    /// 文本模式属性定义。
    Text(Arc<TextAttributeDefinition>),
}

impl AttributeDefinitionValue {
    /// 返回公共属性定义视图。
    #[must_use]
    /// 对应 Java 语义：`AttributeDefinitions` 的 `as_attribute_definition` 行为（Rust 侧辅助/私有路径）。
    pub fn as_attribute_definition(&self) -> &AttributeDefinition {
        match self {
            Self::Html(value) => value.as_attribute_definition(),
            Self::Xml(value) => value.as_attribute_definition(),
            Self::Text(value) => value.as_attribute_definition(),
        }
    }
}

/// 属性定义仓储的参数、Processor 配置或名称构造错误。
#[derive(Clone, Debug, Eq, PartialEq)]
/// 对应 Java 语义：`AttributeDefinitions` 的 Rust 侧类型 `AttributeDefinitionsError`。
pub enum AttributeDefinitionsError {
    /// Java 公开入口的参数校验失败。
    IllegalArgument(String),
    /// Processor 的模式与匹配名称模式不一致。
    Configuration(String),
    /// 属性名构造失败。
    AttributeNames(AttributeNamesError),
    /// 属性定义构造失败。
    AttributeDefinition(AttributeDefinitionError),
}

impl AttributeDefinitionsError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub fn java_class_name(&self) -> &str {
        match self {
            Self::IllegalArgument(_) => "java.lang.IllegalArgumentException",
            Self::Configuration(_) => "org.thymeleaf.exceptions.ConfigurationException",
            Self::AttributeNames(error) => error.java_class_name(),
            Self::AttributeDefinition(error) => error.java_class_name(),
        }
    }
}

impl Display for AttributeDefinitionsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalArgument(message) | Self::Configuration(message) => {
                formatter.write_str(message)
            }
            Self::AttributeNames(error) => Display::fmt(error, formatter),
            Self::AttributeDefinition(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for AttributeDefinitionsError {}

impl From<AttributeNamesError> for AttributeDefinitionsError {
    fn from(value: AttributeNamesError) -> Self {
        Self::AttributeNames(value)
    }
}

impl From<AttributeDefinitionError> for AttributeDefinitionsError {
    fn from(value: AttributeDefinitionError) -> Self {
        Self::AttributeDefinition(value)
    }
}

/// 按模板模式缓存属性定义并关联适用 Processor 的线程安全管理器。
///
/// 对应 Java: `org.thymeleaf.engine.AttributeDefinitions`。
pub struct AttributeDefinitions {
    processors: Arc<ElementProcessorsByTemplateMode>,
    html_repository: AttributeDefinitionRepository,
    xml_repository: AttributeDefinitionRepository,
    text_repository: AttributeDefinitionRepository,
    javascript_repository: AttributeDefinitionRepository,
    css_repository: AttributeDefinitionRepository,
}

impl AttributeDefinitions {
    /// 创建管理器并预注册全部标准 HTML 属性。
    ///
    /// 对应 Java: `AttributeDefinitions#AttributeDefinitions(Map)`。
    pub fn new(
        processors: ElementProcessorsByTemplateMode,
    ) -> Result<Self, AttributeDefinitionsError> {
        let manager = Self {
            processors: Arc::new(processors),
            html_repository: RwLock::new(HashMap::new()),
            xml_repository: RwLock::new(HashMap::new()),
            text_repository: RwLock::new(HashMap::new()),
            javascript_repository: RwLock::new(HashMap::new()),
            css_repository: RwLock::new(HashMap::new()),
        };
        for name in STANDARD_HTML_ATTRIBUTE_NAMES {
            manager.for_html_name(Some(&JavaString::from_rust_str(name)))?;
        }
        Ok(manager)
    }

    /// 返回按字典序排列的标准 HTML 属性名。
    #[must_use]
    /// 对应 Java 语义：`AttributeDefinitions` 的 `all_standard_html_attribute_names` 行为（Rust 侧辅助/私有路径）。
    pub fn all_standard_html_attribute_names() -> Vec<&'static str> {
        let mut values = STANDARD_HTML_ATTRIBUTE_NAMES.to_vec();
        values.sort_unstable();
        values
    }

    /// 按模板模式解析完整属性名。
    /// 对应 Java: `AttributeDefinitions#forName()`。
    pub fn for_name(
        &self,
        template_mode: Option<TemplateMode>,
        attribute_name: Option<&JavaString>,
    ) -> Result<AttributeDefinitionValue, AttributeDefinitionsError> {
        let mode = require_mode(template_mode)?;
        match mode {
            TemplateMode::HTML => self
                .for_html_name(attribute_name)
                .map(AttributeDefinitionValue::Html),
            TemplateMode::XML => self
                .for_xml_name(attribute_name)
                .map(AttributeDefinitionValue::Xml),
            TemplateMode::TEXT | TemplateMode::JAVASCRIPT | TemplateMode::CSS => self
                .for_text_mode_name(mode, attribute_name)
                .map(AttributeDefinitionValue::Text),
            TemplateMode::RAW => Err(raw_mode_error(mode)),
        }
    }

    /// 按模板模式解析 prefix 与本地属性名。
    /// 对应 Java 语义：`AttributeDefinitions` 的 `for_name_with_prefix` 行为（Rust 侧辅助/私有路径）。
    pub fn for_name_with_prefix(
        &self,
        template_mode: Option<TemplateMode>,
        prefix: Option<&JavaString>,
        attribute_name: Option<&JavaString>,
    ) -> Result<AttributeDefinitionValue, AttributeDefinitionsError> {
        let mode = require_mode(template_mode)?;
        match mode {
            TemplateMode::HTML => self
                .for_html_name_with_prefix(prefix, attribute_name)
                .map(AttributeDefinitionValue::Html),
            TemplateMode::XML => self
                .for_xml_name_with_prefix(prefix, attribute_name)
                .map(AttributeDefinitionValue::Xml),
            TemplateMode::TEXT | TemplateMode::JAVASCRIPT | TemplateMode::CSS => self
                .for_text_mode_name_with_prefix(mode, prefix, attribute_name)
                .map(AttributeDefinitionValue::Text),
            TemplateMode::RAW => Err(raw_mode_error(mode)),
        }
    }

    /// 按模板模式解析 UTF-16 buffer 子范围。
    /// 对应 Java 语义：`AttributeDefinitions` 的 `for_name_buffer` 行为（Rust 侧辅助/私有路径）。
    pub fn for_name_buffer(
        &self,
        template_mode: Option<TemplateMode>,
        attribute_name: Option<&[u16]>,
        attribute_name_offset: i32,
        attribute_name_len: i32,
    ) -> Result<AttributeDefinitionValue, AttributeDefinitionsError> {
        let mode = require_mode(template_mode)?;
        if mode == TemplateMode::RAW {
            return Err(raw_mode_error(mode));
        }
        let name = AttributeNames::for_name_buffer(
            Some(mode),
            attribute_name,
            attribute_name_offset,
            attribute_name_len,
        )?;
        self.get_or_build(mode, name)
    }

    /// 返回 HTML 属性定义。
    /// 对应 Java 语义：`AttributeDefinitions` 的 `for_html_name` 行为（Rust 侧辅助/私有路径）。
    pub fn for_html_name(
        &self,
        attribute_name: Option<&JavaString>,
    ) -> Result<Arc<HTMLAttributeDefinition>, AttributeDefinitionsError> {
        let name = AttributeNames::for_html_name(attribute_name)?;
        self.get_or_build(TemplateMode::HTML, AttributeNameValue::Html(name))?
            .into_html()
    }

    /// 返回带 prefix 的 HTML 属性定义。
    /// 对应 Java 语义：`AttributeDefinitions` 的 `for_html_name_with_prefix` 行为（Rust 侧辅助/私有路径）。
    pub fn for_html_name_with_prefix(
        &self,
        prefix: Option<&JavaString>,
        attribute_name: Option<&JavaString>,
    ) -> Result<Arc<HTMLAttributeDefinition>, AttributeDefinitionsError> {
        let name = AttributeNames::for_html_name_with_prefix(prefix, attribute_name)?;
        self.get_or_build(TemplateMode::HTML, AttributeNameValue::Html(name))?
            .into_html()
    }

    /// 返回 XML 属性定义。
    /// 对应 Java 语义：`AttributeDefinitions` 的 `for_xml_name` 行为（Rust 侧辅助/私有路径）。
    pub fn for_xml_name(
        &self,
        attribute_name: Option<&JavaString>,
    ) -> Result<Arc<XMLAttributeDefinition>, AttributeDefinitionsError> {
        let name = AttributeNames::for_xml_name(attribute_name)?;
        self.get_or_build(TemplateMode::XML, AttributeNameValue::Xml(name))?
            .into_xml()
    }

    /// 返回带 prefix 的 XML 属性定义。
    /// 对应 Java 语义：`AttributeDefinitions` 的 `for_xml_name_with_prefix` 行为（Rust 侧辅助/私有路径）。
    pub fn for_xml_name_with_prefix(
        &self,
        prefix: Option<&JavaString>,
        attribute_name: Option<&JavaString>,
    ) -> Result<Arc<XMLAttributeDefinition>, AttributeDefinitionsError> {
        let name = AttributeNames::for_xml_name_with_prefix(prefix, attribute_name)?;
        self.get_or_build(TemplateMode::XML, AttributeNameValue::Xml(name))?
            .into_xml()
    }

    /// 返回 TEXT 属性定义。
    /// 对应 Java: `AttributeDefinitions#forTextName()`。
    pub fn for_text_name(
        &self,
        attribute_name: Option<&JavaString>,
    ) -> Result<Arc<TextAttributeDefinition>, AttributeDefinitionsError> {
        self.for_text_mode_name(TemplateMode::TEXT, attribute_name)
    }

    /// 返回 JAVASCRIPT 属性定义。
    /// 对应 Java 语义：`AttributeDefinitions` 的 `for_javascript_name` 行为（Rust 侧辅助/私有路径）。
    pub fn for_javascript_name(
        &self,
        attribute_name: Option<&JavaString>,
    ) -> Result<Arc<TextAttributeDefinition>, AttributeDefinitionsError> {
        self.for_text_mode_name(TemplateMode::JAVASCRIPT, attribute_name)
    }

    /// 返回 CSS 属性定义。
    /// 对应 Java 语义：`AttributeDefinitions` 的 `for_css_name` 行为（Rust 侧辅助/私有路径）。
    pub fn for_css_name(
        &self,
        attribute_name: Option<&JavaString>,
    ) -> Result<Arc<TextAttributeDefinition>, AttributeDefinitionsError> {
        self.for_text_mode_name(TemplateMode::CSS, attribute_name)
    }

    fn for_text_mode_name(
        &self,
        mode: TemplateMode,
        attribute_name: Option<&JavaString>,
    ) -> Result<Arc<TextAttributeDefinition>, AttributeDefinitionsError> {
        let name = AttributeNames::for_text_name(attribute_name)?;
        self.get_or_build(mode, AttributeNameValue::Text(name))?
            .into_text()
    }

    fn for_text_mode_name_with_prefix(
        &self,
        mode: TemplateMode,
        prefix: Option<&JavaString>,
        attribute_name: Option<&JavaString>,
    ) -> Result<Arc<TextAttributeDefinition>, AttributeDefinitionsError> {
        let name = AttributeNames::for_text_name_with_prefix(prefix, attribute_name)?;
        self.get_or_build(mode, AttributeNameValue::Text(name))?
            .into_text()
    }

    fn get_or_build(
        &self,
        mode: TemplateMode,
        name: AttributeNameValue,
    ) -> Result<AttributeDefinitionValue, AttributeDefinitionsError> {
        // 仓储键取第一个完整属性名（与下方别名插入的原始名称一致），
        // 不能用 `{...}` 包裹的 toString 形式，否则查找永远 miss、
        // 每次调用都新建定义，破坏 Java assertSame 的对象身份合同。
        let name_arc = name.as_attribute_name().get_complete_attribute_names();
        let complete_names = read_lock(&name_arc);
        let key = complete_names.first().and_then(Clone::clone).ok_or(
            AttributeDefinitionError::AttributeName(
                super::AttributeNameError::EmptyCompleteAttributeNames,
            ),
        )?;
        let repository = self.repository(mode);
        if let Some(value) = read_lock(repository).get(&key) {
            return Ok(value.clone());
        }
        let mut repository = write_lock(repository);
        if let Some(value) = repository.get(&key) {
            return Ok(value.clone());
        }
        let value = self.build(mode, name)?;
        for alias in complete_attribute_names(value.as_attribute_definition())? {
            repository.insert(alias, value.clone());
        }
        Ok(value)
    }

    fn build(
        &self,
        mode: TemplateMode,
        name: AttributeNameValue,
    ) -> Result<AttributeDefinitionValue, AttributeDefinitionsError> {
        let mut associated = ElementProcessorSet::new();
        for processor in self.processors.get(&mode).into_iter().flatten() {
            if processor.get_template_mode() != Some(mode) {
                continue;
            }
            let element_match = processor.get_matching_element_name();
            let attribute_match = processor.get_matching_attribute_name();
            if element_match.is_some_and(|value| value.get_template_mode() != mode)
                || attribute_match.is_some_and(|value| value.get_template_mode() != mode)
            {
                return Err(AttributeDefinitionsError::Configuration(format!(
                    "{mode} processors must return {mode} element names and {mode} attribute names (processor: {})",
                    processor.java_class_name()
                )));
            }
            let Some(attribute_match) = attribute_match else {
                continue;
            };
            if attribute_match.is_matching_all_attributes() {
                continue;
            }
            if !attribute_match
                .matches(Some(&name))
                .map_err(|error| AttributeDefinitionsError::Configuration(error.to_string()))?
            {
                continue;
            }
            associated.insert(Some(Arc::clone(processor)));
        }
        let associated = Arc::new(RwLock::new(associated));
        match name {
            AttributeNameValue::Html(name) => {
                let boolean_attribute =
                    complete_name_values(name.as_attribute_name())
                        .iter()
                        .any(|value| {
                            BOOLEAN_HTML_ATTRIBUTE_NAMES.contains(&value.to_string_lossy().as_str())
                        });
                Ok(AttributeDefinitionValue::Html(Arc::new(
                    HTMLAttributeDefinition::new(name, boolean_attribute, associated)?,
                )))
            }
            AttributeNameValue::Xml(name) => Ok(AttributeDefinitionValue::Xml(Arc::new(
                XMLAttributeDefinition::new(name, associated)?,
            ))),
            AttributeNameValue::Text(name) => Ok(AttributeDefinitionValue::Text(Arc::new(
                TextAttributeDefinition::new(name, associated)?,
            ))),
        }
    }

    fn repository(
        &self,
        mode: TemplateMode,
    ) -> &RwLock<HashMap<JavaString, AttributeDefinitionValue>> {
        match mode {
            TemplateMode::HTML => &self.html_repository,
            TemplateMode::XML => &self.xml_repository,
            TemplateMode::TEXT => &self.text_repository,
            TemplateMode::JAVASCRIPT => &self.javascript_repository,
            TemplateMode::CSS => &self.css_repository,
            TemplateMode::RAW => unreachable!("RAW has no attribute repository"),
        }
    }
}

impl AttributeDefinitionValue {
    fn into_html(self) -> Result<Arc<HTMLAttributeDefinition>, AttributeDefinitionsError> {
        match self {
            Self::Html(value) => Ok(value),
            _ => Err(AttributeDefinitionsError::Configuration(
                "HTML repository returned a non-HTML definition".into(),
            )),
        }
    }

    fn into_xml(self) -> Result<Arc<XMLAttributeDefinition>, AttributeDefinitionsError> {
        match self {
            Self::Xml(value) => Ok(value),
            _ => Err(AttributeDefinitionsError::Configuration(
                "XML repository returned a non-XML definition".into(),
            )),
        }
    }

    fn into_text(self) -> Result<Arc<TextAttributeDefinition>, AttributeDefinitionsError> {
        match self {
            Self::Text(value) => Ok(value),
            _ => Err(AttributeDefinitionsError::Configuration(
                "text repository returned a non-text definition".into(),
            )),
        }
    }
}

fn complete_name_values(name: &super::AttributeName) -> Vec<JavaString> {
    let values = name.get_complete_attribute_names();
    read_lock(&values).iter().filter_map(Clone::clone).collect()
}

fn complete_attribute_names(
    definition: &AttributeDefinition,
) -> Result<Vec<JavaString>, AttributeDefinitionsError> {
    let values = complete_name_values(definition.get_attribute_name().as_attribute_name());
    if values.is_empty() {
        return Err(AttributeDefinitionError::AttributeName(
            super::AttributeNameError::EmptyCompleteAttributeNames,
        )
        .into());
    }
    Ok(values)
}

fn require_mode(mode: Option<TemplateMode>) -> Result<TemplateMode, AttributeDefinitionsError> {
    mode.ok_or_else(|| {
        AttributeDefinitionsError::IllegalArgument("Template Mode cannot be null".into())
    })
}

fn raw_mode_error(mode: TemplateMode) -> AttributeDefinitionsError {
    AttributeDefinitionsError::IllegalArgument(format!(
        "Attribute Definitions cannot be obtained for {mode} template mode "
    ))
}

fn read_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

const BOOLEAN_HTML_ATTRIBUTE_NAMES: &[&str] = &[
    "async",
    "autofocus",
    "autoplay",
    "checked",
    "controls",
    "declare",
    "default",
    "defer",
    "disabled",
    "formnovalidate",
    "hidden",
    "ismap",
    "loop",
    "multiple",
    "novalidate",
    "nowrap",
    "open",
    "pubdate",
    "readonly",
    "required",
    "reversed",
    "selected",
    "scoped",
    "seamless",
];

const STANDARD_HTML_ATTRIBUTE_NAMES: &[&str] = &[
    "abbr",
    "accept",
    "accept-charset",
    "accesskey",
    "action",
    "align",
    "alt",
    "archive",
    "async",
    "autocomplete",
    "autofocus",
    "autoplay",
    "axis",
    "border",
    "cellpadding",
    "cellspacing",
    "challenge",
    "char",
    "charoff",
    "charset",
    "checked",
    "cite",
    "class",
    "classid",
    "codebase",
    "codetype",
    "cols",
    "colspan",
    "command",
    "content",
    "contenteditable",
    "contextmenu",
    "controls",
    "coords",
    "data",
    "datetime",
    "declare",
    "default",
    "defer",
    "dir",
    "disabled",
    "draggable",
    "dropzone",
    "enctype",
    "for",
    "form",
    "formaction",
    "formenctype",
    "formmethod",
    "formnovalidate",
    "formtarget",
    "frame",
    "headers",
    "height",
    "hidden",
    "high",
    "href",
    "hreflang",
    "http-equiv",
    "icon",
    "id",
    "ismap",
    "keytype",
    "kind",
    "label",
    "lang",
    "list",
    "longdesc",
    "loop",
    "low",
    "max",
    "maxlength",
    "media",
    "method",
    "min",
    "multiple",
    "muted",
    "name",
    "nohref",
    "novalidate",
    "nowrap",
    "onabort",
    "onafterprint",
    "onbeforeprint",
    "onbeforeunload",
    "onblur",
    "oncanplay",
    "oncanplaythrough",
    "onchange",
    "onclick",
    "oncontextmenu",
    "oncuechange",
    "ondblclick",
    "ondrag",
    "ondragend",
    "ondragenter",
    "ondragleave",
    "ondragover",
    "ondragstart",
    "ondrop",
    "ondurationchange",
    "onemptied",
    "onended",
    "onerror",
    "onfocus",
    "onformchange",
    "onforminput",
    "onhaschange",
    "oninput",
    "oninvalid",
    "onkeydown",
    "onkeypress",
    "onkeyup",
    "onload",
    "onloadeddata",
    "onloadedmetadata",
    "onloadstart",
    "onmessage",
    "onmousedown",
    "onmousemove",
    "onmouseout",
    "onmouseover",
    "onmouseup",
    "onmousewheel",
    "onoffline",
    "ononline",
    "onpagehide",
    "onpageshow",
    "onpause",
    "onplay",
    "onplaying",
    "onpopstate",
    "onprogress",
    "onratechange",
    "onredo",
    "onreset",
    "onresize",
    "onscroll",
    "onseeked",
    "onseeking",
    "onselect",
    "onstalled",
    "onstorage",
    "onsubmit",
    "onsuspend",
    "ontimeupdate",
    "onundo",
    "onunload",
    "onvolumechange",
    "onwaiting",
    "open",
    "optimum",
    "pattern",
    "placeholder",
    "poster",
    "preload",
    "profile",
    "pubdate",
    "radiogroup",
    "readonly",
    "rel",
    "required",
    "rev",
    "reversed",
    "rows",
    "rowspan",
    "rules",
    "scheme",
    "scope",
    "scoped",
    "seamless",
    "selected",
    "shape",
    "size",
    "span",
    "spellcheck",
    "src",
    "srclang",
    "standby",
    "style",
    "summary",
    "tabindex",
    "title",
    "translate",
    "type",
    "usemap",
    "valign",
    "value",
    "valuetype",
    "width",
    "xml:lang",
    "xml:space",
    "xmlns",
];
