use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::ICDATASection;
use crate::processor::IProcessor;

use super::ICDATASectionStructureHandler;

/// CDATA 事件 Processor 合同。
///
/// 对应 Java: `org.thymeleaf.processor.cdatasection.ICDATASectionProcessor`。
pub trait ICDATASectionProcessor: IProcessor {
    /// 处理 CDATA 事件。
    ///
    /// 对应 Java: `ICDATASectionProcessor#process(ITemplateContext,
    /// ICDATASection, ICDATASectionStructureHandler)`。事件不可变，结构变更通过
    /// `structure_handler` 声明。
    fn process(
        &self,
        context: &dyn ITemplateContext,
        cdata_section: &dyn ICDATASection,
        structure_handler: &mut dyn ICDATASectionStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
}
