//! XML declaration 事件 Processor 与结构处理合同。

mod ixml_declaration_processor;
mod ixml_declaration_structure_handler;

pub use ixml_declaration_processor::IXMLDeclarationProcessor;
pub use ixml_declaration_structure_handler::IXMLDeclarationStructureHandler;
mod abstract_xml_declaration_processor;
pub use abstract_xml_declaration_processor::AbstractXMLDeclarationProcessor;
