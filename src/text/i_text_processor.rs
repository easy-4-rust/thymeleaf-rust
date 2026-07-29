use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::IText;
use crate::processor::IProcessor;

use super::ITextStructureHandler;

/// Text 事件 Processor 合同。
///
/// 对应 Java: `org.thymeleaf.processor.text.ITextProcessor`。
pub trait ITextProcessor: IProcessor {
    /// 处理文本并通过结构处理器声明变更。
    fn process(
        &self,
        context: &dyn ITemplateContext,
        text: &dyn IText,
        structure_handler: &mut dyn ITextStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
}
