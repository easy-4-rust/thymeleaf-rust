use std::any::TypeId;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::engine::TemplateData;
use crate::exceptions::TemplateProcessingException;
use crate::expression::TemplateValue;
use crate::inline::IInliner;
use crate::model::{IModelFactory, IProcessableElementTag};
use crate::util::Utf16String;
use crate::{TemplateMode, TemplateResolutionAttributes};

use super::{IExpressionContext, IdentifierSequences};

/// 模板 Processor 执行期间可读取的完整上下文合同。
///
/// 对应 Java: `org.thymeleaf.context.ITemplateContext`。
pub trait ITemplateContext: IExpressionContext {
    /// 返回当前事件来源模板的数据。
    fn get_template_data(&self) -> Arc<TemplateData>;
    /// 返回当前事件来源模板模式。
    fn get_template_mode(&self) -> TemplateMode;
    /// 返回从顶层模板到当前模板的调用栈。
    fn get_template_stack(&self) -> Vec<Arc<TemplateData>>;
    /// 返回处理时元素栈。
    fn get_element_stack(&self) -> Vec<Arc<dyn IProcessableElementTag>>;
    /// 返回模板解析属性。
    fn get_template_resolution_attributes(&self) -> Option<&TemplateResolutionAttributes>;
    /// 返回当前模式的模型工厂。
    fn get_model_factory(&self) -> &dyn IModelFactory;
    /// 判断是否存在 selection target。
    fn has_selection_target(&self) -> bool;
    /// 返回 selection target。
    fn get_selection_target(&self) -> Option<Arc<TemplateValue>>;
    /// 返回当前内联器。
    fn get_inliner(&self) -> Option<Arc<dyn IInliner>>;
    /// 解析外部化消息。
    fn get_message(
        &self,
        origin: Option<TypeId>,
        key: &Utf16String,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
        use_absent_message_representation: bool,
    ) -> crate::messageresolver::MessageResolutionResult<Option<Utf16String>>;
    /// 构建模板链接。
    fn build_link(
        &self,
        base: Option<&Utf16String>,
        parameters: Option<&IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>,
    ) -> Result<Utf16String, TemplateProcessingException>;
    /// 返回上下文级唯一标识序列。
    fn get_identifier_sequences(&self) -> &IdentifierSequences;
}
