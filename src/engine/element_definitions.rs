use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};

use crate::element::ElementProcessorSet;
use crate::templatemode::TemplateMode;
use crate::util::JavaString;

use super::{
    ElementDefinition, ElementDefinitionError, ElementNameValue, ElementNames, ElementNamesError,
    ElementProcessorsByTemplateMode, HTMLElementDefinition, HTMLElementType, TextElementDefinition,
    XMLElementDefinition,
};

/// 单个模板模式下的线程安全元素定义仓储。
///
/// 对应 Java: `ElementDefinitions.ElementDefinitionRepository`。Rust 使用读写锁保护
/// 哈希索引，等价保留 Java 的并发读取、缺失后加写锁创建与别名共同缓存语义。
type ElementDefinitionRepository = RwLock<HashMap<JavaString, ElementDefinitionValue>>;

/// 标准 HTML 元素名称及类别的初始化规格。
///
/// 对应 Java: `ElementDefinitions.HTMLElementDefinitionSpec`。
struct HTMLElementDefinitionSpec {
    name: &'static str,
    element_type: HTMLElementType,
}

impl HTMLElementDefinitionSpec {
    const fn new(name: &'static str, element_type: HTMLElementType) -> Self {
        Self { name, element_type }
    }
}

/// `ElementDefinitions` 返回的具体元素定义。
#[derive(Clone)]
pub enum ElementDefinitionValue {
    /// HTML 元素定义。
    Html(Arc<HTMLElementDefinition>),
    /// XML 元素定义。
    Xml(Arc<XMLElementDefinition>),
    /// 文本模式元素定义。
    Text(Arc<TextElementDefinition>),
}

impl ElementDefinitionValue {
    /// 返回公共元素定义视图。
    #[must_use]
    pub fn as_element_definition(&self) -> &ElementDefinition {
        match self {
            Self::Html(value) => value.as_element_definition(),
            Self::Xml(value) => value.as_element_definition(),
            Self::Text(value) => value.as_element_definition(),
        }
    }
}

/// 元素定义仓储的参数、Processor 配置或名称构造错误。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ElementDefinitionsError {
    /// Java 公开入口参数校验失败。
    IllegalArgument(String),
    /// Processor 的模式与匹配名称模式不一致。
    Configuration(String),
    /// 元素名构造失败。
    ElementNames(ElementNamesError),
    /// 元素定义构造失败。
    ElementDefinition(ElementDefinitionError),
}

impl ElementDefinitionsError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub fn java_class_name(&self) -> &str {
        match self {
            Self::IllegalArgument(_) => "java.lang.IllegalArgumentException",
            Self::Configuration(_) => "org.thymeleaf.exceptions.ConfigurationException",
            Self::ElementNames(error) => error.java_class_name(),
            Self::ElementDefinition(error) => error.java_class_name(),
        }
    }
}

