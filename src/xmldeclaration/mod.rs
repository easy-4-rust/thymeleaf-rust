//! XML declaration 事件 Processor 与结构处理合同。

mod i_xml_declaration_processor;
mod i_xml_declaration_structure_handler;

pub use i_xml_declaration_processor::IXMLDeclarationProcessor;
pub use i_xml_declaration_structure_handler::IXMLDeclarationStructureHandler;
mod abstract_xml_declaration_processor;
pub use abstract_xml_declaration_processor::AbstractXMLDeclarationProcessor;
