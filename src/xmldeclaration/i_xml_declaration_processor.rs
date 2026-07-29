use crate::context::ITemplateContext;
use crate::model::IXMLDeclaration;
use crate::processor::IProcessor;

use super::IXMLDeclarationStructureHandler;

/// XMLDeclaration 事件 Processor 合同。
///
/// 对应 Java: `org.thymeleaf.processor.xmldeclaration.IXMLDeclarationProcessor`。
pub trait IXMLDeclarationProcessor: IProcessor {
    /// 处理 XML declaration。
    fn process(
        &self,
        context: &dyn ITemplateContext,
        xml_declaration: &dyn IXMLDeclaration,
        structure_handler: &mut dyn IXMLDeclarationStructureHandler,
    );
}