impl Display for ElementDefinitionsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalArgument(message) | Self::Configuration(message) => {
                formatter.write_str(message)
            }
            Self::ElementNames(error) => Display::fmt(error, formatter),
            Self::ElementDefinition(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for ElementDefinitionsError {}

impl From<ElementNamesError> for ElementDefinitionsError {
    fn from(value: ElementNamesError) -> Self {
        Self::ElementNames(value)
    }
}

impl From<ElementDefinitionError> for ElementDefinitionsError {
    fn from(value: ElementDefinitionError) -> Self {
        Self::ElementDefinition(value)
    }
}

/// 按模板模式缓存元素定义并关联适用 Processor 的线程安全管理器。
///
/// 对应 Java: `org.thymeleaf.engine.ElementDefinitions`。
pub struct ElementDefinitions {
    processors: Arc<ElementProcessorsByTemplateMode>,
    html_repository: ElementDefinitionRepository,
    xml_repository: ElementDefinitionRepository,
    text_repository: ElementDefinitionRepository,
    javascript_repository: ElementDefinitionRepository,
    css_repository: ElementDefinitionRepository,
}

impl ElementDefinitions {
    /// 创建管理器并按上游类型表预注册全部标准 HTML 元素。
    ///
    /// 对应 Java: `ElementDefinitions#ElementDefinitions(Map)`。
    pub fn new(
        processors: ElementProcessorsByTemplateMode,
    ) -> Result<Self, ElementDefinitionsError> {
        let manager = Self {
            processors: Arc::new(processors),
            html_repository: RwLock::new(HashMap::new()),
            xml_repository: RwLock::new(HashMap::new()),
            text_repository: RwLock::new(HashMap::new()),
            javascript_repository: RwLock::new(HashMap::new()),
            css_repository: RwLock::new(HashMap::new()),
        };
        for (name, element_type) in STANDARD_HTML_ELEMENT_SPECS {
            let spec = HTMLElementDefinitionSpec::new(name, *element_type);
            let name = ElementNames::for_html_name(Some(&JavaString::from_rust_str(spec.name)))?;
            manager.get_or_build(
                TemplateMode::HTML,
                ElementNameValue::Html(name),
                spec.element_type,
            )?;
        }
        Ok(manager)
    }

    /// 返回按字典序排列的标准 HTML 元素名。
    #[must_use]
    pub fn all_standard_html_element_names() -> Vec<&'static str> {
        let mut values = STANDARD_HTML_ELEMENT_SPECS
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>();
        values.sort_unstable();
        values
    }

    /// 按模板模式解析完整元素名。
    pub fn for_name(
        &self,
        template_mode: Option<TemplateMode>,
        element_name: Option<&JavaString>,
    ) -> Result<ElementDefinitionValue, ElementDefinitionsError> {
        let mode = require_mode(template_mode)?;
        match mode {
            TemplateMode::HTML => self
                .for_html_name(element_name)
                .map(ElementDefinitionValue::Html),
            TemplateMode::XML => self
                .for_xml_name(element_name)
                .map(ElementDefinitionValue::Xml),
            TemplateMode::TEXT | TemplateMode::JAVASCRIPT | TemplateMode::CSS => self
                .for_text_mode_name(mode, element_name)
                .map(ElementDefinitionValue::Text),
            TemplateMode::RAW => Err(raw_mode_error(mode)),
        }
    }

    /// 按模板模式解析 prefix 与本地元素名。
    pub fn for_name_with_prefix(
        &self,
        template_mode: Option<TemplateMode>,
        prefix: Option<&JavaString>,
        element_name: Option<&JavaString>,
    ) -> Result<ElementDefinitionValue, ElementDefinitionsError> {
        let mode = require_mode(template_mode)?;
        let name = ElementNames::for_name_with_prefix(Some(mode), prefix, element_name)?;
        self.get_or_build(mode, name, HTMLElementType::NORMAL)
    }

    /// 按模板模式解析 UTF-16 buffer 子范围。
    pub fn for_name_buffer(
        &self,
        template_mode: Option<TemplateMode>,
        element_name: Option<&[u16]>,
        element_name_offset: i32,
        element_name_len: i32,
    ) -> Result<ElementDefinitionValue, ElementDefinitionsError> {
        let mode = require_mode(template_mode)?;
        if mode == TemplateMode::RAW {
            return Err(raw_mode_error(mode));
        }
        let name = ElementNames::for_name_buffer(
            Some(mode),
            element_name,
            element_name_offset,
            element_name_len,
        )?;
        self.get_or_build(mode, name, HTMLElementType::NORMAL)
    }

    /// 返回 HTML 元素定义；非标准名称使用 `NORMAL` 类型。
    pub fn for_html_name(
        &self,
        element_name: Option<&JavaString>,
    ) -> Result<Arc<HTMLElementDefinition>, ElementDefinitionsError> {
        let name = ElementNames::for_html_name(element_name)?;
        let element_type = html_element_type(name.as_element_name());
        self.get_or_build(
            TemplateMode::HTML,
            ElementNameValue::Html(name),
            element_type,
        )?
        .into_html()
    }

    /// 返回带 prefix 的 HTML 元素定义。
    pub fn for_html_name_with_prefix(
        &self,
        prefix: Option<&JavaString>,
        element_name: Option<&JavaString>,
    ) -> Result<Arc<HTMLElementDefinition>, ElementDefinitionsError> {
        let name = ElementNames::for_html_name_with_prefix(prefix, element_name)?;
        let element_type = html_element_type(name.as_element_name());
        self.get_or_build(
            TemplateMode::HTML,
            ElementNameValue::Html(name),
            element_type,
        )?
        .into_html()
    }

    /// 返回 XML 元素定义。
    pub fn for_xml_name(
        &self,
        element_name: Option<&JavaString>,
    ) -> Result<Arc<XMLElementDefinition>, ElementDefinitionsError> {
        let name = ElementNames::for_xml_name(element_name)?;
        self.get_or_build(
            TemplateMode::XML,
            ElementNameValue::Xml(name),
            HTMLElementType::NORMAL,
        )?
        .into_xml()
    }

    /// 返回带 prefix 的 XML 元素定义。
    pub fn for_xml_name_with_prefix(
        &self,
        prefix: Option<&JavaString>,
        element_name: Option<&JavaString>,
    ) -> Result<Arc<XMLElementDefinition>, ElementDefinitionsError> {
        let name = ElementNames::for_xml_name_with_prefix(prefix, element_name)?;
        self.get_or_build(
            TemplateMode::XML,
            ElementNameValue::Xml(name),
            HTMLElementType::NORMAL,
        )?
        .into_xml()
    }

    /// 返回 TEXT 元素定义；空名称合法。
    pub fn for_text_name(
        &self,
        element_name: Option<&JavaString>,
    ) -> Result<Arc<TextElementDefinition>, ElementDefinitionsError> {
        self.for_text_mode_name(TemplateMode::TEXT, element_name)
    }

    /// 返回 JAVASCRIPT 元素定义；空名称合法。
    pub fn for_javascript_name(
        &self,
        element_name: Option<&JavaString>,
    ) -> Result<Arc<TextElementDefinition>, ElementDefinitionsError> {
        self.for_text_mode_name(TemplateMode::JAVASCRIPT, element_name)
    }

    /// 返回 CSS 元素定义；空名称合法。
    pub fn for_css_name(
        &self,
        element_name: Option<&JavaString>,
    ) -> Result<Arc<TextElementDefinition>, ElementDefinitionsError> {
        self.for_text_mode_name(TemplateMode::CSS, element_name)
    }

    fn for_text_mode_name(
        &self,
        mode: TemplateMode,
        element_name: Option<&JavaString>,
    ) -> Result<Arc<TextElementDefinition>, ElementDefinitionsError> {
        let name = ElementNames::for_text_name(element_name)?;
        self.get_or_build(mode, ElementNameValue::Text(name), HTMLElementType::NORMAL)?
            .into_text()
    }

    fn get_or_build(
        &self,
        mode: TemplateMode,
        name: ElementNameValue,
        element_type: HTMLElementType,
    ) -> Result<ElementDefinitionValue, ElementDefinitionsError> {
        let key = name
            .as_element_name()
            .to_java_string()
            .map_err(ElementDefinitionError::ElementName)?;
        let repository = self.repository(mode);
        if let Some(value) = read_lock(repository).get(&key) {
            return Ok(value.clone());
        }
        let mut repository = write_lock(repository);
        if let Some(value) = repository.get(&key) {
            return Ok(value.clone());
        }
        let value = self.build(mode, name, element_type)?;
        for alias in complete_element_names(value.as_element_definition())? {
            repository.insert(alias, value.clone());
        }
        Ok(value)
    }

    fn build(
        &self,
        mode: TemplateMode,
        name: ElementNameValue,
        element_type: HTMLElementType,
    ) -> Result<ElementDefinitionValue, ElementDefinitionsError> {
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
                return Err(ElementDefinitionsError::Configuration(format!(
                    "{mode} processors must return {mode} element names and {mode} attribute names (processor: {})",
                    processor.java_class_name()
                )));
            }
            if attribute_match.is_some_and(|value| !value.is_matching_all_attributes()) {
                continue;
            }
            if let Some(element_match) = element_match
                && !element_match
                    .matches(Some(&name))
                    .map_err(|error| ElementDefinitionsError::Configuration(error.to_string()))?
            {
                continue;
            }
            associated.insert(Some(Arc::clone(processor)));
        }
        let associated = Arc::new(RwLock::new(associated));
        match name {
            ElementNameValue::Html(name) => Ok(ElementDefinitionValue::Html(Arc::new(
                HTMLElementDefinition::new(name, element_type, associated)?,
            ))),
            ElementNameValue::Xml(name) => Ok(ElementDefinitionValue::Xml(Arc::new(
                XMLElementDefinition::new(name, associated)?,
            ))),
            ElementNameValue::Text(name) => Ok(ElementDefinitionValue::Text(Arc::new(
                TextElementDefinition::new(name, associated)?,
            ))),
        }
    }

    fn repository(
        &self,
        mode: TemplateMode,
    ) -> &RwLock<HashMap<JavaString, ElementDefinitionValue>> {
        match mode {
            TemplateMode::HTML => &self.html_repository,
            TemplateMode::XML => &self.xml_repository,
            TemplateMode::TEXT => &self.text_repository,
            TemplateMode::JAVASCRIPT => &self.javascript_repository,
            TemplateMode::CSS => &self.css_repository,
            TemplateMode::RAW => unreachable!("RAW has no element repository"),
        }
    }
}

