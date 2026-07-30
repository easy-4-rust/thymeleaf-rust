use std::sync::Arc;

use indexmap::IndexMap;

use crate::engine::{
    AttributeDefinitionValue, AttributeDefinitions, AttributeName, AttributesError,
};
use crate::util::JavaString;

use super::{AttributeValueQuotes, IAttribute, IElementTag};

/// 可应用 Processor 的打开或独立元素标签合同。
///
/// 对应 Java: `org.thymeleaf.model.IProcessableElementTag`。
pub trait IProcessableElementTag: IElementTag {
    /// 返回引擎内建可处理标签的共享基础状态。
    ///
    /// Java 通过 `instanceof AbstractProcessableElementTag` 使用属性内表达式缓存；
    /// 第三方标签默认返回 `None` 并直接解析。
    fn as_engine_processable_element_tag(
        &self,
    ) -> Option<&crate::engine::AbstractProcessableElementTag> {
        None
    }

    /// 若标签是开放元素，则消费共享引用并保持同一对象身份。
    fn into_open_element_tag(self: Arc<Self>) -> Option<Arc<dyn super::IOpenElementTag>> {
        None
    }

    /// 若标签是 standalone 元素，则消费共享引用并保持同一对象身份。
    fn into_standalone_element_tag(
        self: Arc<Self>,
    ) -> Option<Arc<dyn super::IStandaloneElementTag>> {
        None
    }

    /// 返回属性数组的防御性浅副本；无属性时返回空数组。
    fn get_all_attributes(&self) -> Vec<&dyn IAttribute>;

    /// 返回完整属性名到可空属性值的防御性映射。
    fn get_attribute_map(&self) -> IndexMap<JavaString, Option<JavaString>>;

    /// 按完整名称判断属性是否存在。
    fn has_attribute(&self, complete_name: &JavaString) -> Result<bool, AttributesError>;

    /// 按可空 prefix 与本地名判断属性是否存在。
    fn has_attribute_with_prefix(
        &self,
        prefix: Option<&JavaString>,
        name: &JavaString,
    ) -> Result<bool, AttributesError>;

    /// 按规范化属性名判断属性是否存在。
    fn has_attribute_name(&self, attribute_name: &AttributeName) -> bool;

    /// 按完整名称返回可空属性对象。
    fn get_attribute(
        &self,
        complete_name: &JavaString,
    ) -> Result<Option<&dyn IAttribute>, AttributesError>;

    /// 按可空 prefix 与本地名返回可空属性对象。
    fn get_attribute_with_prefix(
        &self,
        prefix: Option<&JavaString>,
        name: &JavaString,
    ) -> Result<Option<&dyn IAttribute>, AttributesError>;

    /// 按规范化属性名返回可空属性对象。
    fn get_attribute_by_name(&self, attribute_name: &AttributeName) -> Option<&dyn IAttribute>;

    /// 按完整名称返回可空属性值。
    fn get_attribute_value(
        &self,
        complete_name: &JavaString,
    ) -> Result<Option<&JavaString>, AttributesError>;

    /// 按可空 prefix 与本地名返回可空属性值。
    fn get_attribute_value_with_prefix(
        &self,
        prefix: Option<&JavaString>,
        name: &JavaString,
    ) -> Result<Option<&JavaString>, AttributesError>;

    /// 按规范化属性名返回可空属性值。
    fn get_attribute_value_by_name(&self, attribute_name: &AttributeName) -> Option<&JavaString>;

    /// 在当前不可变标签上设置属性并返回派生标签。
    fn with_attribute(
        self: Arc<Self>,
        attribute_definitions: &AttributeDefinitions,
        attribute_definition: Option<&AttributeDefinitionValue>,
        attribute_name: JavaString,
        attribute_value: Option<JavaString>,
        attribute_value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<dyn IProcessableElementTag>, AttributesError>;

    /// 替换指定规范化属性并返回派生标签。
    fn with_replaced_attribute(
        self: Arc<Self>,
        attribute_definitions: &AttributeDefinitions,
        old_attribute_name: &AttributeName,
        attribute_definition: Option<&AttributeDefinitionValue>,
        attribute_name: JavaString,
        attribute_value: Option<JavaString>,
        attribute_value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<dyn IProcessableElementTag>, AttributesError>;

    /// 删除指定规范化属性；不存在时保留当前 `Arc` 对象身份。
    fn without_attribute(
        self: Arc<Self>,
        attribute_name: &AttributeName,
    ) -> Arc<dyn IProcessableElementTag>;

    /// 按完整名称删除属性。
    fn without_attribute_complete(
        self: Arc<Self>,
        attribute_name: &JavaString,
    ) -> Result<Arc<dyn IProcessableElementTag>, AttributesError>;

    /// 按 prefix 与本地名称删除属性。
    fn without_attribute_with_prefix(
        self: Arc<Self>,
        prefix: Option<&JavaString>,
        name: &JavaString,
    ) -> Result<Arc<dyn IProcessableElementTag>, AttributesError>;
}
