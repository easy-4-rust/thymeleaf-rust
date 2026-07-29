use crate::context::ITemplateContext;
use crate::model::{ITemplateEnd, ITemplateStart};
use crate::processor::IProcessor;

use super::ITemplateBoundariesStructureHandler;

/// 模板开始和结束边界 Processor 合同。
///
/// 对应 Java:
/// `org.thymeleaf.processor.templateboundaries.ITemplateBoundariesProcessor`。
pub trait ITemplateBoundariesProcessor: IProcessor {
    /// 处理 TemplateStart。
    fn process_template_start(
        &self,
        context: &dyn ITemplateContext,
        template_start: &dyn ITemplateStart,
        structure_handler: &mut dyn ITemplateBoundariesStructureHandler,
    );
    /// 处理 TemplateEnd。
    fn process_template_end(
        &self,
        context: &dyn ITemplateContext,
        template_end: &dyn ITemplateEnd,
        structure_handler: &mut dyn ITemplateBoundariesStructureHandler,
    );
}
