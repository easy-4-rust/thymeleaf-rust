use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use indexmap::IndexMap;

use crate::model::{AttributeValueQuotes, IAttribute};
use crate::templatemode::TemplateMode;
use crate::util::{FastStringWriter, JavaWriter, Utf16String};

use super::{
    Attribute, AttributeDefinitionValue, AttributeDefinitions, AttributeDefinitionsError,
    AttributeName, AttributeNameValue, AttributeNames, AttributeNamesError,
};

const DEFAULT_WHITE_SPACE: &str = " ";
thread_local! {
    // IElementProcessor 与 Java 一样允许非线程安全的自定义实现，因此 Attributes
    // 不能被谎称为 Sync。空快照按模板执行线程复用，避免为全局常量引入不安全实现。
    static EMPTY_ATTRIBUTES: RefCell<Option<Arc<Attributes>>> = const { RefCell::new(None) };
}

/// 属性集合修改或名称解析错误。
#[derive(Clone, Debug, Eq, PartialEq)]
/// 对应 Java 语义：`Attributes` 的 Rust 侧类型 `AttributesError`。
pub enum AttributesError {
    /// XML 模式不允许值为 null 的属性。
    NullValueInXml,
    /// XML 模式不允许无引号属性值。
    UnquotedValueInXml,
    /// 属性名规范化失败。
    AttributeNames(AttributeNamesError),
    /// 属性 Definition 获取失败。
    AttributeDefinitions(AttributeDefinitionsError),
}

impl AttributesError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub fn java_class_name(&self) -> &str {
        match self {
            Self::NullValueInXml | Self::UnquotedValueInXml => "java.lang.IllegalArgumentException",
            Self::AttributeNames(error) => error.java_class_name(),
            Self::AttributeDefinitions(error) => error.java_class_name(),
        }
    }
}

