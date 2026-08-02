use crate::engine::ElementDefinition;
use crate::templatemode::TemplateMode;
use crate::util::JavaString;

use super::ITemplateEvent;

/// 打开、关闭或独立元素标签的公共事件合同。
///
/// 对应 Java: `org.thymeleaf.model.IElementTag`。
pub trait IElementTag: ITemplateEvent {
    /// 返回该标签绑定的模板模式。
    fn get_template_mode(&self) -> TemplateMode;

    /// 返回模板中原样书写的完整元素名。
    fn get_element_complete_name(&self) -> &JavaString;

    /// 返回元素元数据定义。
    fn get_element_definition(&self) -> &ElementDefinition;

    /// 判断标签是否为平衡结构时产生的合成标签。
    fn is_synthetic(&self) -> bool;
}