impl ElementDefinitionValue {
    fn into_html(self) -> Result<Arc<HTMLElementDefinition>, ElementDefinitionsError> {
        match self {
            Self::Html(value) => Ok(value),
            _ => Err(ElementDefinitionsError::Configuration(
                "HTML repository returned a non-HTML definition".into(),
            )),
        }
    }

    fn into_xml(self) -> Result<Arc<XMLElementDefinition>, ElementDefinitionsError> {
        match self {
            Self::Xml(value) => Ok(value),
            _ => Err(ElementDefinitionsError::Configuration(
                "XML repository returned a non-XML definition".into(),
            )),
        }
    }

    fn into_text(self) -> Result<Arc<TextElementDefinition>, ElementDefinitionsError> {
        match self {
            Self::Text(value) => Ok(value),
            _ => Err(ElementDefinitionsError::Configuration(
                "text repository returned a non-text definition".into(),
            )),
        }
    }
}

fn html_element_type(name: &super::ElementName) -> HTMLElementType {
    let value = name
        .to_java_string()
        .map_or_else(|_| String::new(), |value| value.to_string_lossy());
    STANDARD_HTML_ELEMENT_SPECS
        .iter()
        .find_map(|(candidate, element_type)| (*candidate == value).then_some(*element_type))
        .unwrap_or(HTMLElementType::NORMAL)
}

