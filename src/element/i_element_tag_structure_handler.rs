use std::sync::Arc;

use crate::engine::{AttributeName, TemplateData};
use crate::expression::TemplateValue;
use crate::inline::IInliner;
use crate::model::{AttributeValueQuotes, IModel};
use crate::util::JavaString;

/// ElementTag Processor 声明标签、正文、上下文和迭代变更的完整合同。
///
/// 对应 Java: `org.thymeleaf.processor.element.IElementTagStructureHandler`。
pub trait IElementTagStructureHandler {
    /// 清除全部动作。
    fn reset(&mut self);
    /// 设置局部变量。
    fn set_local_variable(&mut self, name: JavaString, value: Option<Arc<TemplateValue>>);
    /// 删除局部变量。
    fn remove_local_variable(&mut self, name: JavaString);
    /// 设置或添加属性。
    fn set_attribute(
        &mut self,
        attribute_name: JavaString,
        attribute_value: Option<JavaString>,
        quotes: Option<AttributeValueQuotes>,
    );
    /// 替换属性。
    fn replace_attribute(
        &mut self,
        old_attribute_name: &AttributeName,
        attribute_name: JavaString,
        attribute_value: Option<JavaString>,
        quotes: Option<AttributeValueQuotes>,
    );
    /// 删除解析后的属性名。
    fn remove_attribute(&mut self, attribute_name: &AttributeName);
    /// 设置 selection target。
    fn set_selection_target(&mut self, selection_target: Option<Arc<TemplateValue>>);
    /// 设置内联器。
    fn set_inliner(&mut self, inliner: Option<Arc<dyn IInliner>>);
    /// 设置模板来源数据。
    fn set_template_data(&mut self, template_data: TemplateData);
    /// 使用文本设置正文。
    fn set_body_text(&mut self, text: JavaString, processable: bool);
    /// 使用模型设置正文。
    fn set_body_model(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 在元素之前插入模型。
    fn insert_before(&mut self, model: Arc<dyn IModel>);
    /// 紧随元素之后插入模型。
    fn insert_immediately_after(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 使用文本替换整个元素。
    fn replace_with_text(&mut self, text: JavaString, processable: bool);
    /// 使用模型替换整个元素。
    fn replace_with_model(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除整个元素。
    fn remove_element(&mut self);
    /// 仅删除开始和结束标签。
    fn remove_tags(&mut self);
    /// 删除正文。
    fn remove_body(&mut self);
    /// 删除除首个子节点外的所有正文。
    fn remove_all_but_first_child(&mut self);
    /// 为当前元素建立迭代。
    fn iterate_element(
        &mut self,
        iter_variable_name: JavaString,
        iter_status_variable_name: Option<JavaString>,
        iterated_object: Option<Arc<TemplateValue>>,
    );
}
