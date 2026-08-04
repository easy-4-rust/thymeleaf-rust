use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::model::{
    AttributeValueQuotes, IAttribute, IElementTag, IModelVisitor, IProcessableElementTag,
    IStandaloneElementTag, ITemplateEvent,
};
use crate::templatemode::TemplateMode;
use crate::util::{FastStringWriter, TemplateWriter, Utf16String};

use super::{
    AbstractProcessableElementTag, AttributeDefinitionValue, AttributeDefinitions, AttributeName,
    Attributes, AttributesError, ElementDefinition, ElementDefinitionValue, IEngineTemplateEvent,
    ITemplateHandler,
};

/// 非最小化 standalone 标签用于非 HTML 模式时的校验错误。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 对应 Java 语义：`StandaloneElementTag` 的 Rust 侧类型 `StandaloneElementTagError`。
pub struct StandaloneElementTagError {
    template_mode: TemplateMode,
}

impl StandaloneElementTagError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn class_name(&self) -> &'static str {
        "java.lang.IllegalArgumentException"
    }

    /// 返回触发校验的模板模式。
    #[must_use]
    pub const fn template_mode(&self) -> TemplateMode {
        self.template_mode
    }
}

impl Display for StandaloneElementTagError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Not-minimized standalone elements are only allowed in HTML template mode (is {})",
            self.template_mode
        )
    }
}

impl Error for StandaloneElementTagError {}

/// 引擎内部的不可变 standalone 元素标签。
///
/// 对应 Java: `org.thymeleaf.engine.StandaloneElementTag`。
pub struct StandaloneElementTag {
    processable_tag: AbstractProcessableElementTag,
    minimized: bool,
}

impl StandaloneElementTag {
    /// 创建没有原模板位置的 standalone 标签。
    ///
    /// 对应 Java:
    /// `StandaloneElementTag#StandaloneElementTag(TemplateMode,ElementDefinition,String,Attributes,boolean,boolean)`。
    pub fn new(
        template_mode: TemplateMode,
        element_definition: ElementDefinitionValue,
        element_complete_name: Utf16String,
        attributes: Option<Arc<Attributes>>,
        synthetic: bool,
        minimized: bool,
    ) -> Result<Self, StandaloneElementTagError> {
        validate_minimized(template_mode, minimized)?;
        Ok(Self {
            processable_tag: AbstractProcessableElementTag::new(
                template_mode,
                element_definition,
                element_complete_name,
                attributes,
                synthetic,
            ),
            minimized,
        })
    }

    /// 创建携带模板名称、行和列的 standalone 标签。
    ///
    /// 对应 Java:
    /// `StandaloneElementTag#StandaloneElementTag(TemplateMode,ElementDefinition,String,Attributes,boolean,boolean,String,int,int)`。
    #[allow(clippy::too_many_arguments)]
    pub fn with_location(
        template_mode: TemplateMode,
        element_definition: ElementDefinitionValue,
        element_complete_name: Utf16String,
        attributes: Option<Arc<Attributes>>,
        synthetic: bool,
        minimized: bool,
        template_name: Option<Utf16String>,
        line: i32,
        col: i32,
    ) -> Result<Self, StandaloneElementTagError> {
        validate_minimized(template_mode, minimized)?;
        Ok(Self {
            processable_tag: AbstractProcessableElementTag::with_location(
                template_mode,
                element_definition,
                element_complete_name,
                attributes,
                synthetic,
                template_name,
                line,
                col,
            ),
            minimized,
        })
    }

