use std::sync::Arc;

use crate::expression::TemplateValue;
use crate::inline::IInliner;
use crate::model::IModel;
use crate::util::JavaString;

/// 模板开始/结束 Processor 的结构变更合同。
///
/// 对应 Java:
/// `org.thymeleaf.processor.templateboundaries.ITemplateBoundariesStructureHandler`。
pub trait ITemplateBoundariesStructureHandler {
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
    /// 插入文本。
    fn insert_text(&mut self, text: JavaString, processable: bool);
    /// 插入模型。
    fn insert_model(&mut self, model: Arc<dyn IModel>, processable: bool);
}
