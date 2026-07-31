use std::cmp::Ordering;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use indexmap::IndexMap;

use crate::element::{IElementProcessor, MatchingElementNameError};
use crate::model::IAttribute;
use crate::templatemode::TemplateMode;
use crate::util::JavaString;

use super::{
    AbstractElementTag, Attribute, AttributeName, Attributes, AttributesError,
    ElementDefinitionValue,
};

static NEXT_TAG_IDENTITY: AtomicU64 = AtomicU64::new(1);

/// 可处理 open/standalone 标签共享的属性查询与 Processor 合并状态。
///
/// 对应 Java: `org.thymeleaf.engine.AbstractProcessableElementTag`。
pub struct AbstractProcessableElementTag {
    identity: u64,
    element_tag: AbstractElementTag,
    attributes: Option<Arc<Attributes>>,
    associated_processors: RwLock<Option<Vec<Arc<dyn IElementProcessor>>>>,
}

impl AbstractProcessableElementTag {
    /// 创建没有原模板位置的可处理标签基础状态。
    ///
    /// 对应 Java:
    /// `AbstractProcessableElementTag#AbstractProcessableElementTag(TemplateMode,ElementDefinition,String,Attributes,boolean)`。
    #[must_use]
    pub fn new(
        template_mode: TemplateMode,
        element_definition: ElementDefinitionValue,
        element_complete_name: JavaString,
        attributes: Option<Arc<Attributes>>,
        synthetic: bool,
    ) -> Self {
        Self {
            identity: NEXT_TAG_IDENTITY.fetch_add(1, AtomicOrdering::Relaxed),
            element_tag: AbstractElementTag::new(
                template_mode,
                element_definition,
                element_complete_name,
                synthetic,
            ),
            attributes,
            associated_processors: RwLock::new(None),
        }
    }

    /// 创建携带原模板位置的可处理标签基础状态。
    ///
    /// 对应 Java:
    /// `AbstractProcessableElementTag#AbstractProcessableElementTag(TemplateMode,ElementDefinition,String,Attributes,boolean,String,int,int)`。
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn with_location(
        template_mode: TemplateMode,
        element_definition: ElementDefinitionValue,
        element_complete_name: JavaString,
        attributes: Option<Arc<Attributes>>,
        synthetic: bool,
        template_name: Option<JavaString>,
        line: i32,
        col: i32,
    ) -> Self {
        Self {
            identity: NEXT_TAG_IDENTITY.fetch_add(1, AtomicOrdering::Relaxed),
            element_tag: AbstractElementTag::with_location(
                template_mode,
                element_definition,
                element_complete_name,
                synthetic,
                template_name,
                line,
                col,
            ),
            attributes,
            associated_processors: RwLock::new(None),
        }
    }

    /// 返回共享元素标签基础状态。
    #[must_use]
    pub const fn as_element_tag(&self) -> &AbstractElementTag {
        &self.element_tag
    }

    /// 返回此不可变标签实例的内部身份号。
    ///
    /// 对应 Java: 对 `AbstractProcessableElementTag` 的引用身份比较。编号避免 Rust
    /// 短生命周期 `Arc` 释放后地址复用，从而错误地把新标签识别为旧标签。
    #[must_use]
    pub(crate) const fn identity(&self) -> u64 {
        self.identity
    }

    /// 返回可空属性快照。
    #[must_use]
    pub fn attributes(&self) -> Option<&Arc<Attributes>> {
        self.attributes.as_ref()
    }

    /// 按完整名称判断属性是否存在。
    pub fn has_attribute(&self, complete_name: &JavaString) -> Result<bool, AttributesError> {
        let Some(attributes) = self.attributes.as_ref() else {
            return Ok(false);
        };
        attributes.has_attribute(self.element_tag.get_template_mode(), complete_name)
    }

    /// 按 prefix 与本地名称判断属性是否存在。
    pub fn has_attribute_with_prefix(
        &self,
        prefix: Option<&JavaString>,
        name: &JavaString,
    ) -> Result<bool, AttributesError> {
        let Some(attributes) = self.attributes.as_ref() else {
            return Ok(false);
        };
        attributes.has_attribute_with_prefix(self.element_tag.get_template_mode(), prefix, name)
    }

    /// 按规范化名称对象身份判断属性是否存在。
    #[must_use]
    pub fn has_attribute_name(&self, attribute_name: &AttributeName) -> bool {
        self.attributes
            .as_ref()
            .is_some_and(|attributes| attributes.has_attribute_base_name(attribute_name))
    }

