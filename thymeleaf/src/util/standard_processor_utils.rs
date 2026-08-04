use crate::element::IElementTagStructureHandler;
use crate::engine::{AttributeDefinition, AttributeNameValue};
use crate::model::AttributeValueQuotes;
use crate::util::Utf16String;

/// Standard Dialect 修改事件属性的内部工具。
///
/// 对应 Java: `org.thymeleaf.standard.util.StandardProcessorUtils`。
pub struct StandardProcessorUtils;

impl StandardProcessorUtils {
    /// 用新属性替换旧属性。
    /// 对应 Java: `StandardProcessorUtils#replaceAttribute()`。
    pub fn replace_attribute(
        structure_handler: &mut dyn IElementTagStructureHandler,
        old_attribute_name: AttributeNameValue,
        _attribute_definition: &AttributeDefinition,
        attribute_name: Utf16String,
        attribute_value: Option<Utf16String>,
    ) {
        structure_handler.replace_attribute(
            old_attribute_name,
            attribute_name,
            attribute_value,
            None::<AttributeValueQuotes>,
        );
    }

    /// 设置属性。
    /// 对应 Java: `StandardProcessorUtils#setAttribute()`。
    pub fn set_attribute(
        structure_handler: &mut dyn IElementTagStructureHandler,
        _attribute_definition: &AttributeDefinition,
        attribute_name: Utf16String,
        attribute_value: Option<Utf16String>,
    ) {
        structure_handler.set_attribute(
            attribute_name,
            attribute_value,
            None::<AttributeValueQuotes>,
        );
    }
}
