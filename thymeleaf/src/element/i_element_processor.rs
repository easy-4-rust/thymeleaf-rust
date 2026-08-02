use crate::processor::IProcessor;

use super::{MatchingAttributeName, MatchingElementName};

/// 在元素名称和/或属性名称上执行的 Processor 基础合同。
///
/// 对应 Java: `org.thymeleaf.processor.element.IElementProcessor`。
///
/// 至少一个匹配名称应非 null；该不变量由具体抽象 Processor 构造器校验。
pub trait IElementProcessor: IProcessor + Send + Sync {
    /// 返回可选 Tag Processor capability。
    fn as_element_tag_processor(&self) -> Option<&dyn super::IElementTagProcessor> {
        None
    }

    /// 返回可选 Model Processor capability。
    fn as_element_model_processor(&self) -> Option<&dyn super::IElementModelProcessor> {
        None
    }

    /// 返回可空匹配元素名称规则。
    fn get_matching_element_name(&self) -> Option<&MatchingElementName>;

    /// 返回可空匹配属性名称规则。
    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName>;
}
