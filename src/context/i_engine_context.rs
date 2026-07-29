use std::sync::Arc;

use indexmap::IndexMap;

use crate::engine::TemplateData;
use crate::expression::TemplateValue;
use crate::inline::IInliner;
use crate::model::IProcessableElementTag;
use crate::util::JavaString;

use super::ITemplateContext;

/// 模板引擎内部可变上下文合同。
///
/// 对应 Java: `org.thymeleaf.context.IEngineContext`。
pub trait IEngineContext: ITemplateContext {
    /// 在当前上下文层设置变量。
    fn set_variable(&mut self, name: JavaString, value: Option<Arc<TemplateValue>>);
    /// 批量设置变量。
    fn set_variables(
        &mut self,
        variables: &IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>,
    );
    /// 删除变量。
    fn remove_variable(&mut self, name: &JavaString);
    /// 设置 selection target。
    fn set_selection_target(&mut self, selection_target: Option<Arc<TemplateValue>>);
    /// 设置当前内联器。
    fn set_inliner(&mut self, inliner: Option<Arc<dyn IInliner>>);
    /// 切换当前模板数据。
    fn set_template_data(&mut self, template_data: Arc<TemplateData>);
    /// 设置当前处理元素。
    fn set_element_tag(&mut self, element_tag: Option<Arc<dyn IProcessableElementTag>>);
    /// 返回指定层级之上的元素栈。
    fn get_element_stack_above(&self, context_level: i32) -> Vec<&dyn IProcessableElementTag>;
    /// 判断变量是否定义于局部层。
    fn is_variable_local(&self, name: &JavaString) -> bool;
    /// 增加上下文层级。
    fn increase_level(&mut self);
    /// 减少上下文层级并恢复局部状态。
    fn decrease_level(&mut self);
    /// 返回当前上下文层级。
    fn level(&self) -> i32;
}