    /// 按完整名称返回属性。
    pub fn get_attribute(
        &self,
        complete_name: &JavaString,
    ) -> Result<Option<&Attribute>, AttributesError> {
        let Some(attributes) = self.attributes.as_ref() else {
            return Ok(None);
        };
        Ok(attributes
            .get_attribute(self.element_tag.get_template_mode(), complete_name)?
            .map(Arc::as_ref))
    }

    /// 按 prefix 与本地名称返回属性。
    pub fn get_attribute_with_prefix(
        &self,
        prefix: Option<&JavaString>,
        name: &JavaString,
    ) -> Result<Option<&Attribute>, AttributesError> {
        let Some(attributes) = self.attributes.as_ref() else {
            return Ok(None);
        };
        Ok(attributes
            .get_attribute_with_prefix(self.element_tag.get_template_mode(), prefix, name)?
            .map(Arc::as_ref))
    }

    /// 按规范化名称对象身份返回属性。
    #[must_use]
    pub fn get_attribute_name(&self, attribute_name: &AttributeName) -> Option<&Attribute> {
        self.attributes
            .as_ref()
            .and_then(|attributes| attributes.get_attribute_base_name(attribute_name))
            .map(Arc::as_ref)
    }

    /// 返回属性数组的防御性浅副本。
    #[must_use]
    pub fn get_all_attributes(&self) -> Vec<Arc<Attribute>> {
        self.attributes
            .as_ref()
            .map_or_else(Vec::new, |attributes| attributes.get_all_attributes())
    }

    /// 按插入顺序返回属性名称和值的防御性 Map。
    #[must_use]
    pub fn get_attribute_map(&self) -> IndexMap<JavaString, Option<JavaString>> {
        self.attributes
            .as_ref()
            .map_or_else(IndexMap::new, |attributes| attributes.get_attribute_map())
    }

    /// 返回合并并排序后的元素/属性 Processor 快照。
    ///
    /// 对应 Java: `AbstractProcessableElementTag#getAssociatedProcessors()`。
    pub fn get_associated_processors(
        &self,
    ) -> Result<Vec<Arc<dyn IElementProcessor>>, MatchingElementNameError> {
        if let Some(processors) = read_lock(&self.associated_processors).as_ref() {
            return Ok(processors.clone());
        }
        let processors = self.compute_processors()?;
        *write_lock(&self.associated_processors) = Some(processors.clone());
        Ok(processors)
    }

    /// 判断标签是否至少关联一个适用 Processor。
    pub fn has_associated_processors(&self) -> Result<bool, MatchingElementNameError> {
        Ok(!self.get_associated_processors()?.is_empty())
    }

    fn compute_processors(
        &self,
    ) -> Result<Vec<Arc<dyn IElementProcessor>>, MatchingElementNameError> {
        let element_definition = self.element_tag.get_element_definition();
        let associated_attribute_count = self
            .attributes
            .as_ref()
            .map_or(0, |attributes| attributes.get_associated_processor_count());
        if self.attributes.is_none() || associated_attribute_count == 0 {
            return Ok(element_definition.sorted_associated_processors().to_vec());
        }

        let mut processors = element_definition.sorted_associated_processors().to_vec();
        let attributes = self
            .attributes
            .as_ref()
            .and_then(|attributes| attributes.as_attribute_slice())
            .expect("positive associated count requires attributes");
        for attribute in attributes.iter().rev() {
            let definition = attribute.get_attribute_definition();
            if !definition.has_associated_processors() {
                continue;
            }
            for processor in definition.sorted_associated_processors() {
                if let Some(matching_element_name) = processor.get_matching_element_name()
                    && !matching_element_name
                        .matches(Some(element_definition.get_element_name()))?
                {
                    continue;
                }
                processors.push(processor.clone());
            }
        }
        if processors.len() > 1 {
            processors.sort_by(compare_processors);
        }
        Ok(processors)
    }
}

fn compare_processors(
    left: &Arc<dyn IElementProcessor>,
    right: &Arc<dyn IElementProcessor>,
) -> Ordering {
    if Arc::ptr_eq(left, right) {
        return Ordering::Equal;
    }
    crate::util::ProcessorComparators::compare_processors(left.as_ref(), right.as_ref())
}

fn read_lock<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
