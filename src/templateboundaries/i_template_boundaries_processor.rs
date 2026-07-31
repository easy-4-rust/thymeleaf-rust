use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::{ITemplateEnd, ITemplateStart};
use crate::processor::IProcessor;

use super::ITemplateBoundariesStructureHandler;

/// 模板开始和结束边界 Processor 合同。
///
/// 对应 Java:
/// `org.thymeleaf.processor.templateboundaries.ITemplateBoundariesProcessor`。
///
/// 仅为完整模板的一级 `TemplateStart`/`TemplateEnd` 触发，不为片段的内部边界
/// 重复触发；事件本身不可变。
pub trait ITemplateBoundariesProcessor: IProcessor {
    /// 处理一级 `TemplateStart`，并通过 handler 声明开始事件之后的插入或上下文变更。
    ///
    /// 对应 Java: `ITemplateBoundariesProcessor#processTemplateStart(...)`。
    fn process_template_start(
        &self,
        context: &dyn ITemplateContext,
        template_start: &dyn ITemplateStart,
        structure_handler: &mut dyn ITemplateBoundariesStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 处理一级 `TemplateEnd`，并通过 handler 声明结束事件之前的插入或上下文变更。
    ///
    /// 对应 Java: `ITemplateBoundariesProcessor#processTemplateEnd(...)`。
    fn process_template_end(
        &self,
        context: &dyn ITemplateContext,
        template_end: &dyn ITemplateEnd,
        structure_handler: &mut dyn ITemplateBoundariesStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
}
