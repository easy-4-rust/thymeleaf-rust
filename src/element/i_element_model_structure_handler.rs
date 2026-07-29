use std::sync::Arc;

use crate::engine::TemplateData;
use crate::expression::TemplateValue;
use crate::inline::IInliner;
use crate::util::JavaString;

/// ElementModel Processor 的上下文结构变更合同。
///
/// 对应 Java:
/// `org.thymeleaf.processor.element.IElementModelStructureHandler`。
pub trait IElementModelStructureHandler {
    /// 清除全部动作。
    fn reset(&mut self);
    /// 设置局部变量。
    fn set_local_variable(&mut self, name: JavaString, value: Option<Arc<TemplateValue>>);
    /// 删除局部变量。
    fn remove_local_variable(&mut self, name: JavaString);
    /// 设置 selection target。
    fn set_selection_target(&mut self, selection_target: Option<Arc<TemplateValue>>);
    /// 设置内联器。
    fn set_inliner(&mut self, inliner: Option<Arc<dyn IInliner>>);
    /// 设置后续模型事件的模板来源数据。
    fn set_template_data(&mut self, template_data: Arc<TemplateData>);
}