    /// 设置或新增属性，返回新的不可变标签。
    ///
    /// 对应 Java: `StandaloneElementTag#setAttribute(...)`。
    pub fn set_attribute(
        self: &Arc<Self>,
        attribute_definitions: &AttributeDefinitions,
        attribute_definition: Option<&AttributeDefinitionValue>,
        complete_name: Utf16String,
        value: Option<Utf16String>,
        value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<Self>, AttributesError> {
        let old_attributes = self
            .processable_tag
            .attributes()
            .cloned()
            .unwrap_or_else(Attributes::empty);
        let new_attributes = old_attributes.set_attribute(
            attribute_definitions,
            self.get_template_mode(),
            attribute_definition,
            complete_name,
            value,
            value_quotes,
        )?;
        Ok(Arc::new(self.derive(Some(new_attributes))))
    }

    /// 将旧名称属性替换为新名称和值。
    ///
    /// 对应 Java: `StandaloneElementTag#replaceAttribute(...)`。
    #[allow(clippy::too_many_arguments)]
    pub fn replace_attribute(
        self: &Arc<Self>,
        attribute_definitions: &AttributeDefinitions,
        old_name: &AttributeName,
        new_attribute_definition: Option<&AttributeDefinitionValue>,
        complete_new_name: Utf16String,
        value: Option<Utf16String>,
        value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<Self>, AttributesError> {
        let old_attributes = self
            .processable_tag
            .attributes()
            .cloned()
            .unwrap_or_else(Attributes::empty);
        let new_attributes = old_attributes.replace_attribute(
            attribute_definitions,
            self.get_template_mode(),
            old_name,
            new_attribute_definition,
            complete_new_name,
            value,
            value_quotes,
        )?;
        Ok(Arc::new(self.derive(Some(new_attributes))))
    }

    /// 按完整名称删除属性；不存在时返回当前同一 `Arc`。
    ///
    /// 对应 Java: `StandaloneElementTag#removeAttribute(String)`。
    pub fn remove_attribute(
        self: &Arc<Self>,
        complete_name: &Utf16String,
    ) -> Result<Arc<Self>, AttributesError> {
        let old_attributes = self
            .processable_tag
            .attributes()
            .cloned()
            .unwrap_or_else(Attributes::empty);
        let new_attributes =
            old_attributes.remove_attribute(self.get_template_mode(), complete_name)?;
        if Arc::ptr_eq(&old_attributes, &new_attributes) {
            return Ok(self.clone());
        }
        Ok(Arc::new(self.derive(Some(new_attributes))))
    }

    /// 按 prefix 与本地名称删除属性；不存在时返回当前同一 `Arc`。
    ///
    /// 对应 Java: `StandaloneElementTag#removeAttribute(String,String)`。
    pub fn remove_attribute_with_prefix(
        self: &Arc<Self>,
        prefix: Option<&Utf16String>,
        name: &Utf16String,
    ) -> Result<Arc<Self>, AttributesError> {
        let old_attributes = self
            .processable_tag
            .attributes()
            .cloned()
            .unwrap_or_else(Attributes::empty);
        let new_attributes =
            old_attributes.remove_attribute_with_prefix(self.get_template_mode(), prefix, name)?;
        if Arc::ptr_eq(&old_attributes, &new_attributes) {
            return Ok(self.clone());
        }
        Ok(Arc::new(self.derive(Some(new_attributes))))
    }

    /// 按规范化名称对象删除属性；不存在时返回当前同一 `Arc`。
    ///
    /// 对应 Java: `StandaloneElementTag#removeAttribute(AttributeName)`。
    #[must_use]
    pub fn remove_attribute_name(self: &Arc<Self>, attribute_name: &AttributeName) -> Arc<Self> {
        let old_attributes = self
            .processable_tag
            .attributes()
            .cloned()
            .unwrap_or_else(Attributes::empty);
        let new_attributes = old_attributes.remove_attribute_base_name(attribute_name);
        if Arc::ptr_eq(&old_attributes, &new_attributes) {
            return self.clone();
        }
        Arc::new(self.derive(Some(new_attributes)))
    }

    /// 返回完整 UTF-16 标签表示。
    #[must_use]
    /// 对应 Java 语义：`StandaloneElementTag` 的 `to_utf16_string` 行为（Rust 侧辅助/私有路径）。
    pub fn to_utf16_string(&self) -> Utf16String {
        let mut writer = FastStringWriter::new();
        self.write(&mut writer)
            .expect("FastStringWriter must accept complete UTF-16 slices");
        writer.to_string()
    }

    fn derive(&self, attributes: Option<Arc<Attributes>>) -> Self {
        let element_tag = self.processable_tag.as_element_tag();
        // 现有对象已经通过 minimized/mode 校验，派生时组合保持不变量。
        Self::with_location(
            element_tag.get_template_mode(),
            element_tag.element_definition_value().clone(),
            element_tag.get_element_complete_name().clone(),
            attributes,
            element_tag.is_synthetic(),
            self.minimized,
            element_tag.as_template_event().get_template_name().cloned(),
            element_tag.as_template_event().get_line(),
            element_tag.as_template_event().get_col(),
        )
        .expect("derived standalone tag preserves validated mode")
    }

    /// 创建与当前 standalone 标签字段完全一致的合成开放标签。
    ///
    /// 对应 Java:
    /// `TemplateModelController#createStandaloneEquivalentModel` 中的
    /// `OpenElementTag` 构造。
    pub(crate) fn as_synthetic_open_equivalent(&self) -> super::OpenElementTag {
        let element_tag = self.processable_tag.as_element_tag();
        super::OpenElementTag::with_location(
            element_tag.get_template_mode(),
            element_tag.element_definition_value().clone(),
            element_tag.get_element_complete_name().clone(),
            self.processable_tag.attributes().cloned(),
            element_tag.is_synthetic(),
            element_tag.as_template_event().get_template_name().cloned(),
            element_tag.as_template_event().get_line(),
            element_tag.as_template_event().get_col(),
        )
    }

    /// 创建与当前 standalone 标签字段完全一致的合成关闭标签。
    ///
    /// 对应 Java:
    /// `TemplateModelController#createStandaloneEquivalentModel` 中的
    /// `CloseElementTag` 构造。
    pub(crate) fn as_synthetic_close_equivalent(&self) -> super::CloseElementTag {
        let element_tag = self.processable_tag.as_element_tag();
        super::CloseElementTag::with_location(
            element_tag.get_template_mode(),
            element_tag.element_definition_value().clone(),
            element_tag.get_element_complete_name().clone(),
            None,
            element_tag.is_synthetic(),
            false,
            element_tag.as_template_event().get_template_name().cloned(),
            element_tag.as_template_event().get_line(),
            element_tag.as_template_event().get_col(),
        )
    }
}

impl IStandaloneElementTag for StandaloneElementTag {
    fn into_engine_standalone_element_tag(self: Arc<Self>) -> Option<Arc<Self>> {
        Some(self)
    }

    fn as_engine_standalone_element_tag(&self) -> Option<&Self> {
        Some(self)
    }

    fn is_minimized(&self) -> bool {
        self.minimized
    }
}

impl IProcessableElementTag for StandaloneElementTag {
    fn as_engine_processable_element_tag(&self) -> Option<&AbstractProcessableElementTag> {
        Some(&self.processable_tag)
    }

    fn into_standalone_element_tag(self: Arc<Self>) -> Option<Arc<dyn IStandaloneElementTag>> {
        Some(self)
    }

    fn get_all_attributes(&self) -> Vec<&dyn IAttribute> {
        self.processable_tag
            .attributes()
            .and_then(|attributes| attributes.as_attribute_slice())
            .map_or_else(Vec::new, |attributes| {
                attributes
                    .iter()
                    .map(|attribute| attribute.as_ref() as &dyn IAttribute)
                    .collect()
            })
    }

    fn get_attribute_map(&self) -> IndexMap<Utf16String, Option<Utf16String>> {
        self.processable_tag.get_attribute_map()
    }

    fn has_attribute(&self, complete_name: &Utf16String) -> Result<bool, AttributesError> {
        self.processable_tag.has_attribute(complete_name)
    }

    fn has_attribute_with_prefix(
        &self,
        prefix: Option<&Utf16String>,
        name: &Utf16String,
    ) -> Result<bool, AttributesError> {
        self.processable_tag.has_attribute_with_prefix(prefix, name)
    }

    fn has_attribute_name(&self, attribute_name: &AttributeName) -> bool {
        self.processable_tag.has_attribute_name(attribute_name)
    }

    fn get_attribute(
        &self,
        complete_name: &Utf16String,
    ) -> Result<Option<&dyn IAttribute>, AttributesError> {
        Ok(self
            .processable_tag
            .get_attribute(complete_name)?
            .map(|attribute| attribute as &dyn IAttribute))
    }

    fn get_attribute_with_prefix(
        &self,
        prefix: Option<&Utf16String>,
        name: &Utf16String,
    ) -> Result<Option<&dyn IAttribute>, AttributesError> {
        Ok(self
            .processable_tag
            .get_attribute_with_prefix(prefix, name)?
            .map(|attribute| attribute as &dyn IAttribute))
    }

    fn get_attribute_by_name(&self, attribute_name: &AttributeName) -> Option<&dyn IAttribute> {
        self.processable_tag
            .get_attribute_name(attribute_name)
            .map(|attribute| attribute as &dyn IAttribute)
    }

    fn get_attribute_value(
        &self,
        complete_name: &Utf16String,
    ) -> Result<Option<&Utf16String>, AttributesError> {
        Ok(self
            .processable_tag
            .get_attribute(complete_name)?
            .and_then(IAttribute::get_value))
    }

    fn get_attribute_value_with_prefix(
        &self,
        prefix: Option<&Utf16String>,
        name: &Utf16String,
    ) -> Result<Option<&Utf16String>, AttributesError> {
        Ok(self
            .processable_tag
            .get_attribute_with_prefix(prefix, name)?
            .and_then(IAttribute::get_value))
    }

    fn get_attribute_value_by_name(&self, attribute_name: &AttributeName) -> Option<&Utf16String> {
        self.processable_tag
            .get_attribute_name(attribute_name)
            .and_then(IAttribute::get_value)
    }

    fn with_attribute(
        self: Arc<Self>,
        attribute_definitions: &AttributeDefinitions,
        attribute_definition: Option<&AttributeDefinitionValue>,
        attribute_name: Utf16String,
        attribute_value: Option<Utf16String>,
        attribute_value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<dyn IProcessableElementTag>, AttributesError> {
        StandaloneElementTag::set_attribute(
            &self,
            attribute_definitions,
            attribute_definition,
            attribute_name,
            attribute_value,
            attribute_value_quotes,
        )
        .map(|tag| tag as Arc<dyn IProcessableElementTag>)
    }

    fn with_replaced_attribute(
        self: Arc<Self>,
        attribute_definitions: &AttributeDefinitions,
        old_attribute_name: &AttributeName,
        attribute_definition: Option<&AttributeDefinitionValue>,
        attribute_name: Utf16String,
        attribute_value: Option<Utf16String>,
        attribute_value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<dyn IProcessableElementTag>, AttributesError> {
        StandaloneElementTag::replace_attribute(
            &self,
            attribute_definitions,
            old_attribute_name,
            attribute_definition,
            attribute_name,
            attribute_value,
            attribute_value_quotes,
        )
        .map(|tag| tag as Arc<dyn IProcessableElementTag>)
    }

    fn without_attribute(
        self: Arc<Self>,
        attribute_name: &AttributeName,
    ) -> Arc<dyn IProcessableElementTag> {
        StandaloneElementTag::remove_attribute_name(&self, attribute_name)
    }

    fn without_attribute_complete(
        self: Arc<Self>,
        attribute_name: &Utf16String,
    ) -> Result<Arc<dyn IProcessableElementTag>, AttributesError> {
        StandaloneElementTag::remove_attribute(&self, attribute_name)
            .map(|tag| tag as Arc<dyn IProcessableElementTag>)
    }

    fn without_attribute_with_prefix(
        self: Arc<Self>,
        prefix: Option<&Utf16String>,
        name: &Utf16String,
    ) -> Result<Arc<dyn IProcessableElementTag>, AttributesError> {
        StandaloneElementTag::remove_attribute_with_prefix(&self, prefix, name)
            .map(|tag| tag as Arc<dyn IProcessableElementTag>)
    }
}

impl IElementTag for StandaloneElementTag {
    fn get_template_mode(&self) -> TemplateMode {
        self.processable_tag.as_element_tag().get_template_mode()
    }

    fn get_element_complete_name(&self) -> &Utf16String {
        self.processable_tag
            .as_element_tag()
            .get_element_complete_name()
    }

    fn get_element_definition(&self) -> &ElementDefinition {
        self.processable_tag
            .as_element_tag()
            .get_element_definition()
    }

    fn is_synthetic(&self) -> bool {
        self.processable_tag.as_element_tag().is_synthetic()
    }
}

impl ITemplateEvent for StandaloneElementTag {
    fn has_location(&self) -> bool {
        self.processable_tag
            .as_element_tag()
            .as_template_event()
            .has_location()
    }

    fn get_template_name(&self) -> Option<&Utf16String> {
        self.processable_tag
            .as_element_tag()
            .as_template_event()
            .get_template_name()
    }

    fn get_line(&self) -> i32 {
        self.processable_tag
            .as_element_tag()
            .as_template_event()
            .get_line()
    }

    fn get_col(&self) -> i32 {
        self.processable_tag
            .as_element_tag()
            .as_template_event()
            .get_col()
    }

    fn accept(&self, visitor: &mut dyn IModelVisitor) {
        visitor.visit_standalone_element_tag(self);
    }

    fn be_handled(
        self: Arc<Self>,
        handler: &mut dyn ITemplateHandler,
    ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
        handler.handle_standalone_element(self)
    }

    fn into_processable_element_tag(self: Arc<Self>) -> Option<Arc<dyn IProcessableElementTag>> {
        Some(self)
    }

    fn write(&self, writer: &mut dyn TemplateWriter) -> io::Result<()> {
        if self.is_synthetic() {
            return Ok(());
        }
        if self.get_template_mode().is_text() {
            writer.write_utf16(&[u16::from(b'['), u16::from(b'#')])?;
            writer.write_utf16(self.get_element_complete_name().as_utf16())?;
            if let Some(attributes) = self.processable_tag.attributes() {
                attributes.write(writer)?;
            }
            return if self.minimized {
                writer.write_utf16(&[u16::from(b'/'), u16::from(b']')])
            } else {
                writer.write_utf16(&[u16::from(b']')])
            };
        }
        writer.write_utf16(&[u16::from(b'<')])?;
        writer.write_utf16(self.get_element_complete_name().as_utf16())?;
        if let Some(attributes) = self.processable_tag.attributes() {
            attributes.write(writer)?;
        }
        if self.minimized {
            writer.write_utf16(&[u16::from(b'/'), u16::from(b'>')])
        } else {
            writer.write_utf16(&[u16::from(b'>')])
        }
    }
}

impl IEngineTemplateEvent for StandaloneElementTag {}

impl Display for StandaloneElementTag {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.to_utf16_string().to_string_lossy())
    }
}

fn validate_minimized(
    template_mode: TemplateMode,
    minimized: bool,
) -> Result<(), StandaloneElementTagError> {
    if !minimized && template_mode != TemplateMode::HTML {
        return Err(StandaloneElementTagError { template_mode });
    }
    Ok(())
}
