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
    /// 清除全部动作。对应 Java:
    /// `ITemplateBoundariesStructureHandler#reset()`。
    fn reset(&mut self);

    /// 在当前上下文层设置局部变量。
    ///
    /// 对应 Java: `ITemplateBoundariesStructureHandler#setLocalVariable(String,
    /// Object)`。Java `HashMap` 允许 `name` 与 `value` 为 null，同名设置以后一次为准；
    /// 此动作可与插入、删除变量、selection target 和 inliner 动作组合。
    fn set_local_variable(&mut self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>);

    /// 从当前上下文层删除局部变量。
    ///
    /// 对应 Java: `ITemplateBoundariesStructureHandler#removeLocalVariable(String)`。
    /// Java `HashSet` 允许 null 并对重复名称去重；此动作可与其他动作组合。
    fn remove_local_variable(&mut self, name: Option<JavaString>);

    /// 设置 selection target；`None` 精确对应 Java null。
    ///
    /// 对应 Java: `ITemplateBoundariesStructureHandler#setSelectionTarget(Object)`。
    fn set_selection_target(&mut self, selection_target: Option<Arc<TemplateValue>>);

    /// 设置当前 inliner；`None` 表示显式设置为 Java null。
    ///
    /// 对应 Java: `ITemplateBoundariesStructureHandler#setInliner(IInliner)`。
    fn set_inliner(&mut self, inliner: Option<Arc<dyn IInliner>>);

    /// 在模板开始事件之后或模板结束事件之前插入文本。
    ///
    /// 对应 Java: `ITemplateBoundariesStructureHandler#insert(String, boolean)`。
    /// 文本与模型插入互斥；`processable` 决定插入内容是否再次进入 Processor 链。
    fn insert_text(&mut self, text: JavaString, processable: bool);

    /// 在模板开始事件之后或模板结束事件之前插入模型。
    ///
    /// 对应 Java: `ITemplateBoundariesStructureHandler#insert(IModel, boolean)`。
    fn insert_model(&mut self, model: Arc<dyn IModel>, processable: bool);
}
