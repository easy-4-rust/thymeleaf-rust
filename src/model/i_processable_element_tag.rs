use indexmap::IndexMap;

use crate::engine::{AttributeName, AttributesError};
use crate::util::JavaString;

use super::{IAttribute, IElementTag};

/// 可应用 Processor 的打开或独立元素标签合同。
///
/// 对应 Java: `org.thymeleaf.model.IProcessableElementTag`。
pub trait IProcessableElementTag: IElementTag {
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
}