fn complete_element_names(
    definition: &ElementDefinition,
) -> Result<Vec<JavaString>, ElementDefinitionsError> {
    let values = definition
        .get_element_name()
        .as_element_name()
        .get_complete_element_names();
    let values = read_lock(&values)
        .iter()
        .filter_map(Clone::clone)
        .collect::<Vec<_>>();
    if values.is_empty() {
        return Err(ElementDefinitionError::ElementName(
            super::ElementNameError::EmptyCompleteElementNames,
        )
        .into());
    }
    Ok(values)
}

fn require_mode(mode: Option<TemplateMode>) -> Result<TemplateMode, ElementDefinitionsError> {
    mode.ok_or_else(|| {
        ElementDefinitionsError::IllegalArgument("Template Mode cannot be null".into())
    })
}

fn raw_mode_error(mode: TemplateMode) -> ElementDefinitionsError {
    ElementDefinitionsError::IllegalArgument(format!(
        "Element Definitions cannot be obtained for {mode} template mode "
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

const STANDARD_HTML_ELEMENT_SPECS: &[(&str, HTMLElementType)] = &[
    ("html", HTMLElementType::NORMAL),
    ("head", HTMLElementType::NORMAL),
    ("title", HTMLElementType::ESCAPABLE_RAW_TEXT),
    ("base", HTMLElementType::VOID),
    ("link", HTMLElementType::VOID),
    ("meta", HTMLElementType::VOID),
    ("style", HTMLElementType::RAW_TEXT),
    ("script", HTMLElementType::RAW_TEXT),
    ("noscript", HTMLElementType::NORMAL),
    ("body", HTMLElementType::NORMAL),
    ("article", HTMLElementType::NORMAL),
    ("section", HTMLElementType::NORMAL),
    ("nav", HTMLElementType::NORMAL),
    ("aside", HTMLElementType::NORMAL),
    ("h1", HTMLElementType::NORMAL),
    ("h2", HTMLElementType::NORMAL),
    ("h3", HTMLElementType::NORMAL),
    ("h4", HTMLElementType::NORMAL),
    ("h5", HTMLElementType::NORMAL),
    ("h6", HTMLElementType::NORMAL),
    ("hgroup", HTMLElementType::NORMAL),
    ("header", HTMLElementType::NORMAL),
    ("footer", HTMLElementType::NORMAL),
    ("address", HTMLElementType::NORMAL),
    ("main", HTMLElementType::NORMAL),
    ("p", HTMLElementType::NORMAL),
    ("hr", HTMLElementType::VOID),
    ("pre", HTMLElementType::NORMAL),
    ("blockquote", HTMLElementType::NORMAL),
    ("ol", HTMLElementType::NORMAL),
    ("ul", HTMLElementType::NORMAL),
    ("li", HTMLElementType::NORMAL),
    ("dl", HTMLElementType::NORMAL),
    ("dt", HTMLElementType::NORMAL),
    ("dd", HTMLElementType::NORMAL),
    ("figure", HTMLElementType::NORMAL),
    ("figcaption", HTMLElementType::NORMAL),
    ("div", HTMLElementType::NORMAL),
    ("a", HTMLElementType::NORMAL),
    ("em", HTMLElementType::NORMAL),
    ("strong", HTMLElementType::NORMAL),
    ("small", HTMLElementType::NORMAL),
    ("s", HTMLElementType::NORMAL),
    ("cite", HTMLElementType::NORMAL),
    ("g", HTMLElementType::NORMAL),
    ("dfn", HTMLElementType::NORMAL),
    ("abbr", HTMLElementType::NORMAL),
    ("time", HTMLElementType::NORMAL),
    ("code", HTMLElementType::NORMAL),
    ("var", HTMLElementType::NORMAL),
    ("samp", HTMLElementType::NORMAL),
    ("kbd", HTMLElementType::NORMAL),
    ("sub", HTMLElementType::NORMAL),
    ("sup", HTMLElementType::NORMAL),
    ("i", HTMLElementType::NORMAL),
    ("b", HTMLElementType::NORMAL),
    ("u", HTMLElementType::NORMAL),
    ("mark", HTMLElementType::NORMAL),
    ("ruby", HTMLElementType::NORMAL),
    ("rb", HTMLElementType::NORMAL),
    ("rt", HTMLElementType::NORMAL),
    ("rtc", HTMLElementType::NORMAL),
    ("rp", HTMLElementType::NORMAL),
    ("bdi", HTMLElementType::NORMAL),
    ("bdo", HTMLElementType::NORMAL),
    ("span", HTMLElementType::NORMAL),
    ("br", HTMLElementType::VOID),
    ("wbr", HTMLElementType::VOID),
    ("ins", HTMLElementType::NORMAL),
    ("del", HTMLElementType::NORMAL),
    ("img", HTMLElementType::VOID),
    ("iframe", HTMLElementType::NORMAL),
    ("embed", HTMLElementType::VOID),
    ("object", HTMLElementType::NORMAL),
    ("param", HTMLElementType::VOID),
    ("video", HTMLElementType::NORMAL),
    ("audio", HTMLElementType::NORMAL),
    ("source", HTMLElementType::VOID),
    ("track", HTMLElementType::VOID),
    ("canvas", HTMLElementType::NORMAL),
    ("map", HTMLElementType::NORMAL),
    ("area", HTMLElementType::VOID),
    ("table", HTMLElementType::NORMAL),
    ("caption", HTMLElementType::NORMAL),
    ("colgroup", HTMLElementType::NORMAL),
    ("col", HTMLElementType::VOID),
    ("tbody", HTMLElementType::NORMAL),
    ("thead", HTMLElementType::NORMAL),
    ("tfoot", HTMLElementType::NORMAL),
    ("tr", HTMLElementType::NORMAL),
    ("td", HTMLElementType::NORMAL),
    ("th", HTMLElementType::NORMAL),
    ("form", HTMLElementType::NORMAL),
    ("fieldset", HTMLElementType::NORMAL),
    ("legend", HTMLElementType::NORMAL),
    ("label", HTMLElementType::NORMAL),
    ("input", HTMLElementType::VOID),
    ("button", HTMLElementType::NORMAL),
    ("select", HTMLElementType::NORMAL),
    ("datalist", HTMLElementType::NORMAL),
    ("optgroup", HTMLElementType::NORMAL),
    ("option", HTMLElementType::NORMAL),
    ("textarea", HTMLElementType::ESCAPABLE_RAW_TEXT),
    ("keygen", HTMLElementType::VOID),
    ("output", HTMLElementType::NORMAL),
    ("progress", HTMLElementType::NORMAL),
    ("meter", HTMLElementType::NORMAL),
    ("details", HTMLElementType::NORMAL),
    ("summary", HTMLElementType::NORMAL),
    ("command", HTMLElementType::NORMAL),
    ("menu", HTMLElementType::NORMAL),
    ("menuitem", HTMLElementType::VOID),
    ("dialog", HTMLElementType::NORMAL),
    ("template", HTMLElementType::RAW_TEXT),
    ("element", HTMLElementType::NORMAL),
    ("decorator", HTMLElementType::NORMAL),
    ("content", HTMLElementType::NORMAL),
    ("shadow", HTMLElementType::NORMAL),
];
