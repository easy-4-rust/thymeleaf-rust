use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::IXMLDeclaration;
use crate::processor::IProcessor;

use super::IXMLDeclarationStructureHandler;

/// XMLDeclaration 事件 Processor 合同。
///
/// 对应 Java: `org.thymeleaf.processor.xmldeclaration.IXMLDeclarationProcessor`。
pub trait IXMLDeclarationProcessor: IProcessor {
    /// 处理 XML declaration。
    ///
    /// 对应 Java: `IXMLDeclarationProcessor#process(ITemplateContext,
    /// IXMLDeclaration, IXMLDeclarationStructureHandler)`。事件不可变，结构变更通过
    /// handler 声明。
    fn process(
        &self,
        context: &dyn ITemplateContext,
        xml_declaration: &dyn IXMLDeclaration,
        structure_handler: &mut dyn IXMLDeclarationStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
}
