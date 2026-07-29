use crate::context::ITemplateContext;
use crate::model::IDocType;
use crate::processor::IProcessor;

use super::IDocTypeStructureHandler;

/// DOCTYPE 事件 Processor 合同。
///
/// 对应 Java: `org.thymeleaf.processor.doctype.IDocTypeProcessor`。
pub trait IDocTypeProcessor: IProcessor {
    /// 处理 DOCTYPE 事件。
    fn process(
        &self,
        context: &dyn ITemplateContext,
        doc_type: &dyn IDocType,
        structure_handler: &mut dyn IDocTypeStructureHandler,
    );
}