impl Display for AttributesError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullValueInXml => {
                formatter.write_str("Cannot set null-value attributes in XML template mode")
            }
            Self::UnquotedValueInXml => {
                formatter.write_str("Cannot set unquoted attributes in XML template mode")
            }
            Self::AttributeNames(error) => Display::fmt(error, formatter),
            Self::AttributeDefinitions(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for AttributesError {}

impl From<AttributeNamesError> for AttributesError {
    fn from(value: AttributeNamesError) -> Self {
        Self::AttributeNames(value)
    }
}

impl From<AttributeDefinitionsError> for AttributesError {
    fn from(value: AttributeDefinitionsError) -> Self {
        Self::AttributeDefinitions(value)
    }
}

/// 元素内部属性及属性间原始空白的不可变快照。
///
/// 对应 Java: `org.thymeleaf.engine.Attributes`。
///
/// 每次修改都会返回新快照；找不到待删除属性时返回原 `Arc`，从而保留 Java
/// 原对象身份。属性数组保存 `Arc<Attribute>`，防御性克隆只复制引用，与 Java
/// `Attribute[]#clone()` 的浅克隆语义一致。
pub struct Attributes {
    attributes: Option<Vec<Arc<Attribute>>>,
    inner_white_spaces: Option<Vec<Utf16String>>,
    associated_processor_count: AtomicI32,
}

impl Attributes {
    /// 使用属性数组与原始内部空白创建快照。
    ///
    /// 对应 Java: `Attributes#Attributes(Attribute[],String[])`。调用方负责维持
    /// Java 引擎内部数组约束：每个属性之前均有一个空白项，末尾可再有一个空白项。
    #[must_use]
    pub fn new(
        attributes: Option<Vec<Arc<Attribute>>>,
        inner_white_spaces: Option<Vec<Utf16String>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            attributes,
            inner_white_spaces,
            associated_processor_count: AtomicI32::new(-1),
        })
    }

    /// 返回全局空属性快照。
    ///
    /// 对应 Java: `Attributes.EMPTY_ATTRIBUTES`。
    #[must_use]
    pub fn empty() -> Arc<Self> {
        EMPTY_ATTRIBUTES.with(|slot| {
            let mut slot = slot.borrow_mut();
            slot.get_or_insert_with(|| Self::new(None, None)).clone()
        })
    }

    /// 返回全部关联属性 Processor 的数量并惰性缓存。
    ///
    /// 对应 Java: `Attributes#getAssociatedProcessorCount()`。
    #[must_use]
    pub fn get_associated_processor_count(&self) -> i32 {
        let cached = self.associated_processor_count.load(Ordering::Acquire);
        if cached >= 0 {
            return cached;
        }
        let computed = self.compute_associated_processor_count();
        self.associated_processor_count
            .store(computed, Ordering::Release);
        computed
    }

    /// 判断是否存在指定完整属性名。
    ///
    /// 对应 Java: `Attributes#hasAttribute(TemplateMode,String)`。
    pub fn has_attribute(
        &self,
        template_mode: TemplateMode,
        complete_name: &Utf16String,
    ) -> Result<bool, AttributesError> {
        Ok(self.search_attribute(template_mode, complete_name)? >= 0)
    }

    /// 判断是否存在指定 prefix 与本地名的属性。
    ///
    /// 对应 Java: `Attributes#hasAttribute(TemplateMode,String,String)`。
    pub fn has_attribute_with_prefix(
        &self,
        template_mode: TemplateMode,
        prefix: Option<&Utf16String>,
        name: &Utf16String,
    ) -> Result<bool, AttributesError> {
        Ok(self.search_attribute_with_prefix(template_mode, prefix, name)? >= 0)
    }

    /// 按 repository 单例属性名判断属性是否存在。
    ///
    /// 对应 Java: `Attributes#hasAttribute(AttributeName)`。
    #[must_use]
    pub fn has_attribute_name(&self, attribute_name: &AttributeNameValue) -> bool {
        self.search_attribute_name(attribute_name) >= 0
    }

    /// 按公共 `AttributeName` 基类身份判断属性是否存在。
    ///
    /// 对应 Java: `Attributes#hasAttribute(AttributeName)`。
    #[must_use]
    pub fn has_attribute_base_name(&self, attribute_name: &AttributeName) -> bool {
        self.search_attribute_base_name(attribute_name) >= 0
    }

    /// 按完整属性名返回属性。
    ///
    /// 对应 Java: `Attributes#getAttribute(TemplateMode,String)`。
    pub fn get_attribute(
        &self,
        template_mode: TemplateMode,
        complete_name: &Utf16String,
    ) -> Result<Option<&Arc<Attribute>>, AttributesError> {
        let position = self.search_attribute(template_mode, complete_name)?;
        Ok(self.attribute_at(position))
    }

    /// 按 prefix 与本地名返回属性。
    ///
    /// 对应 Java: `Attributes#getAttribute(TemplateMode,String,String)`。
    pub fn get_attribute_with_prefix(
        &self,
        template_mode: TemplateMode,
        prefix: Option<&Utf16String>,
        name: &Utf16String,
    ) -> Result<Option<&Arc<Attribute>>, AttributesError> {
        let position = self.search_attribute_with_prefix(template_mode, prefix, name)?;
        Ok(self.attribute_at(position))
    }

    /// 按 repository 单例属性名返回属性。
    ///
    /// 对应 Java: `Attributes#getAttribute(AttributeName)`。
    #[must_use]
    pub fn get_attribute_name(
        &self,
        attribute_name: &AttributeNameValue,
    ) -> Option<&Arc<Attribute>> {
        self.attribute_at(self.search_attribute_name(attribute_name))
    }

    /// 按公共 `AttributeName` 基类身份返回属性。
    ///
    /// 对应 Java: `Attributes#getAttribute(AttributeName)`。
    #[must_use]
    pub fn get_attribute_base_name(
        &self,
        attribute_name: &AttributeName,
    ) -> Option<&Arc<Attribute>> {
        self.attribute_at(self.search_attribute_base_name(attribute_name))
    }

    /// 返回属性数组的防御性浅克隆。
    ///
    /// 对应 Java: `Attributes#getAllAttributes()`。
    #[must_use]
    pub fn get_all_attributes(&self) -> Vec<Arc<Attribute>> {
        self.attributes.clone().unwrap_or_default()
    }

    /// 返回引擎内部属性引用切片；无属性时返回 `None`。
    ///
    /// 该入口用于 Processor 合并，不执行防御性复制。
    #[must_use]
    /// 对应 Java 语义：`Attributes` 的 `as_attribute_slice` 行为（Rust 侧辅助/私有路径）。
    pub fn as_attribute_slice(&self) -> Option<&[Arc<Attribute>]> {
        self.attributes.as_deref()
    }

    /// 返回 parser/Handler 内部使用的原始属性间空白切片。
    ///
    /// 对应 Java 同包代码对 `Attributes.innerWhiteSpaces` 的构造语义；仅用于在
    /// decoupled logic 注入时保留“最后一个事件是否为空白”的状态。
    pub(crate) fn inner_white_spaces(&self) -> Option<&[Utf16String]> {
        self.inner_white_spaces.as_deref()
    }

    /// 按模板出现顺序返回完整属性名到可空值的映射。
    ///
    /// 对应 Java: `Attributes#getAttributeMap()`。
    #[must_use]
    pub fn get_attribute_map(&self) -> IndexMap<Utf16String, Option<Utf16String>> {
        let Some(attributes) = self.attributes.as_ref() else {
            return IndexMap::new();
        };
        let mut result = IndexMap::with_capacity(attributes.len() + 5);
        for attribute in attributes {
            result.insert(
                attribute.get_attribute_complete_name().clone(),
                attribute.get_value().cloned(),
            );
        }
        result
    }

    /// 新增属性或修改已有属性，返回新的不可变快照。
    ///
    /// 对应 Java: `Attributes#setAttribute(...)`。
    pub fn set_attribute(
        self: &Arc<Self>,
        attribute_definitions: &AttributeDefinitions,
        template_mode: TemplateMode,
        attribute_definition: Option<&AttributeDefinitionValue>,
        complete_name: Utf16String,
        value: Option<Utf16String>,
        value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<Self>, AttributesError> {
        validate_xml_value(template_mode, value.as_ref(), value_quotes)?;
        let existing_index = match attribute_definition {
            Some(definition) => self.search_attribute_definition_name(definition),
            None => self.search_attribute(template_mode, &complete_name)?,
        };

        if existing_index >= 0 {
            let mut attributes = self
                .attributes
                .as_ref()
                .expect("non-negative search result requires attributes")
                .clone();
            let index = existing_index as usize;
            attributes[index] =
                Arc::new(attributes[index].modify(None, Some(complete_name), value, value_quotes));
            return Ok(Self::new(
                attributes.into(),
                self.inner_white_spaces.clone(),
            ));
        }

        let definition = match attribute_definition {
            Some(definition) => definition.clone(),
            None => attribute_definitions.for_name(Some(template_mode), Some(&complete_name))?,
        };
        let attribute = Arc::new(Attribute::new(
            definition,
            complete_name,
            None,
            value,
            value_quotes,
            None,
            -1,
            -1,
        ));
        let old_attribute_count = self.attributes.as_ref().map_or(0, Vec::len);
        let mut attributes = self.attributes.clone().unwrap_or_default();
        attributes.push(attribute);

        let inner_white_spaces = match self.inner_white_spaces.as_ref() {
            Some(existing) => {
                let mut spaces = existing.clone();
                if existing.len() == old_attribute_count {
                    spaces.push(default_white_space());
                } else {
                    let final_space = spaces
                        .last()
                        .expect("non-empty trailing whitespace array")
                        .clone();
                    let last = spaces.len() - 1;
                    spaces[last] = default_white_space();
                    spaces.push(final_space);
                }
                spaces
            }
            None => vec![default_white_space()],
        };
        Ok(Self::new(Some(attributes), Some(inner_white_spaces)))
    }

    /// 将旧属性替换为新属性，并合并可能已存在的新名称。
    ///
    /// 对应 Java: `Attributes#replaceAttribute(...)`。
    #[allow(clippy::too_many_arguments)]
    pub fn replace_attribute(
        self: &Arc<Self>,
        attribute_definitions: &AttributeDefinitions,
        template_mode: TemplateMode,
        old_name: &AttributeName,
        new_attribute_definition: Option<&AttributeDefinitionValue>,
        new_complete_name: Utf16String,
        value: Option<Utf16String>,
        value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<Self>, AttributesError> {
        validate_xml_value(template_mode, value.as_ref(), value_quotes)?;
        if self.attributes.is_none() {
            return self.set_attribute(
                attribute_definitions,
                template_mode,
                new_attribute_definition,
                new_complete_name,
                value,
                value_quotes,
            );
        }

        let old_index = self.search_attribute_base_name(old_name);
        if old_index < 0 {
            return self.set_attribute(
                attribute_definitions,
                template_mode,
                new_attribute_definition,
                new_complete_name,
                value,
                value_quotes,
            );
        }

        let mut existing_index = match new_attribute_definition {
            Some(definition) => self.search_attribute_definition_name(definition),
            None => self.search_attribute(template_mode, &new_complete_name)?,
        };
        if existing_index >= 0 {
            if old_index == existing_index {
                return self.set_attribute(
                    attribute_definitions,
                    template_mode,
                    new_attribute_definition,
                    new_complete_name,
                    value,
                    value_quotes,
                );
            }

            let mut attributes = self.attributes.as_ref().expect("attributes").clone();
            attributes.remove(old_index as usize);
            let mut spaces = self
                .inner_white_spaces
                .as_ref()
                .expect("attributes require inner whitespace")
                .clone();
            let whitespace_index =
                if old_index as usize + 1 == self.attributes.as_ref().expect("attributes").len() {
                    old_index as usize
                } else {
                    old_index as usize + 1
                };
            spaces.remove(whitespace_index);
            if existing_index > old_index {
                existing_index -= 1;
            }
            let index = existing_index as usize;
            attributes[index] = Arc::new(attributes[index].modify(
                None,
                Some(new_complete_name),
                value,
                value_quotes,
            ));
            return Ok(Self::new(Some(attributes), Some(spaces)));
        }

        let definition = match new_attribute_definition {
            Some(definition) => definition.clone(),
            None => {
                attribute_definitions.for_name(Some(template_mode), Some(&new_complete_name))?
            }
        };
        let mut attributes = self.attributes.as_ref().expect("attributes").clone();
        let index = old_index as usize;
        attributes[index] = Arc::new(attributes[index].modify(
            Some(definition),
            Some(new_complete_name),
            value,
            value_quotes,
        ));
        Ok(Self::new(Some(attributes), self.inner_white_spaces.clone()))
    }

    /// 按完整属性名删除属性；不存在时返回原对象。
    ///
    /// 对应 Java: `Attributes#removeAttribute(TemplateMode,String)`。
    pub fn remove_attribute(
        self: &Arc<Self>,
        template_mode: TemplateMode,
        complete_name: &Utf16String,
    ) -> Result<Arc<Self>, AttributesError> {
        if self.attributes.is_none() {
            return Ok(self.clone());
        }
        let index = self.search_attribute(template_mode, complete_name)?;
        Ok(self.remove_attribute_at(index))
    }

    /// 按 prefix 与本地名删除属性；不存在时返回原对象。
    ///
    /// 对应 Java: `Attributes#removeAttribute(TemplateMode,String,String)`。
    pub fn remove_attribute_with_prefix(
        self: &Arc<Self>,
        template_mode: TemplateMode,
        prefix: Option<&Utf16String>,
        name: &Utf16String,
    ) -> Result<Arc<Self>, AttributesError> {
        if self.attributes.is_none() {
            return Ok(self.clone());
        }
        let index = self.search_attribute_with_prefix(template_mode, prefix, name)?;
        Ok(self.remove_attribute_at(index))
    }

    /// 按 repository 单例属性名删除属性；不存在时返回原对象。
    ///
    /// 对应 Java: `Attributes#removeAttribute(AttributeName)`。
    #[must_use]
    pub fn remove_attribute_name(
        self: &Arc<Self>,
        attribute_name: &AttributeNameValue,
    ) -> Arc<Self> {
        if self.attributes.is_none() {
            return self.clone();
        }
        self.remove_attribute_at(self.search_attribute_name(attribute_name))
    }

    /// 按公共 `AttributeName` 基类身份删除属性；不存在时返回原对象。
    ///
    /// 对应 Java: `Attributes#removeAttribute(AttributeName)`。
    #[must_use]
    pub fn remove_attribute_base_name(
        self: &Arc<Self>,
        attribute_name: &AttributeName,
    ) -> Arc<Self> {
        if self.attributes.is_none() {
            return self.clone();
        }
        self.remove_attribute_at(self.search_attribute_base_name(attribute_name))
    }

    /// 按原始空白与引号形态写出全部属性。
    ///
    /// 对应 Java: `Attributes#write(Writer)`。
    pub fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        let Some(attributes) = self.attributes.as_ref() else {
            if let Some(spaces) = self.inner_white_spaces.as_ref() {
                writer.write_utf16(spaces[0].as_utf16())?;
            }
            return Ok(());
        };
        let spaces = self
            .inner_white_spaces
            .as_ref()
            .expect("attributes require inner whitespace");
        for (index, attribute) in attributes.iter().enumerate() {
            writer.write_utf16(spaces[index].as_utf16())?;
            attribute.write(writer)?;
        }
        if attributes.len() < spaces.len() {
            writer.write_utf16(spaces[attributes.len()].as_utf16())?;
        }
        Ok(())
    }

    /// 返回 Java `toString()` 对应的 UTF-16 属性序列。
    #[must_use]
    /// 对应 Java 语义：`Attributes` 的 `to_utf16_string` 行为（Rust 侧辅助/私有路径）。
    pub fn to_utf16_string(&self) -> Utf16String {
        let mut writer = FastStringWriter::new();
        self.write(&mut writer)
            .expect("FastStringWriter must accept complete UTF-16 slices");
        writer.to_string()
    }

    fn compute_associated_processor_count(&self) -> i32 {
        let Some(attributes) = self.attributes.as_ref() else {
            return 0;
        };
        let mut count = 0_i32;
        for attribute in attributes.iter().rev() {
            let definition = attribute.get_attribute_definition();
            if definition.has_associated_processors() {
                count = count.wrapping_add(definition.sorted_associated_processors().len() as i32);
            }
        }
        count
    }

    fn search_attribute(
        &self,
        template_mode: TemplateMode,
        complete_name: &Utf16String,
    ) -> Result<i32, AttributesError> {
        let Some(attributes) = self.attributes.as_ref() else {
            return Ok(-1);
        };
        for (index, attribute) in attributes.iter().enumerate().rev() {
            if attribute.get_attribute_complete_name() == complete_name {
                return Ok(index as i32);
            }
        }
        let name = AttributeNames::for_name(Some(template_mode), Some(complete_name))?;
        Ok(self.search_attribute_name(&name))
    }

    fn search_attribute_with_prefix(
        &self,
        template_mode: TemplateMode,
        prefix: Option<&Utf16String>,
        name: &Utf16String,
    ) -> Result<i32, AttributesError> {
        if prefix.is_none_or(Utf16String::is_empty) {
            return self.search_attribute(template_mode, name);
        }
        let attribute_name =
            AttributeNames::for_name_with_prefix(Some(template_mode), prefix, Some(name))?;
        Ok(self.search_attribute_name(&attribute_name))
    }

    fn search_attribute_definition_name(&self, definition: &AttributeDefinitionValue) -> i32 {
        let name = definition.as_attribute_definition().get_attribute_name();
        self.search_attribute_name(name)
    }

    fn search_attribute_name(&self, attribute_name: &AttributeNameValue) -> i32 {
        self.search_attribute_base_name(attribute_name.as_attribute_name())
    }

    fn search_attribute_base_name(&self, attribute_name: &AttributeName) -> i32 {
        let Some(attributes) = self.attributes.as_ref() else {
            return -1;
        };
        for (index, attribute) in attributes.iter().enumerate().rev() {
            let actual = attribute
                .get_attribute_definition()
                .get_attribute_name()
                .as_attribute_name();
            if std::ptr::eq(actual, attribute_name) {
                return index as i32;
            }
        }
        -1
    }

    fn attribute_at(&self, position: i32) -> Option<&Arc<Attribute>> {
        if position < 0 {
            return None;
        }
        self.attributes
            .as_ref()
            .and_then(|attributes| attributes.get(position as usize))
    }

    fn remove_attribute_at(self: &Arc<Self>, attribute_index: i32) -> Arc<Self> {
        if attribute_index < 0 {
            return self.clone();
        }
        let attributes = self.attributes.as_ref().expect("attributes");
        let spaces = self
            .inner_white_spaces
            .as_ref()
            .expect("attributes require inner whitespace");
        if attributes.len() == 1 && spaces.len() == 1 {
            return Self::empty();
        }

        let index = attribute_index as usize;
        let new_attributes = if attributes.len() == 1 {
            None
        } else {
            let mut result = attributes.clone();
            result.remove(index);
            Some(result)
        };
        let whitespace_index = if index + 1 == attributes.len() {
            index
        } else {
            index + 1
        };
        let mut new_spaces = spaces.clone();
        new_spaces.remove(whitespace_index);
        Self::new(new_attributes, Some(new_spaces))
    }
}

impl Display for Attributes {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.to_utf16_string().to_string_lossy())
    }
}

fn validate_xml_value(
    template_mode: TemplateMode,
    value: Option<&Utf16String>,
    value_quotes: Option<AttributeValueQuotes>,
) -> Result<(), AttributesError> {
    if template_mode == TemplateMode::XML && value.is_none() {
        return Err(AttributesError::NullValueInXml);
    }
    if template_mode == TemplateMode::XML && value_quotes == Some(AttributeValueQuotes::NONE) {
        return Err(AttributesError::UnquotedValueInXml);
    }
    Ok(())
}

fn default_white_space() -> Utf16String {
    Utf16String::from_rust_str(DEFAULT_WHITE_SPACE)
}
